//! Fetch phase implementation for downloading army data from Army Forge

use crate::cache::Cache;
use crate::error::Result;
use crate::http_client::HttpClient;

/// Base URL for Army Forge
const ARMY_FORGE_BASE: &str = "https://army-forge.onepagerules.com";

/// URL for the main army books page
const ARMY_BOOKS_URL: &str = "https://army-forge.onepagerules.com/army-books/grimdark-future";

/// Fetch phase coordinator
pub struct Fetcher<'a> {
    /// HTTP client with rate limiting
    client: &'a HttpClient,
    /// Cache for storing fetched content
    cache: &'a mut Cache,
    /// Whether to force refresh (ignore cache)
    force_refresh: bool,
    /// Whether to print verbose output
    verbose: bool,
}

impl<'a> Fetcher<'a> {
    /// Create a new fetcher
    pub const fn new(
        client: &'a HttpClient,
        cache: &'a mut Cache,
        force_refresh: bool,
        verbose: bool,
    ) -> Self {
        Self {
            client,
            cache,
            force_refresh,
            verbose,
        }
    }

    /// Execute the fetch phase: download all army data
    ///
    /// # Errors
    ///
    /// Returns an error if any fetch operation fails
    pub async fn fetch_all(&mut self) -> Result<()> {
        println!("Starting fetch phase...");
        println!("Force refresh: {}", self.force_refresh);
        println!();

        // Step 1: Fetch the main army list page
        println!("Step 1: Fetching army list page...");
        let army_list_html = self
            .client
            .fetch(ARMY_BOOKS_URL, self.cache, self.force_refresh)
            .await?;
        println!(
            "  ✓ Fetched army list page ({} bytes)",
            army_list_html.len()
        );
        println!();

        // Step 2: Extract army URLs from the army list page
        println!("Step 2: Extracting army URLs...");
        let mut army_urls = Self::extract_army_urls(&army_list_html);
        println!("  ✓ Found {} armies", army_urls.len());
        println!();

        // Step 3: Check for subfactions that need browser rendering
        let subfaction_parents = [
            "Battle Brothers",
            "Prime Brothers",
            "Havoc Brothers",
            "Titan Lords",
            "Wormhole Daemons",
        ];

        let has_subfactions = subfaction_parents.iter().any(|parent| {
            army_list_html.contains(parent) && !army_urls.iter().any(|(name, _)| name == parent)
        });

        if has_subfactions {
            println!("Step 2b: Adding known subfaction URLs...");

            // Known subfaction URLs based on observed HTML structure
            // These are the subfactions that appear in dropdown menus
            let known_subfactions = vec![
                // Battle Brothers subfactions
                ("Battle Brothers".to_string(), "https://army-forge.onepagerules.com/army-info/grimdark-future/78qp9l5alslt6yj8?armyName=Battle+Brothers".to_string()),
                ("Blood Brothers".to_string(), "https://army-forge.onepagerules.com/army-info/grimdark-future/xnnqhh1775kvmz2r?armyName=Blood+Brothers".to_string()),
                ("Dark Brothers".to_string(), "https://army-forge.onepagerules.com/army-info/grimdark-future/xp5zwh73lg1uaym4?armyName=Dark+Brothers".to_string()),
                ("Knight Brothers".to_string(), "https://army-forge.onepagerules.com/army-info/grimdark-future/w70ha3o85pa7nigq?armyName=Knight+Brothers".to_string()),
                ("Watch Brothers".to_string(), "https://army-forge.onepagerules.com/army-info/grimdark-future/rvvb3kdn2x2pqkki?armyName=Watch+Brothers".to_string()),
                ("Wolf Brothers".to_string(), "https://army-forge.onepagerules.com/army-info/grimdark-future/yxjboa8oma9bbdck?armyName=Wolf+Brothers".to_string()),
                // Havoc Brothers (Havoc Disciples) subfactions
                ("Havoc Brothers".to_string(), "https://army-forge.onepagerules.com/army-info/grimdark-future/7o6om21wxlvvy3hq?armyName=Havoc+Brothers".to_string()),
                ("Change Disciples".to_string(), "https://army-forge.onepagerules.com/army-info/grimdark-future/r6hr29338u4micfw?armyName=Change+Disciples".to_string()),
                ("Lust Disciples".to_string(), "https://army-forge.onepagerules.com/army-info/grimdark-future/drqw1iswxmuugp3d?armyName=Lust+Disciples".to_string()),
                ("Plague Disciples".to_string(), "https://army-forge.onepagerules.com/army-info/grimdark-future/jlray7cwf8mvw5sn?armyName=Plague+Disciples".to_string()),
                ("War Disciples".to_string(), "https://army-forge.onepagerules.com/army-info/grimdark-future/31xjrm9ivdimkjxp?armyName=War+Disciples".to_string()),
                // Prime Brothers subfactions
                ("Prime Brothers".to_string(), "https://army-forge.onepagerules.com/army-info/grimdark-future/oqnnu0gk8q6hyyny?armyName=Prime+Brothers".to_string()),
                ("Blood Prime Brothers".to_string(), "https://army-forge.onepagerules.com/army-info/grimdark-future/7ex2x15bpkmy1alv?armyName=Blood+Prime+Brothers".to_string()),
                ("Dark Prime Brothers".to_string(), "https://army-forge.onepagerules.com/army-info/grimdark-future/gk7me4sgn9s740kw?armyName=Dark+Prime+Brothers".to_string()),
                ("Knight Prime Brothers".to_string(), "https://army-forge.onepagerules.com/army-info/grimdark-future/wopr4xvwa51xh3mc?armyName=Knight+Prime+Brothers".to_string()),
                ("Watch Prime Brothers".to_string(), "https://army-forge.onepagerules.com/army-info/grimdark-future/rl7ympklz4r0ls38?armyName=Watch+Prime+Brothers".to_string()),
                ("Wolf Prime Brothers".to_string(), "https://army-forge.onepagerules.com/army-info/grimdark-future/e8mflytiz51kc4n6?armyName=Wolf+Prime+Brothers".to_string()),
                // Titan Lords subfactions
                ("Titan Lords".to_string(), "https://army-forge.onepagerules.com/army-info/grimdark-future/3j10zage1lddt6sr?armyName=Titan+Lords".to_string()),
                ("Titan Lords Change Disciples".to_string(), "https://army-forge.onepagerules.com/army-info/grimdark-future/m0JXdcVTyo-WFrMF?armyName=Titan+Lords+Change+Disciples".to_string()),
                ("Titan Lords Lust Disciples".to_string(), "https://army-forge.onepagerules.com/army-info/grimdark-future/YYIAF_LCwgJiyXYa?armyName=Titan+Lords+Lust+Disciples".to_string()),
                ("Titan Lords Plague Disciples".to_string(), "https://army-forge.onepagerules.com/army-info/grimdark-future/bf20fNmJEyUS-PIX?armyName=Titan+Lords+Plague+Disciples".to_string()),
                ("Titan Lords War Disciples".to_string(), "https://army-forge.onepagerules.com/army-info/grimdark-future/iV_3U33NE1_ZTXBB?armyName=Titan+Lords+War+Disciples".to_string()),

            ];

            // Add known subfactions to the army list (avoiding duplicates)
            let mut added_count = 0;
            for (name, url) in known_subfactions {
                if !army_urls.iter().any(|(n, _)| n == &name) {
                    army_urls.push((name, url));
                    added_count += 1;
                }
            }

            println!("  ✓ Added {added_count} known subfaction URLs");
        }

        println!("  Total armies to fetch: {}", army_urls.len());
        println!();

        // Step 3: Fetch each army's info page and construct preview URLs
        println!("Step 3: Fetching army info pages and constructing preview URLs...");
        let mut preview_urls = Vec::new();
        for (army_name, army_url) in &army_urls {
            if self.verbose {
                println!("  Fetching: {army_name}");
            }
            let _army_html = self
                .client
                .fetch(army_url, self.cache, self.force_refresh)
                .await?;

            // Extract army ID from URL and construct preview URL directly
            // URL pattern: https://army-forge.onepagerules.com/army-info/grimdark-future/{id}?armyName={name}
            if let Some(army_id) = extract_army_id_from_url(army_url) {
                let preview_url = format!("{ARMY_FORGE_BASE}/armyInfo/{army_id}/2/preview");
                preview_urls.push((army_name.clone(), preview_url));
            } else if self.verbose {
                println!("  Warning: Could not extract army ID from URL for {army_name}");
            }
        }
        println!("  ✓ Constructed {} preview URLs", preview_urls.len());
        println!();

        // Step 4: Fetch all preview pages
        println!("Step 4: Fetching preview pages...");
        for (army_name, preview_url) in &preview_urls {
            if self.verbose {
                println!("  Fetching preview: {army_name}");
            }
            let _preview_html = self
                .client
                .fetch(preview_url, self.cache, self.force_refresh)
                .await?;
        }
        println!("  ✓ Fetched {} preview pages", preview_urls.len());
        println!();

        println!("Fetch phase complete!");
        println!("All data cached in: {}", self.cache.cache_dir().display());

        Ok(())
    }

    /// Extract army URLs from the army list page HTML
    fn extract_army_urls(html: &str) -> Vec<(String, String)> {
        let mut armies = Vec::new();

        // Parse the HTML to find army links
        // Army URLs follow the pattern: /army-info/grimdark-future/{id}?armyName={name}
        for line in html.lines() {
            // Look for army-info links
            if line.contains("/army-info/grimdark-future/") {
                // Extract URL and army name
                if let Some(url) = extract_url_from_line(line)
                    && let Some(name) = extract_army_name_from_url(&url)
                {
                    let full_url = format!("{ARMY_FORGE_BASE}{url}");
                    armies.push((name, full_url));
                }
            }
        }

        if armies.is_empty() {
            // Try alternative parsing for markdown format from Jina Reader
            for line in html.lines() {
                if line.starts_with('[')
                    && line.contains("](https://army-forge.onepagerules.com/army-info/")
                    && let Some((name, url)) = parse_markdown_link(line)
                {
                    armies.push((name, url));
                }
            }
        }

        armies
    }


}

/// Extract URL from an HTML line containing an href attribute
fn extract_url_from_line(line: &str) -> Option<String> {
    // Look for href="..." pattern
    if let Some(start) = line.find("href=\"") {
        let (_, rest) = line.split_at(start.saturating_add(6));
        if let Some(end) = rest.find('"') {
            let url = rest.get(..end)?;
            if url.starts_with('/') {
                return Some(url.to_string());
            }
        }
    }
    None
}

/// Extract army name from a URL
fn extract_army_name_from_url(url: &str) -> Option<String> {
    // URL pattern: /army-info/grimdark-future/{id}?armyName={name}
    if let Some(pos) = url.find("armyName=") {
        let (_, rest) = url.split_at(pos.saturating_add(9));
        // URL decode the name
        let name = rest.split('&').next()?;
        let name = name.replace('+', " ");
        // Decode percent encoding
        let name = url_decode(&name);
        return Some(name);
    }
    None
}

/// Parse a markdown link: [Name](URL)
fn parse_markdown_link(line: &str) -> Option<(String, String)> {
    if let Some(bracket_start) = line.find('[') {
        let (_, rest) = line.split_at(bracket_start.saturating_add(1));
        if let Some(bracket_end) = rest.find(']') {
            let name = rest.get(..bracket_end)?;
            let (_, rest) = rest.split_at(bracket_end.saturating_add(1));
            if let Some(paren_start) = rest.find('(') {
                let (_, rest) = rest.split_at(paren_start.saturating_add(1));
                if let Some(paren_end) = rest.find(')') {
                    let url = rest.get(..paren_end)?;
                    return Some((name.to_string(), url.to_string()));
                }
            }
        }
    }
    None
}

/// Simple URL decoding for common percent-encoded characters
fn url_decode(s: &str) -> String {
    s.replace("%20", " ")
        .replace("%27", "'")
        .replace("%26", "&")
}

/// Extract army ID from army info URL
/// URL pattern: <https://army-forge.onepagerules.com/army-info/grimdark-future/{id}?armyName={name>}
fn extract_army_id_from_url(url: &str) -> Option<String> {
    // Find the army-info path segment
    let path_start = url.find("/army-info/grimdark-future/")?;
    let rest = url.get(path_start.checked_add(27)?..)?;

    // The ID ends at the query string
    let id_end = rest.find('?')?;
    let army_id = rest.get(..id_end)?;

    Some(army_id.to_string())
}
