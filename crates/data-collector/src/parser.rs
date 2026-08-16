//! Parser module for extracting army data from cached HTML

use crate::cache::Cache;
use crate::error::{CollectorError, Result};

/// Represents an army with its metadata
#[derive(Debug, Clone, serde::Serialize)]
pub struct Army {
    /// Army name (e.g., "Alien Hives")
    pub name: String,
    /// Army ID from the URL
    pub id: String,
    /// Version number (e.g., "v3.5.3")
    #[serde(skip_serializing)]
    pub version: Option<String>,
    /// URL to the army info page
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub info_url: String,
    /// URL to the preview page
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub preview_url: String,
    /// Whether this is a subfaction parent (has dropdown)
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub has_subfactions: bool,
    /// List of units in this army
    pub units: Vec<Unit>,
}

/// Represents a subfaction option
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Subfaction {
    /// Subfaction name
    pub name: String,
    /// Parent army name
    pub parent: String,
    /// URL to the subfaction page
    pub url: String,
}

/// Represents a weapon with its stats
#[derive(Debug, Clone, serde::Serialize)]
pub struct Weapon {
    /// Weapon name
    pub name: String,
    /// Range (e.g., "18\"") or empty for melee
    pub range: Option<String>,
    /// Attacks (e.g., "A4")
    pub attacks: String,
    /// Armor piercing value (e.g., "AP(1)") or empty
    pub ap: Option<String>,
    /// Special rules for this weapon
    pub special_rules: Vec<String>,
}

/// Represents an upgrade option for a unit
#[derive(Debug, Clone, serde::Serialize)]
pub struct Upgrade {
    /// Upgrade name
    pub name: String,
    /// Rules or stats in parentheses
    pub rules: Option<String>,
    /// Cost modifier (e.g., "+5pts", "Free")
    pub cost_modifier: String,
}

/// Represents a category of upgrades for a unit
#[derive(Debug, Clone, serde::Serialize)]
pub struct UpgradeCategory {
    /// Category name (e.g., "Upgrade with one", "Replace any Heavy Razor Claw")
    pub category: String,
    /// List of upgrades in this category
    pub upgrades: Vec<Upgrade>,
}

/// Represents a unit with its stats
#[derive(Debug, Clone, serde::Serialize)]
pub struct Unit {
    /// Unit name
    pub name: String,
    /// Unit size (number of models)
    pub size: u32,
    /// Quality rating (e.g., "3+")
    pub quality: String,
    /// Defense rating (e.g., "2+")
    pub defense: String,
    /// Tough value (if present in special rules, e.g., "Tough(3)")
    pub tough: Option<u32>,
    /// List of weapons
    pub weapons: Vec<Weapon>,
    /// Special rules for the unit (excluding Tough)
    pub special_rules: Vec<String>,
    /// Point cost
    pub cost: u32,
    /// Upgrade categories available for this unit
    pub upgrade_categories: Vec<UpgradeCategory>,
}

/// Parse a preview page to extract unit data
///
/// # Errors
///
/// Returns an error if the cached file cannot be read or parsed
pub fn parse_preview_page(cache: &Cache, army_id: &str) -> Result<Army> {
    let cache_dir = cache.cache_dir();
    let preview_path = cache_dir.join(format!(
        "army-forge.onepagerules.com_armyInfo_{army_id}_2_preview.html"
    ));

    if !preview_path.exists() {
        return Err(CollectorError::ParseError(format!(
            "Preview page not found in cache: {}",
            preview_path.display()
        )));
    }

    let html = std::fs::read_to_string(&preview_path)?;
    let version = extract_version_from_preview(&html);
    let units = extract_units_from_table(&html)?;

    // Extract army name from the line "GF - Army Name vX.Y.Z" or "# Army Name"
    let army_name = html
        .lines()
        .find(|line| line.starts_with("GF - "))
        .and_then(|line| line.get(5..)) // Skip "GF - "
        .and_then(|rest| rest.rfind(" v").and_then(|pos| rest.get(..pos))) // Get everything before " v"
        .or_else(|| {
            // Fallback: look for "# Army Name" format
            html.lines()
                .find(|line| line.starts_with("# ") && !line.contains("v3."))
                .and_then(|line| line.get(2..)) // Skip "# "
        })
        .unwrap_or("Unknown Army");

    let army = Army {
        name: army_name.to_string(),
        id: army_id.to_string(),
        version,
        info_url: String::new(),
        preview_url: String::new(),
        has_subfactions: false,
        units,
    };

    Ok(army)
}

/// Extract version from preview page HTML
fn extract_version_from_preview(html: &str) -> Option<String> {
    // Look for pattern like "GF - Army Name vX.Y.Z" in the first few lines
    for line in html.lines().take(50) {
        if let Some(version_start) = line.find(" v") {
            let rest = line.get(version_start.checked_add(1)?..)?;
            // Extract version string (vX.Y.Z format)
            let version_end = rest
                .find(' ')
                .or_else(|| rest.find('\n'))
                .unwrap_or(rest.len());
            let version = rest.get(..version_end)?;
            if version.starts_with('v') && version.contains('.') {
                return Some(version.to_string());
            }
        }
    }
    None
}

/// Extract units from the markdown table in preview page HTML
fn extract_units_from_table(html: &str) -> Result<Vec<Unit>> {
    // First pass: extract basic unit data from the summary table
    let mut in_table = false;
    let mut basic_units: Vec<Unit> = Vec::new();

    for line in html.lines() {
        // Check if this is the table header
        if line.contains("| Name [Size] |") && line.contains("| Qua |") {
            in_table = true;
            continue;
        }

        // Skip separator row
        if in_table && line.starts_with("| ---") {
            continue;
        }

        // Parse table rows
        if in_table && line.starts_with('|') {
            if let Some(unit) = parse_unit_row(line)? {
                basic_units.push(unit);
            }
        } else if in_table && !line.starts_with('|') {
            // End of table
            break;
        }
    }

    // Second pass: extract detailed unit data including upgrades
    let mut current_unit_index = None;
    let mut upgrade_categories: Vec<UpgradeCategory> = Vec::new();

    for line in html.lines() {
        let line = line.trim();

        // Check if this is a unit detail header: **Unit Name [Size]**- Costpts
        if line.starts_with("**") && line.contains("**-") {
            // Save previous unit's upgrades if we had one
            if let Some(idx) = current_unit_index
                && let Some(unit) = basic_units.get_mut(idx)
            {
                let unit: &mut Unit = unit;
                let categories: Vec<UpgradeCategory> = std::mem::take(&mut upgrade_categories);
                unit.upgrade_categories = categories;
            }

            // Find which unit this is
            if let Some(unit_name) = extract_unit_name_from_header(line) {
                current_unit_index = basic_units.iter().position(|u| u.name == unit_name);
                upgrade_categories = Vec::new();
            }
            continue;
        }

        // Check if this is an upgrade category header
        if current_unit_index.is_some() && is_upgrade_category(line) {
            upgrade_categories.push(UpgradeCategory {
                category: line.to_string(),
                upgrades: Vec::new(),
            });
            continue;
        }

        // Check if this is an upgrade option
        if current_unit_index.is_some()
            && !upgrade_categories.is_empty()
            && let Some(upgrade) = parse_upgrade_option(line)
            && let Some(last_category) = upgrade_categories.last_mut()
        {
            last_category.upgrades.push(upgrade);
        }
    }

    // Save the last unit's upgrades
    if let Some(idx) = current_unit_index
        && let Some(unit) = basic_units.get_mut(idx)
    {
        unit.upgrade_categories = upgrade_categories;
    }

    Ok(basic_units)
}

/// Extract unit name from a detail header like "**Hive Lord [1]**- 360pts"
fn extract_unit_name_from_header(line: &str) -> Option<String> {
    let start = line.find("**")?;
    let rest = line.get(start.checked_add(2)?..)?;
    let end = rest.find("**")?;
    let name_with_size = rest.get(..end)?;

    // Remove size if present: "Hive Lord [1]" -> "Hive Lord"
    if let Some(bracket) = name_with_size.rfind('[') {
        Some(name_with_size.get(..bracket)?.trim().to_string())
    } else {
        Some(name_with_size.trim().to_string())
    }
}

/// Check if a line is an upgrade category header
fn is_upgrade_category(line: &str) -> bool {
    // Upgrade categories are typically short lines that don't start with special characters
    // and contain keywords like "Upgrade", "Replace", "Add"
    if line.is_empty() || line.starts_with('|') || line.starts_with("**") {
        return false;
    }

    let keywords = ["Upgrade with", "Replace", "Add"];
    keywords.iter().any(|k| line.contains(k))
}

/// Parse an upgrade option line like "Combat Bio-Engineer (Furious)+5pts"
fn parse_upgrade_option(line: &str) -> Option<Upgrade> {
    // Find the cost modifier at the end (e.g., "+5pts" or "Free")
    let (main_part, cost_modifier) = if line.ends_with("pts") {
        let pts_pos = line.rfind("pts")?;
        let cost_start = line.get(..pts_pos)?.rfind(['+', '-'])?;
        let main = line.get(..cost_start)?;
        let cost = line.get(cost_start..)?;
        (main, cost.to_string())
    } else if line.ends_with("Free") {
        let main = line.get(..line.len().checked_sub(4)?)?;
        (main, "Free".to_string())
    } else {
        return None;
    };

    // Handle multiple weapons: "Weapon1 (rules1), Weapon2 (rules2)"
    let mut all_names = Vec::new();
    let mut all_rules = Vec::new();

    // Split by ", " to separate weapons, but be careful about commas inside parentheses
    let mut current_weapon = String::new();
    let mut paren_depth: i32 = 0;

    for ch in main_part.chars() {
        if ch == '(' {
            paren_depth = paren_depth.saturating_add(1);
            current_weapon.push(ch);
        } else if ch == ')' {
            paren_depth = paren_depth.saturating_sub(1);
            current_weapon.push(ch);
        } else if ch == ',' && paren_depth == 0 {
            // Found a separator between weapons
            if !current_weapon.trim().is_empty() {
                parse_single_weapon(&current_weapon, &mut all_names, &mut all_rules);
            }
            current_weapon.clear();
        } else {
            current_weapon.push(ch);
        }
    }

    // Don't forget the last weapon
    if !current_weapon.trim().is_empty() {
        parse_single_weapon(&current_weapon, &mut all_names, &mut all_rules);
    }

    let name = if all_names.is_empty() {
        main_part.trim().to_string()
    } else {
        all_names.join(", ")
    };

    let rules_str = if all_rules.is_empty() {
        None
    } else {
        Some(all_rules.join(", "))
    };

    Some(Upgrade {
        name,
        rules: rules_str,
        cost_modifier,
    })
}

fn parse_single_weapon(weapon_str: &str, names: &mut Vec<String>, rules: &mut Vec<String>) {
    let weapon_str = weapon_str.trim();
    if let Some(paren_start) = weapon_str.find('(') {
        let name = weapon_str.get(..paren_start).unwrap_or("").trim();
        let rest = weapon_str
            .get(paren_start.saturating_add(1)..)
            .unwrap_or("");

        // Find the matching closing parenthesis
        let mut depth: i32 = 1;
        let mut paren_end = None;
        for (i, ch) in rest.char_indices() {
            if ch == '(' {
                depth = depth.saturating_add(1);
            } else if ch == ')' {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    paren_end = Some(i);
                    break;
                }
            }
        }

        if let Some(end) = paren_end {
            let rules_str = rest.get(..end).unwrap_or("");
            if !name.is_empty() {
                names.push(name.to_string());
            }
            if !rules_str.is_empty() {
                rules.push(rules_str.to_string());
            }
        } else {
            // No matching closing paren, treat whole thing as name
            names.push(weapon_str.to_string());
        }
    } else {
        // No parentheses, just a name
        if !weapon_str.is_empty() {
            names.push(weapon_str.to_string());
        }
    }
}

/// Parse a single unit row from the markdown table
fn parse_unit_row(line: &str) -> Result<Option<Unit>> {
    // Split by | and filter empty strings
    let cells: Vec<&str> = line
        .split('|')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    // Expect 6 columns: Name [Size], Qua, Def, Equipment, Special Rules, Cost
    if cells.len() < 6 {
        return Ok(None);
    }

    // Parse name and size from "Name [Size]"
    let name_size = cells
        .first()
        .ok_or_else(|| CollectorError::ParseError("Missing name column".to_string()))?;
    let (name, size) = parse_name_and_size(name_size)?;

    // Parse quality
    let quality = cells
        .get(1)
        .ok_or_else(|| CollectorError::ParseError("Missing quality column".to_string()))?
        .to_string();

    // Parse defense
    let defense = cells
        .get(2)
        .ok_or_else(|| CollectorError::ParseError("Missing defense column".to_string()))?
        .to_string();

    // Parse equipment
    let equipment_str = cells
        .get(3)
        .ok_or_else(|| CollectorError::ParseError("Missing equipment column".to_string()))?;
    let weapons = parse_equipment(equipment_str)?;

    // Parse special rules
    let rules_str = cells
        .get(4)
        .ok_or_else(|| CollectorError::ParseError("Missing special rules column".to_string()))?;
    let (special_rules, tough) = parse_special_rules_with_tough(rules_str);

    // Parse cost
    let cost_str = cells
        .get(5)
        .ok_or_else(|| CollectorError::ParseError("Missing cost column".to_string()))?;
    let cost = parse_cost(cost_str)?;

    Ok(Some(Unit {
        name,
        size,
        quality,
        defense,
        tough,
        weapons,
        special_rules,
        cost,
        upgrade_categories: Vec::new(), // Will be populated later
    }))
}

/// Parse name and size from "Name [Size]" format
fn parse_name_and_size(text: &str) -> Result<(String, u32)> {
    // Find the last [ to get size
    if let Some(bracket_start) = text.rfind('[') {
        let name = text
            .get(..bracket_start)
            .ok_or_else(|| CollectorError::ParseError("Failed to parse name".to_string()))?
            .trim()
            .to_string();

        let rest = text
            .get(
                bracket_start.checked_add(1).ok_or_else(|| {
                    CollectorError::ParseError("Failed to parse size".to_string())
                })?..,
            )
            .ok_or_else(|| {
                CollectorError::ParseError("Failed to extract size substring".to_string())
            })?;

        if let Some(bracket_end) = rest.find(']') {
            let size_str = rest
                .get(..bracket_end)
                .ok_or_else(|| CollectorError::ParseError("Failed to extract size".to_string()))?;

            let size = size_str
                .parse::<u32>()
                .map_err(|_| CollectorError::ParseError(format!("Invalid size: {size_str}")))?;

            Ok((name, size))
        } else {
            Err(CollectorError::ParseError(
                "Missing closing bracket".to_string(),
            ))
        }
    } else {
        // No size specified, default to 1
        Ok((text.trim().to_string(), 1))
    }
}

/// Parse equipment string into list of weapons
fn parse_equipment(text: &str) -> Result<Vec<Weapon>> {
    let mut weapons = Vec::new();

    // Equipment format: "Nx Weapon Name (stats) Nx Another Weapon (stats)"
    // Split by looking for patterns like "1x ", "2x ", etc.
    let mut current = String::new();

    for word in text.split_whitespace() {
        // Check if this looks like a count (e.g., "1x", "2x", "10x")
        if word.ends_with('x') && word.get(..word.len().saturating_sub(1)).is_some() {
            // If we have accumulated text, parse it as a weapon
            if !current.is_empty()
                && let Some(weapon) = parse_weapon(&current)?
            {
                weapons.push(weapon);
            }
            current = word.to_string();
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
    }

    // Parse the last weapon
    if !current.is_empty()
        && let Some(weapon) = parse_weapon(&current)?
    {
        weapons.push(weapon);
    }

    Ok(weapons)
}

/// Parse a single weapon from text like "1x Shredder Cannon (18\", A4, Rending)"
fn parse_weapon(text: &str) -> Result<Option<Weapon>> {
    // Extract count prefix (e.g., "1x")
    let (count, rest) = if let Some(x_pos) = text.find('x') {
        let count_str = text.get(..x_pos).ok_or_else(|| {
            CollectorError::ParseError("Failed to parse weapon count".to_string())
        })?;
        if count_str.chars().all(|c| c.is_ascii_digit()) {
            let rest = text
                .get(
                    x_pos.checked_add(1).ok_or_else(|| {
                        CollectorError::ParseError("Failed to parse weapon".to_string())
                    })?..,
                )
                .ok_or_else(|| {
                    CollectorError::ParseError("Failed to extract weapon substring".to_string())
                })?;
            (count_str.to_string(), rest.trim())
        } else {
            ("1".to_string(), text)
        }
    } else {
        ("1".to_string(), text)
    };

    // Find stats in parentheses
    if let Some(paren_start) = rest.find('(') {
        let name = rest
            .get(..paren_start)
            .ok_or_else(|| CollectorError::ParseError("Failed to parse weapon name".to_string()))?
            .trim()
            .to_string();

        let stats_part = rest
            .get(
                paren_start.checked_add(1).ok_or_else(|| {
                    CollectorError::ParseError("Failed to parse weapon stats".to_string())
                })?..,
            )
            .ok_or_else(|| {
                CollectorError::ParseError("Failed to extract weapon stats substring".to_string())
            })?;

        if let Some(paren_end) = stats_part.rfind(')') {
            let stats = stats_part.get(..paren_end).ok_or_else(|| {
                CollectorError::ParseError("Failed to extract weapon stats".to_string())
            })?;

            // Parse stats: range, attacks, AP, special rules
            let stat_parts: Vec<&str> = stats.split(',').map(str::trim).collect();

            let mut range = None;
            let mut attacks = String::new();
            let mut ap = None;
            let mut special_rules = Vec::new();

            for stat in stat_parts {
                if stat.contains('"') {
                    // Range like "18\""
                    range = Some(stat.to_string());
                } else if stat.starts_with("AP") {
                    // AP like "AP(1)" - check BEFORE attacks to avoid "AP(1)" matching as attacks
                    ap = Some(stat.to_string());
                } else if stat.starts_with('A') && stat.get(1..).is_some() {
                    // Attacks like "A4"
                    attacks = stat.to_string();
                } else if !stat.is_empty() {
                    // Special rule
                    special_rules.push(stat.to_string());
                }
            }

            // Default attacks if not specified
            if attacks.is_empty() {
                attacks = format!("A{count}");
            }

            return Ok(Some(Weapon {
                name,
                range,
                attacks,
                ap,
                special_rules,
            }));
        }
    }

    // No stats, just weapon name
    Ok(Some(Weapon {
        name: rest.trim().to_string(),
        range: None,
        attacks: format!("A{count}"),
        ap: None,
        special_rules: Vec::new(),
    }))
}

/// Parse special rules and extract tough value
fn parse_special_rules_with_tough(text: &str) -> (Vec<String>, Option<u32>) {
    let mut special_rules = Vec::new();
    let mut tough = None;

    for rule in text.split(',') {
        let rule = rule.trim();
        if rule.is_empty() {
            continue;
        }

        // Check if this is a Tough rule
        if rule.starts_with("Tough(") && rule.ends_with(')') {
            if let Some(tough_value) = rule.get(6..rule.len().saturating_sub(1))
                && let Ok(tough_val) = tough_value.parse::<u32>()
            {
                tough = Some(tough_val);
            }
        } else {
            special_rules.push(rule.to_string());
        }
    }

    (special_rules, tough)
}

/// Parse cost from string like "360pts"
fn parse_cost(text: &str) -> Result<u32> {
    let cost_str = text.trim().trim_end_matches("pts");
    cost_str
        .parse::<u32>()
        .map_err(|_| CollectorError::ParseError(format!("Invalid cost: {cost_str}")))
}

/// Parse the army list from cached HTML
///
/// # Errors
///
/// Returns an error if the cached file cannot be read or parsed
pub fn parse_army_list(cache: &Cache) -> Result<Vec<Army>> {
    let cache_dir = cache.cache_dir();
    let army_list_path =
        cache_dir.join("army-forge.onepagerules.com_army-books_grimdark-future.html");

    if !army_list_path.exists() {
        return Err(CollectorError::ParseError(
            "Army list cache file not found. Run fetch phase first.".to_string(),
        ));
    }

    let html = std::fs::read_to_string(&army_list_path)?;
    let armies = extract_armies_from_html(&html);

    println!("Parsed {} armies from cache", armies.len());
    Ok(armies)
}

/// Extract army data from HTML content
fn extract_armies_from_html(html: &str) -> Vec<Army> {
    let mut armies = Vec::new();

    // Parse markdown links: [Name](URL)
    for line in html.lines() {
        if line.starts_with('[')
            && line.contains("](https://army-forge.onepagerules.com/army-info/")
            && let Some((name, url)) = parse_markdown_link(line)
            && let Some(id) = extract_army_id_from_url(&url)
        {
            let preview_url =
                format!("https://army-forge.onepagerules.com/armyInfo/{id}/2/preview");

            armies.push(Army {
                name,
                id,
                version: None,
                info_url: url,
                preview_url,
                has_subfactions: false,
                units: Vec::new(),
            });
        }
    }

    armies
}

/// Parse a markdown link: [Name](URL)
fn parse_markdown_link(line: &str) -> Option<(String, String)> {
    let bracket_start = line.find('[')?;
    let rest = line.get(bracket_start.checked_add(1)?..)?;
    let bracket_end = rest.find(']')?;
    let name = rest.get(..bracket_end)?;
    let rest = rest.get(bracket_end.checked_add(1)?..)?;
    let paren_start = rest.find('(')?;
    let rest = rest.get(paren_start.checked_add(1)?..)?;
    let paren_end = rest.find(')')?;
    let url = rest.get(..paren_end)?;
    Some((name.to_string(), url.to_string()))
}

/// Extract army ID from URL
/// URL pattern: <https://army-forge.onepagerules.com/army-info/grimdark-future/{id}?armyName={name>}
fn extract_army_id_from_url(url: &str) -> Option<String> {
    let path_start = url.find("/army-info/grimdark-future/")?;
    let rest = url.get(path_start.checked_add(27)?..)?;
    let id_end = rest.find('?')?;
    let army_id = rest.get(..id_end)?;
    Some(army_id.to_string())
}

/// Parse subfaction dropdown options from cached HTML
///
/// Note: This is a placeholder. Subfactions require fetching the dropdown
/// pages which is not yet implemented.
#[allow(dead_code)]
pub fn parse_subfactions(_cache: &Cache, _parent_name: &str) -> Vec<Subfaction> {
    // TODO: Implement subfaction dropdown fetching
    // For now, return empty list
    println!("Warning: Subfaction parsing not yet implemented");
    Vec::new()
}
