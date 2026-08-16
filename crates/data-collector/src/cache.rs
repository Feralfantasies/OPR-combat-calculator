//! Cache layer for storing raw HTML files with metadata

use crate::error::{CollectorError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Metadata for cached files
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// Map of URL to cache entry
    pub entries: HashMap<String, CacheEntry>,
}

/// Entry for a single cached file
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheEntry {
    /// Path to the cached file relative to cache directory
    pub file_path: String,
    /// Unix timestamp when the file was fetched
    pub fetched_at: u64,
    /// Original URL that was fetched
    pub url: String,
}

/// Cache manager for storing and retrieving HTML files
pub struct Cache {
    /// Directory where cache files are stored
    cache_dir: PathBuf,
    /// In-memory metadata
    metadata: CacheMetadata,
}

impl Cache {
    /// Create a new cache manager
    ///
    /// # Errors
    ///
    /// Returns an error if the cache directory cannot be created or metadata cannot be loaded
    pub async fn new(cache_dir: &Path) -> Result<Self> {
        fs::create_dir_all(cache_dir).await?;

        let metadata_path = cache_dir.join("metadata.json");
        let metadata = if metadata_path.exists() {
            let contents = fs::read_to_string(&metadata_path).await?;
            serde_json::from_str(&contents)?
        } else {
            CacheMetadata {
                entries: HashMap::new(),
            }
        };

        Ok(Self {
            cache_dir: cache_dir.to_path_buf(),
            metadata,
        })
    }

    /// Check if a URL is cached
    #[allow(dead_code)]
    pub fn is_cached(&self, url: &str) -> bool {
        self.metadata.entries.contains_key(url)
    }

    /// Get the cached HTML content for a URL
    ///
    /// # Errors
    ///
    /// Returns an error if the cached file cannot be read
    pub async fn get(&self, url: &str) -> Result<Option<String>> {
        if let Some(entry) = self.metadata.entries.get(url) {
            let file_path = self.cache_dir.join(&entry.file_path);
            if file_path.exists() {
                let contents = fs::read_to_string(&file_path).await?;
                return Ok(Some(contents));
            }
        }
        Ok(None)
    }

    /// Store HTML content in the cache
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written or metadata cannot be saved
    pub async fn store(&mut self, url: &str, html: &str) -> Result<()> {
        // Generate a safe filename from the URL
        let filename = Self::url_to_filename(url);
        let file_path = self.cache_dir.join(&filename);

        // Write the HTML content
        fs::write(&file_path, html).await?;

        // Get current timestamp
        let fetched_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| CollectorError::CacheError(format!("Failed to get timestamp: {e}")))?
            .as_secs();

        // Update metadata
        let entry = CacheEntry {
            file_path: filename,
            fetched_at,
            url: url.to_string(),
        };
        self.metadata.entries.insert(url.to_string(), entry);

        // Save metadata
        self.save_metadata().await?;

        Ok(())
    }

    /// Clear all cached data
    ///
    /// # Errors
    ///
    /// Returns an error if files cannot be deleted
    #[allow(dead_code)]
    pub async fn clear(&mut self) -> Result<()> {
        // Delete all cached files
        for entry in self.metadata.entries.values() {
            let file_path = self.cache_dir.join(&entry.file_path);
            if file_path.exists() {
                fs::remove_file(&file_path).await?;
            }
        }

        // Clear metadata
        self.metadata.entries.clear();
        self.save_metadata().await?;

        Ok(())
    }

    /// Convert a URL to a safe filename
    fn url_to_filename(url: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Remove protocol and replace unsafe characters
        let safe = url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .replace(['/', '?', '=', '&', '+'], "_");

        // Append hash to ensure uniqueness when different URLs produce the same safe string
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        let hash = hasher.finish();

        format!("{safe}_{hash:016x}.html")
    }

    /// Save metadata to disk atomically
    async fn save_metadata(&self) -> Result<()> {
        let metadata_path = self.cache_dir.join("metadata.json");
        let temp_path = self.cache_dir.join("metadata.json.tmp");
        let contents = serde_json::to_string_pretty(&self.metadata)?;
        fs::write(&temp_path, contents).await?;
        fs::rename(&temp_path, &metadata_path).await?;
        Ok(())
    }

    /// Get the cache directory path
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}
