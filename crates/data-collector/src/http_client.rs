//! Rate-limited HTTP client for fetching web pages with Jina Reader support

use crate::cache::Cache;
use crate::error::Result;
use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;

/// Base URL for Jina Reader API
const JINA_READER_BASE: &str = "https://r.jina.ai/";

/// Rate-limited HTTP client with Jina Reader support
pub struct HttpClient {
    /// Underlying reqwest client
    client: Client,
    /// Delay between requests in milliseconds
    delay_ms: u64,
    /// Whether to print verbose output
    verbose: bool,
}

impl HttpClient {
    /// Create a new HTTP client with rate limiting
    ///
    /// # Errors
    ///
    /// Returns an error if the client cannot be built
    pub fn new(delay_ms: u64, verbose: bool) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_mins(1)) // Longer timeout for Jina Reader
            .user_agent("OPR-Data-Collector/0.1.0")
            .build()?;

        Ok(Self {
            client,
            delay_ms,
            verbose,
        })
    }

    /// Fetch a URL, using cache if available
    ///
    /// If the URL is cached and `force_refresh` is false, returns the cached content.
    /// Otherwise, fetches from the network with rate limiting.
    /// Uses Jina Reader API to render JavaScript-heavy pages.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or cache operations fail
    pub async fn fetch(&self, url: &str, cache: &mut Cache, force_refresh: bool) -> Result<String> {
        // Check cache first (unless force refresh)
        if !force_refresh && let Some(cached) = cache.get(url).await? {
            if self.verbose {
                println!("  [cached] {url}");
            }
            return Ok(cached);
        }

        // Fetch from network using Jina Reader (required for SPA sites)
        if self.verbose {
            println!("  [fetching] {url}");
        }

        let html = self.fetch_with_jina(url).await?;

        // Store in cache
        cache.store(url, &html).await?;

        // Rate limit delay
        if self.delay_ms > 0 {
            sleep(Duration::from_millis(self.delay_ms)).await;
        }

        Ok(html)
    }

    /// Fetch a URL using Jina Reader API for JavaScript rendering
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails
    async fn fetch_with_jina(&self, url: &str) -> Result<String> {
        let jina_url = format!("{JINA_READER_BASE}{url}");
        if self.verbose {
            println!("  [jina] {jina_url}");
        }

        // Add headers to help Jina Reader wait for JavaScript rendering
        let response = self
            .client
            .get(&jina_url)
            .header("X-Return-Format", "markdown")
            .header("X-With-Links-Summary", "true")
            .header("X-With-Images-Summary", "true")
            .send()
            .await?
            .error_for_status()?;
        let content = response.text().await?;
        Ok(content)
    }



    /// Fetch multiple URLs with rate limiting
    ///
    /// # Errors
    ///
    /// Returns an error if any HTTP request fails or cache operations fail
    #[allow(dead_code)]
    pub async fn fetch_all(
        &self,
        urls: &[String],
        cache: &mut Cache,
        force_refresh: bool,
    ) -> Result<Vec<(String, String)>> {
        let mut results = Vec::new();

        for url in urls {
            let html = self.fetch(url, cache, force_refresh).await?;
            results.push((url.clone(), html));
        }

        Ok(results)
    }
}
