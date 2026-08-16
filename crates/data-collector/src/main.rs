//! OPR Army Data Collector
//!
//! This tool fetches and parses army data from the One Page Rules Army Forge
//! website and outputs them as versioned YAML files.

mod cache;
mod error;
mod fetch;
mod http_client;
mod parser;

use cache::Cache;
use clap::{Parser, Subcommand};
use error::Result;
use fetch::Fetcher;
use http_client::HttpClient;
use parser::{Army, parse_army_list, parse_preview_page};
use std::path::{Path, PathBuf};

/// Command-line interface for the OPR data collector
#[derive(Parser, Debug)]
#[command(name = "data-collector")]
#[command(about = "Fetches and parses OPR army data from Army Forge")]
struct Cli {
    /// Subcommand to run (defaults to full-run if not specified)
    #[command(subcommand)]
    command: Option<Command>,

    /// Force refresh of cached data (re-fetch from network)
    #[arg(long, default_value_t = false, global = true)]
    force_refresh: bool,

    /// Output directory for YAML files
    #[arg(short, long, default_value = "data", global = true)]
    output_dir: String,

    /// Cache directory for raw HTML files
    #[arg(long, default_value = "data/cache", global = true)]
    cache_dir: String,

    /// Increase verbosity
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,
}

/// Available subcommands
#[derive(Subcommand, Debug)]
enum Command {
    /// Fetch data from Army Forge (network only, no parsing)
    Fetch,
    /// Parse cached data and generate YAML (no network)
    Parse,
    /// Fetch and parse in one go (default behavior)
    Run,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("OPR Data Collector");
    println!("Output directory: {}", cli.output_dir);
    println!("Cache directory: {}", cli.cache_dir);
    println!("Force refresh: {}", cli.force_refresh);
    println!("Verbosity: {}", cli.verbose);
    println!();

    // Determine which mode to run
    let mode = cli.command.as_ref().unwrap_or(&Command::Run);

    match mode {
        Command::Fetch => {
            run_fetch_phase(&cli).await?;
        }
        Command::Parse => {
            run_parse_phase(&cli).await?;
        }
        Command::Run => {
            run_fetch_phase(&cli).await?;
            println!();
            run_parse_phase(&cli).await?;
        }
    }

    Ok(())
}

/// Execute the fetch phase: download data from Army Forge
async fn run_fetch_phase(cli: &Cli) -> Result<()> {
    // Initialize cache
    let cache_path = Path::new(&cli.cache_dir);
    let mut cache = Cache::new(cache_path).await?;
    println!("Cache initialized at: {}", cli.cache_dir);

    // Initialize HTTP client with 3 second delay between requests
    let http_client = HttpClient::new(3000, cli.verbose > 0)?;
    println!("HTTP client ready with rate limiting (3s delay)");

    // Execute fetch phase
    let mut fetcher = Fetcher::new(&http_client, &mut cache, cli.force_refresh, cli.verbose > 0);
    fetcher.fetch_all().await?;

    Ok(())
}

/// Find and add subfaction armies from cached preview pages
async fn add_subfactions_from_cache(
    armies: &mut Vec<Army>,
    cache: &Cache,
    subfaction_names: &[&str],
    verbose: bool,
) -> Result<()> {
    for subfaction_name in subfaction_names {
        // Check if we already have this army
        if !armies.iter().any(|a| a.name == *subfaction_name) {
            // Try to find the cached preview page
            let cache_dir = cache.cache_dir();
            let preview_files =
                std::fs::read_dir(cache_dir).map_err(error::CollectorError::IoError)?;

            for entry in preview_files {
                let entry = entry.map_err(error::CollectorError::IoError)?;
                let path = entry.path();

                if path.extension().is_some_and(|e| e == "html") {
                    let filename = path
                        .file_name()
                        .ok_or_else(|| {
                            error::CollectorError::ParseError("Invalid filename".to_string())
                        })?
                        .to_string_lossy();

                    // Check if this is a preview page
                    if filename.contains("preview") {
                        // Extract army ID from filename
                        if let Some(army_id) = extract_army_id_from_filename(&filename) {
                            // Try to parse this page
                            if let Ok(parsed_army) = parse_preview_page(cache, &army_id).await
                                && parsed_army.name == *subfaction_name
                            {
                                armies.push(parsed_army);
                                if verbose {
                                    println!("  ✓ Found subfaction: {subfaction_name}");
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Execute the parse phase: parse cached data and generate YAML files
async fn run_parse_phase(cli: &Cli) -> Result<()> {
    // Initialize cache
    let cache_path = Path::new(&cli.cache_dir);
    let cache = Cache::new(cache_path).await?;

    // Parse army list
    let mut armies = parse_army_list(&cache).await?;
    println!("\nParsed {} armies from army list", armies.len());

    // Also parse subfaction pages that were fetched
    let subfaction_names = [
        "Battle Brothers",
        "Blood Brothers",
        "Dark Brothers",
        "Knight Brothers",
        "Watch Brothers",
        "Wolf Brothers",
        "Prime Brothers",
        "Blood Prime Brothers",
        "Dark Prime Brothers",
        "Knight Prime Brothers",
        "Watch Prime Brothers",
        "Wolf Prime Brothers",
        "Havoc Brothers",
        "Change Disciples",
        "Lust Disciples",
        "Plague Disciples",
        "War Disciples",
        "Elven Jesters",
        "Titan Lords",
        "Titan Lords Change Disciples",
        "Titan Lords Lust Disciples",
        "Titan Lords Plague Disciples",
        "Titan Lords War Disciples",
    ];

    add_subfactions_from_cache(&mut armies, &cache, &subfaction_names, cli.verbose > 0).await?;
    println!("Total armies to parse: {}", armies.len());

    // Parse and generate YAML for each army
    let mut success_count: usize = 0;
    let mut error_count: usize = 0;
    
    for army in &armies {
        if cli.verbose > 0 {
            println!("\nParsing units for {}...", army.name);
        }
        
        match parse_preview_page(&cache, &army.id).await {
            Ok(parsed_army) => {
                if cli.verbose > 0 {
                    println!(
                        "✓ Parsed {} units from {} (version: {:?})",
                        parsed_army.units.len(),
                        parsed_army.name,
                        parsed_army.version
                    );
                }
                
                // Generate YAML file
                match generate_yaml_file(&parsed_army, &cli.output_dir) {
                    Ok(path) => {
                        if cli.verbose > 0 {
                            println!("✓ Generated YAML: {}", path.display());
                        }
                        success_count = success_count.saturating_add(1);
                    }
                    Err(e) => {
                        println!("✗ Failed to generate YAML for {}: {e}", army.name);
                        error_count = error_count.saturating_add(1);
                    }
                }
            }
            Err(e) => {
                println!("✗ Failed to parse units for {}: {e}", army.name);
                error_count = error_count.saturating_add(1);
            }
        }
    }

    println!("\nParse phase complete:");
    println!("  ✓ Successfully generated: {success_count} armies");
    if error_count > 0 {
        println!("  ✗ Failed: {error_count} armies");
    }

    Ok(())
}

/// Extract army ID from a cached filename
/// Filename format: army-forge.onepagerules.com_armyInfo_{id}_`2_preview.html`
fn extract_army_id_from_filename(filename: &str) -> Option<String> {
    // Look for the pattern: armyInfo_{id}_2_preview
    // The {id} can contain underscores, so we need to find the boundaries
    let army_info_prefix = "armyInfo_";
    let preview_suffix = "_2_preview";

    // Find where "armyInfo_" starts
    let prefix_pos = filename.find(army_info_prefix)?;
    let id_start = prefix_pos.saturating_add(army_info_prefix.len());

    // Find where "_2_preview" starts (after the ID)
    let suffix_pos = filename.get(id_start..)?.find(preview_suffix)?;
    let id_end = id_start.saturating_add(suffix_pos);

    // Extract the ID
    filename
        .get(id_start..id_end)
        .map(std::string::ToString::to_string)
}

/// Generate YAML file for an army
fn generate_yaml_file(army: &Army, output_dir: &str) -> Result<PathBuf> {
    use std::collections::BTreeMap;
    use std::fs;

    // Convert army name to kebab-case filename
    let filename = army.name.to_lowercase().replace(' ', "-").replace('\'', "");
    let filepath = PathBuf::from(output_dir).join(format!("{filename}.yaml"));

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;

    // Create versioned YAML structure using a wrapper map
    let version = army.version.as_deref().unwrap_or("unknown");
    let mut versioned_data = BTreeMap::new();
    versioned_data.insert(version.to_string(), army);
    let content = serde_yaml::to_string(&versioned_data)?;

    fs::write(&filepath, content)?;

    Ok(filepath)
}
