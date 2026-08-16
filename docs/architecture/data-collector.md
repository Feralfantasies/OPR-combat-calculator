---
okf_version: "0.2"
---

# Data Collector Module

The `data-collector` crate scrapes unit data (stats, weapons, upgrades) from all Grimdark Future armies on the OPR Army Forge website, caches raw HTML locally, and outputs versioned YAML files for use by the combat simulator.

## Purpose

The combat simulator needs accurate, up-to-date unit data from all Grimdark Future armies. Rather than manually maintaining this data, the data-collector automates the extraction process:

1. **Fetch phase**: Downloads all army pages from Army Forge with rate limiting
2. **Parse phase**: Extracts structured data from cached HTML and writes YAML files
3. **Output**: 40+ YAML files (one per army, including subfactions) with complete unit rosters

The YAML files are consumed by the `api` crate to populate the combat simulator with real unit stats, weapons, and upgrade options.

## Architecture

### Two-Phase Design

The data-collector uses a strict two-phase architecture to separate network I/O from data processing:

```text
┌─────────────┐
│ Fetch Phase │──▶ data/cache/*.html (raw HTML)
└─────────────┘
        │
        ▼
┌─────────────┐
│ Parse Phase │──▶ data/*.yaml (structured data)
└─────────────┘
```

**Why two phases?**

- Parse phase can run offline (zero network requests)
- Cached HTML can be re-parsed without re-fetching
- Easier debugging: inspect raw HTML when parsing fails
- Rate limiting only affects fetch phase

### Fetch Phase

The fetch phase downloads HTML from Army Forge with:

- **Rate limiting**: 3-second delay between requests to avoid being blocked
- **Cache-first**: Skips files that already exist in cache (unless `--force-refresh`)
- **Jina Reader API**: Army Forge is a JavaScript SPA; direct HTTP requests return only the shell. We use `https://r.jina.ai/<url>` to get rendered HTML.
- **Metadata tracking**: `metadata.json` stores timestamps for all fetched files

### Parse Phase

The parse phase reads exclusively from local cache:

- **Zero network requests**: All data comes from `data/cache/`
- **Preview pages**: Main data source (contains unit tables)
- **Army list**: Extracts army names and IDs from the main Grimdark Future page
- **Subfactions**: Hardcoded list of 22 subfaction URLs (see "Gotchas" section)

### Output Format

Each army is written to `data/<army-name>.yaml` with:

```yaml
v3.5.3:  # Army version as top-level key
  name: Alien Hives
  id: w7qor7b2kuifcyvk
  units:
    - name: Hive Lord
      size: 1
      quality: 3+
      defense: 2+
      tough: 12  # Extracted from special rules
      weapons:
        - name: Shredder Cannon
          range: 18"
          attacks: A4
          ap: AP(1)  # Correctly parsed AP value
          special_rules: [Rending]
      special_rules: [Fear(2), Fearless, Hero, Hive Bond]
      cost: 360
      upgrade_categories:
        - category: Replace Shredder Cannon
          upgrades:
            - name: Heavy Razor Claws
              rules: A3, AP(1)
              cost_modifier: +5pts
```

## Usage

### Basic Usage

```bash
# Fetch and parse in one go (default)
cargo run -p data-collector

# Fetch only (network phase)
cargo run -p data-collector -- fetch

# Parse only (offline, uses cache)
cargo run -p data-collector -- parse

# Force re-fetch (ignore cache)
cargo run -p data-collector -- --force-refresh

# Verbose output
cargo run -p data-collector -- -vv
```

### Output

- **Cache**: `data/cache/*.html` (raw HTML from Army Forge)
- **YAML**: `data/*.yaml` (40+ files, one per army)
- **Metadata**: `data/cache/metadata.json` (timestamps)

## Gotchas and Known Issues

These are problems encountered during development that may resurface. Understanding them will save time if they reoccur.

### 1. Army Forge is a JavaScript SPA

**Problem**: Direct HTTP requests to Army Forge URLs return only the JavaScript shell, not the rendered HTML.

**Solution**: Use Jina Reader API (`https://r.jina.ai/<url>`) to fetch server-rendered HTML.

**Code location**: `src/http_client.rs` line ~50

```rust
let jina_url = format!("https://r.jina.ai/{}", url);
```

**If this breaks**: Jina Reader may change their API or rate limits. Alternatives:

- Use a headless browser (Playwright, Puppeteer)
- Ask Army Forge for an API endpoint

### 2. Subfaction Dropdowns Can't Be Scraped

**Problem**: Subfactions (Battle Brothers variants, Prime Brothers variants, etc.) are accessed via dropdown menus in the Army Forge UI. There's no direct URL to list all subfactions for a given army.

**Solution**: Hardcode a list of 22 known subfaction URLs in `src/main.rs`.

**Code location**: `src/main.rs` line ~120

```rust
const SUBFACTION_URLS: &[&str] = &[
    "https://army-forge.onepagerules.com/army-books/grimdark-future/battle-brothers",
    "https://army-forge.onepagerules.com/army-books/grimdark-future/prime-brothers",
    // ... 20 more
];
```

**If this breaks**: New subfactions won't be discovered automatically. Solutions:

- Manually update the hardcoded list when new subfactions are added
- Use a headless browser to click dropdowns and extract URLs
- Ask Army Forge for a subfaction listing API

### 3. Weapon AP Parsing Bug (Fixed)

**Problem**: Weapons like "Shredder Cannon (18\", A4, AP(1))" were being parsed incorrectly. The `AP(1)` value was being captured in the `attacks` field instead of the `ap` field.

**Root cause**: The regex was matching `A` before `AP`, so `AP(1)` matched the attacks pattern first.

**Solution**: Check for `AP` pattern before `A` pattern in `src/parser.rs`.

**Code location**: `src/parser.rs` line ~320

```rust
if stat.starts_with("AP") {
    ap = Some(stat.to_string());
} else if stat.starts_with('A') && stat.get(1..).is_some() {
    attacks = stat.to_string();
}
```

**How to verify**: Run `grep "attacks: AP" data/*.yaml | wc -l` — should return 0.

### 4. Multi-Weapon Upgrade Parsing Bug (Fixed)

**Problem**: Upgrades that replace multiple weapons with nested parentheses were being parsed incorrectly. Example:

```text
Replace Shredder Cannon with:
  - Heavy Razor Claws (A3, AP(1)), CCW (A4)
```

Was being parsed as:

```yaml
rules: A3, AP(1)), CCW (A4
```

**Root cause**: The parser used `rfind(')')` to find the closing parenthesis, but this found the wrong closing paren when there were nested parens like `AP(1)`.

**Solution**: Track parenthesis depth and find the matching closing paren.

**Code location**: `src/parser.rs` line ~450

```rust
let mut paren_depth: i32 = 0;
let mut start = None;
let mut end = None;

for (i, c) in text.char_indices() {
    match c {
        '(' => {
            if paren_depth == 0 {
                start = Some(i);
            }
            paren_depth = paren_depth.saturating_add(1);
        }
        ')' => {
            paren_depth = paren_depth.saturating_sub(1);
            if paren_depth == 0 {
                end = Some(i);
                break;
            }
        }
        _ => {}
    }
}
```

**How to verify**: Check `data/battle-brothers.yaml` for "Heavy Razor Claws, CCW" upgrades — should have clean `rules` field.

### 5. Tough Field Extraction

**Problem**: The `Tough(X)` special rule was being left in the `special_rules` array instead of being extracted into a dedicated `tough` field.

**Solution**: Added `tough` field to `Unit` struct and extraction logic in `parse_special_rules_with_tough`.

**Code location**: `src/parser.rs` line ~180

```rust
pub struct Unit {
    // ...
    pub tough: Option<u32>,  // Extracted from special rules
    // ...
}

fn parse_special_rules_with_tough(rules: &str) -> (Vec<String>, Option<u32>) {
    let mut special_rules = Vec::new();
    let mut tough = None;
    
    for rule in rules.split(',') {
        let rule = rule.trim();
        if rule.starts_with("Tough(") && rule.ends_with(')') {
            if let Some(value) = rule.get(6..rule.len().saturating_sub(1)) {
                if let Ok(n) = value.parse::<u32>() {
                    tough = Some(n);
                }
            }
        } else if !rule.is_empty() {
            special_rules.push(rule.to_string());
        }
    }
    
    (special_rules, tough)
}
```

**How to verify**: Run `grep -l "tough:" data/*.yaml | wc -l` — should match total YAML count.

### 6. Filename Parsing for Army IDs with Underscores

**Problem**: The function `extract_army_id_from_filename` was splitting on underscores and only taking the first part, breaking IDs like `iV_3U33NE1_ZTXBB`.

**Root cause**: Filename format is `army-forge.onepagerules.com_armyInfo_{id}_2_preview.html`, but the code was splitting on all underscores.

**Solution**: Use `find` and `rfind` to extract the ID between `armyInfo_` and `_2_preview`.

**Code location**: `src/main.rs` line ~180

```rust
fn extract_army_id_from_filename(filename: &str) -> Option<String> {
    let start_marker = "armyInfo_";
    let end_marker = "_2_preview";
    
    let start_pos = filename.find(start_marker)?;
    let start_idx = start_pos + start_marker.len();
    
    let end_pos = filename.rfind(end_marker)?;
    
    filename.get(start_idx..end_pos).map(|s| s.to_string())
}
```

**How to verify**: Check that Titan Lords subfactions (e.g., `titan-lords-war-disciples.yaml`) are present.

### 7. Elven Jesters Name Extraction Fallback

**Problem**: Some army preview pages don't have the expected `GF - Army Name vX.Y.Z` header format. Elven Jesters was being parsed as "Unknown Army".

**Solution**: Added fallback to extract name from `# Army Name` markdown headers.

**Code location**: `src/parser.rs` line ~140

```rust
let army_name = html
    .lines()
    .find(|line| line.starts_with("GF - "))
    .and_then(|line| line.get(5..))
    .and_then(|rest| rest.rfind(" v").and_then(|pos| rest.get(..pos)))
    .or_else(|| {
        // Fallback: look for "# Army Name" markdown header
        html.lines()
            .find(|line| line.starts_with("# ") && !line.contains(" v"))
            .and_then(|line| line.get(2..))
    })
    .unwrap_or("Unknown Army");
```

**How to verify**: Check that `data/elven-jesters.yaml` exists and has correct name.

### 8. Rate Limiting

**Problem**: Army Forge has aggressive rate limiting. Without delays, requests get blocked after ~10 requests.

**Solution**: 3-second delay between requests in `HttpClient`.

**Code location**: `src/http_client.rs` line ~80

```rust
impl HttpClient {
    pub async fn get(&self, url: &str) -> Result<String> {
        // ... existing code ...
        tokio::time::sleep(Duration::from_millis(self.rate_limit_ms)).await;
        // ... make request ...
    }
}
```

**If this breaks**: Increase the delay to 5 or 10 seconds. Check `metadata.json` for 429 errors.

## Testing and Validation

### Automated Checks

The following checks are run by the auditor to verify correctness:

1. **Clippy**: `cargo clippy --all-targets --all-features -p data-collector` — zero warnings
2. **YAML count**: `ls data/*.yaml | wc -l` — should be 40+
3. **Tough field**: `grep -l "tough:" data/*.yaml | wc -l` — should match YAML count
4. **AP parsing**: `grep "attacks: AP" data/*.yaml | wc -l` — should be 0
5. **Subfactions**: Check for `battle-brothers.yaml`, `prime-brothers.yaml`, `havoc-brothers.yaml` and their variants

### Manual Validation

To manually validate the data-collector:

1. **Fetch a specific army**:

   ```bash
   cargo run -p data-collector -- fetch
   ls data/cache/ | grep alien
   ```

2. **Parse from cache**:

   ```bash
   cargo run -p data-collector -- parse
   cat data/alien-hives.yaml | head -30
   ```

3. **Check for parsing errors**:

   ```bash
   grep -r "Unknown Army" data/*.yaml  # Should return nothing
   grep -r "units: []" data/*.yaml    # Should return nothing
   ```

## Future Improvements

These are known limitations that could be addressed in future work:

1. **Automatic subfaction discovery**: Use a headless browser to click dropdowns and extract all subfaction URLs instead of hardcoding.

2. **Cache invalidation**: Add `--cache-max-age` flag to automatically re-fetch stale data.

3. **Error recovery**: Add retry logic for failed requests (currently just skips).

4. **Progress reporting**: Add a progress bar for long fetch operations.

5. **Validation**: Add schema validation for generated YAML files.

## See Also

- [API Module](api.md) — The combat simulator that consumes these YAML files
- [Modular Monolith](modular-monolith.md) — Workspace layout and crate boundaries
- [Alien Hives](../armies/alien-hives.md) — Example army documentation
