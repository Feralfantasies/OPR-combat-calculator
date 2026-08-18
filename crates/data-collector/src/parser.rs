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
    /// Weapons parsed from the unit's fixed weapon table; used to remap
    /// replacement targets onto actual equipment names (not serialized).
    #[serde(skip_serializing)]
    pub fixed_weapons: Vec<Weapon>,
}

/// Parse a preview page to extract unit data
///
/// # Errors
///
/// Returns an error if the cached file cannot be read or parsed
pub async fn parse_preview_page(cache: &Cache, preview_url: &str) -> Result<Army> {
    let Some(html) = cache.get(preview_url).await? else {
        return Err(CollectorError::ParseError(format!(
            "Preview page not found in cache: {preview_url}"
        )));
    };

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

    // Extract army ID from URL
    let army_id = extract_army_id_from_url(preview_url).unwrap_or_else(|| "unknown".to_string());

    let army = Army {
        name: army_name.to_string(),
        id: army_id,
        version,
        info_url: String::new(),
        preview_url: preview_url.to_string(),
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

    // Second pass: extract detailed unit data including upgrades.
    // A unit's section ends at the next unit header, so upgrade categories
    // can never bleed across two units.
    let mut current_unit_index: Option<usize> = None;
    let mut upgrade_categories: Vec<UpgradeCategory> = Vec::new();
    let mut fixed_weapons: Vec<Weapon> = Vec::new();
    let mut in_fixed_table = false;

    for line in html.lines() {
        let line = line.trim();

        // Check if this is a unit detail header: **Unit Name [Size]**- Costpts
        if line.starts_with("**") && line.contains("**-") {
            if let Some(idx) = current_unit_index
                && let Some(unit) = basic_units.get_mut(idx)
            {
                unit.upgrade_categories = std::mem::take(&mut upgrade_categories);
                unit.fixed_weapons = std::mem::take(&mut fixed_weapons);
            }
            in_fixed_table = false;

            // Find which unit this is
            if let Some(unit_name) = extract_unit_name_from_header(line) {
                current_unit_index = basic_units.iter().position(|u| u.name == unit_name);
            }
            continue;
        }

        let Some(_) = current_unit_index else {
            continue;
        };

        // Fixed weapon table (| Weapon | RNG | ATK | AP | SPE | ...):
        // its rows carry the unit's base equipment and are kept for target
        // remapping. Only the weapon *header* table is captured; other
        // two-column tables (e.g. `| Upgrade | SPE |`) are ignored.
        if line.starts_with('|') {
            if line.starts_with("| ---") {
                continue;
            }
            if in_fixed_table {
                if let Some(weapon) = parse_fixed_weapon_row(line) {
                    fixed_weapons.push(weapon);
                }
                continue;
            }
            if line.starts_with("| Weapon |")
                && line.contains("| RNG |")
                && line.contains("| ATK |")
            {
                in_fixed_table = true;
            }
            continue;
        }
        in_fixed_table = false;

        // Check if this is an upgrade category header
        if is_upgrade_category(line) {
            upgrade_categories.push(UpgradeCategory {
                category: line.to_string(),
                upgrades: Vec::new(),
            });
            continue;
        }

        // Check if this is an upgrade option
        if !upgrade_categories.is_empty()
            && let Some(upgrade) = parse_upgrade_option(line)
            && let Some(last_category) = upgrade_categories.last_mut()
        {
            last_category.upgrades.push(upgrade);
        }
    }

    // Save the final unit's upgrades (its section ends at end-of-input, not
    // at a next header).
    if let Some(idx) = current_unit_index
        && let Some(unit) = basic_units.get_mut(idx)
    {
        unit.upgrade_categories = upgrade_categories;
        unit.fixed_weapons = fixed_weapons;
    }

    // Tidy up per unit: remap `Replace ...` target names onto the unit's
    // actual (fixed) weapon names so replacements resolve against
    // equipment, then drop exact-duplicate options and categories left
    // empty. `fixed_weapons` is left in place (serialized-then-skipped) so
    // callers and tests can inspect the captured table.
    for unit in &mut basic_units {
        let weapon_names: Vec<String> = unit.fixed_weapons.iter().map(|w| w.name.clone()).collect();
        for category in &mut unit.upgrade_categories {
            category.category = remap_replace_targets(&category.category, &weapon_names);
        }
        dedupe_unit_categories(unit);
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

/// Directive prefixes that mark upgrade category headers in source pages.
const CATEGORY_DIRECTIVES: [&str; 6] = [
    "replace ",
    "upgrade ",
    "add ",
    "take ",
    "select ",
    "any model",
];

/// Check if a line is an upgrade category header. Source pages mark
/// categories with a directive prefix ("Replace ...", "Upgrade ...",
/// "Take ...", "Any model may ..."), so every section gets its own
/// category instead of merging into the previous one (e.g.
/// "Any model may replace 2x Suit-Burst" stays separate from
/// "Replace one Suit-Burst").
fn is_upgrade_category(line: &str) -> bool {
    if line.is_empty()
        || line.starts_with('|')
        || line.starts_with("**")
        || line.starts_with("GF")
        || line.starts_with(char::is_numeric)
    {
        return false;
    }
    let lower = line.to_ascii_lowercase();
    CATEGORY_DIRECTIVES.iter().any(|d| lower.starts_with(d))
}

/// Remove options that duplicate an earlier option *within the same*
/// category (byte-identical name + rules + cost - e.g. the source repeating
/// a row) and drop categories left with no options.
///
/// Repeated *category names* (two genuine "Upgrade with one" sections) are
/// kept: they are distinct groups the API addresses by index.
fn dedupe_unit_categories(unit: &mut Unit) {
    for category in &mut unit.upgrade_categories {
        let mut seen: std::collections::HashSet<(String, String, String)> =
            std::collections::HashSet::new();
        category.upgrades.retain(|upgrade| {
            seen.insert((
                upgrade.name.clone(),
                upgrade.rules.clone().unwrap_or_default(),
                upgrade.cost_modifier.clone(),
            ))
        });
    }
    unit.upgrade_categories.retain(|c| !c.upgrades.is_empty());
}

/// Remap the `Replace ...` category name onto the unit's actual weapon
/// names, so replacement targets resolve against equipment (e.g.
/// `Replace any Heavy Hammer` -> `Replace any Heavy Hammers` when the
/// equipment entry is plural).
///
/// Applied per ` and ` part; parts matching no weapon are kept verbatim.
/// `Nx` count prefixes in the category name are preserved.
/// Leading quantifier/count tokens in `Replace ...` category names.
const REPLACE_QUANTIFIERS: [&str; 7] = ["any", "one", "all", "up", "to", "two", "three"];

/// Remap a `Replace ...` category name onto the unit's actual weapon names
/// (e.g. `Replace one CCW` -> `Replace one CCWs`, `Replace any Heavy Hammer`
/// -> `Replace any Heavy Hammers`) after stripping the quantifier and any
/// `Nx` count prefix. Parts that still match no weapon are kept verbatim.
fn remap_replace_targets(category: &str, weapon_names: &[String]) -> String {
    let Some(rest) = category.strip_prefix("Replace ") else {
        return category.to_string();
    };
    let mut any_changed = false;
    let mapped: Vec<String> = rest
        .split(" and ")
        .map(|part| {
            let words: Vec<&str> = part.split_whitespace().collect();
            let core_start = words
                .iter()
                .take_while(|word| {
                    let lower = word.to_ascii_lowercase();
                    REPLACE_QUANTIFIERS.contains(&lower.as_str()) || is_count_token(word)
                })
                .count();
            let prefix: Vec<&str> = words.iter().take(core_start).copied().collect();
            let core: String = words
                .iter()
                .skip(core_start)
                .copied()
                .collect::<Vec<_>>()
                .join(" ");
            if core.is_empty() {
                return part.to_string();
            }
            let mapped_core =
                find_weapon_name(&core, weapon_names).map_or_else(|| core.clone(), String::from);
            if mapped_core != core {
                any_changed = true;
            }
            if prefix.is_empty() {
                mapped_core
            } else {
                format!("{} {}", prefix.join(" "), mapped_core)
            }
        })
        .collect();
    if any_changed {
        format!("Replace {}", mapped.join(" and "))
    } else {
        category.to_string()
    }
}

/// Find the weapon name equal to `target` exactly, or differing only by a
/// single trailing `s` (singular/plural tolerance).
fn find_weapon_name<'a>(target: &str, weapons: &'a [String]) -> Option<&'a str> {
    for name in weapons {
        if name == target {
            return Some(name);
        }
    }
    let plural = format!("{target}s");
    let singular = target.strip_suffix('s').unwrap_or(target);
    weapons
        .iter()
        .find(|&name| *name == plural || *name == singular)
        .map(String::as_str)
}

/// Parse a row of the unit's fixed weapon table, e.g. `| 9x Suit-Bursts |
/// 18" | A1 | - | Rending |`. Returns the weapon for target remapping.
fn parse_fixed_weapon_row(line: &str) -> Option<Weapon> {
    let cells: Vec<&str> = line
        .split('|')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if cells.len() < 2 {
        return None;
    }
    let name_cell = cells.first().copied()?;
    // Strip an optional count prefix, e.g. `9x Suit-Bursts` -> `Suit-Bursts`.
    let name = name_cell
        .split_once(' ')
        .and_then(|(head, tail)| is_count_token(head).then_some(tail))
        .unwrap_or(name_cell)
        .to_string();
    if name.is_empty() || name == "Weapon" {
        return None;
    }

    // Cells after the name: RNG, ATK, AP, SPE. Reuse the stats parser on a
    // synthesized `(rng, atk, ap, spe)` string when possible.
    let mut range: Option<String> = None;
    let mut attacks: Option<u8> = None;
    let mut ap: Option<String> = None;
    let mut special_rules: Vec<String> = Vec::new();

    for cell in cells.iter().skip(1) {
        let cell = *cell;
        if cell == "-" || cell.is_empty() {
            continue;
        }
        if cell.contains('"') {
            range = Some(cell.to_string());
        } else if let Some(digits) = cell.strip_prefix("AP(").and_then(|d| d.strip_suffix(')'))
            && let Ok(value) = digits.parse::<u32>()
        {
            // `AP(n)` form.
            ap = Some(format!("AP({value})"));
        } else if let Some(digits) = cell.strip_prefix('A')
            && let Ok(value) = digits.parse::<u8>()
        {
            attacks = Some(attacks.map_or(value, |c| c.max(value)));
        } else if cell.chars().all(|c| c.is_ascii_digit())
            && let Ok(value) = cell.parse::<u32>()
        {
            // Bare number in the AP column (the pages emit `1` for `AP(1)`).
            ap = Some(format!("AP({value})"));
        } else {
            special_rules.extend(
                cell.split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string),
            );
        }
    }

    let attacks = attacks.unwrap_or(1);
    Some(Weapon {
        name,
        range,
        attacks: format!("A{attacks}"),
        ap,
        special_rules,
    })
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
                // Parameterized rule written as `RuleName(N)`, e.g.
                // `Transport(6)+20pts`: the paren holds the rule's value,
                // not a weapon specification. Without the name the bare
                // number is uninterpretable, so re-attach it.
                let rule = if !name.is_empty() && rules_str.bytes().all(|b| b.is_ascii_digit()) {
                    format!("{name}({rules_str})")
                } else {
                    rules_str.to_string()
                };
                rules.push(rule);
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
    let weapons = parse_equipment(equipment_str);

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
        fixed_weapons: Vec::new(),      // Populated from the detail table
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

/// Whether a whole token is a weapon count prefix like `3x` (digits then a
/// single trailing `x`). Weapon names that merely end in `x` (`Flux`, `Twin`)
/// are *not* count tokens, so names such as `Rapid Flux Carbines` are never
/// split at the interior `x`.
fn is_count_token(token: &str) -> bool {
    let Some(digits) = token.strip_suffix('x') else {
        return false;
    };
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
}

/// Split an equipment cell into `(count, fragment)` entries. A count token
/// (`2x`) starts a new entry; all other tokens append to the current entry,
/// so multi-word weapon names are preserved verbatim.
fn split_equipment(text: &str) -> Vec<(Option<u32>, String)> {
    let mut entries: Vec<(Option<u32>, Vec<&str>)> = Vec::new();

    for token in text.split_whitespace() {
        if is_count_token(token) {
            let count = token
                .strip_suffix('x')
                .and_then(|digits| digits.parse().ok());
            entries.push((count, Vec::new()));
        } else if entries.is_empty() {
            // First token without a count prefix: a single (un-counted) weapon.
            entries.push((None, vec![token]));
        } else if let Some(last) = entries.last_mut() {
            last.1.push(token);
        }
    }

    entries
        .into_iter()
        .filter(|(_, fragments)| !fragments.is_empty())
        .map(|(count, fragments)| (count, fragments.join(" ")))
        .collect()
}

/// Parse equipment string into list of weapons
fn parse_equipment(text: &str) -> Vec<Weapon> {
    let mut weapons = Vec::new();
    for (count, fragment) in split_equipment(text) {
        if let Some(weapon) = parse_weapon(count, &fragment) {
            weapons.push(weapon);
        }
    }
    weapons
}

/// Parse a single weapon from a fragment like `Shredder Cannon (18\", A4, Rending)`.
///
/// `count` is the fragment's count prefix (from an `Nx` marker). It is used
/// as the fallback attack total when the stats carry no explicit `A<number>`.
fn parse_weapon(count: Option<u32>, text: &str) -> Option<Weapon> {
    let fallback = u8::try_from(count.unwrap_or(1).min(u32::from(u8::MAX))).unwrap_or(u8::MAX);

    let (name, stats_part) = match text.split_once('(') {
        Some((name, rest)) => (name.trim(), Some(rest)),
        None => (text.trim(), None),
    };
    if name.is_empty() {
        // Nameless fragment: nothing usable rather than an empty-name weapon.
        return None;
    }

    let (range, attacks, ap, special_rules) = if let Some(stats) = stats_part
        && let Some(inner) = stats.rsplit_once(')').map(|(inner, _)| inner)
    {
        let mut range: Option<String> = None;
        let mut explicit_attacks: Option<u8> = None;
        let mut ap: Option<String> = None;
        let mut special_rules: Vec<String> = Vec::new();

        for stat in inner.split(',').map(str::trim) {
            if stat.is_empty() {
                continue;
            }
            if stat.contains('"') {
                // Range like `18"`
                range = Some(stat.to_string());
            } else if stat.starts_with("AP") {
                // AP like `AP(1)` - checked before attacks so `AP(1)` is not
                // mistaken for an attack count.
                ap = Some(stat.to_string());
            } else if let Some(digits) = stat.strip_prefix('A')
                && let Ok(value) = digits.parse::<u8>()
            {
                explicit_attacks =
                    Some(explicit_attacks.map_or(value, |current| current.max(value)));
            } else {
                // Special rule
                special_rules.push(stat.to_string());
            }
        }

        let attacks = explicit_attacks.unwrap_or(fallback);
        (range, attacks, ap, special_rules)
    } else {
        (None, fallback, None, Vec::new())
    };

    Some(Weapon {
        name: name.to_string(),
        range,
        attacks: format!("A{attacks}"),
        ap,
        special_rules,
    })
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

/// Normalize Wolf Brothers transports: the army book grants Wolfborn to
/// every Wolf Brothers unit, but the source pages omit it on transport
/// carriers (e.g. Wolf Drop Pod). Any unit that provides capacity via a
/// Transport capacity rule or the `Open Sides` transport rule carries
/// Wolfborn, so add it when missing (without duplicating existing rules).
pub fn normalize_wolf_brothers_transports(army: &mut Army) {
    let is_wolf_faction = army
        .name
        .to_ascii_uppercase()
        .strip_prefix("WOLF")
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(' '));
    if !is_wolf_faction {
        return;
    }
    for unit in &mut army.units {
        let is_transport = unit
            .special_rules
            .iter()
            .any(|r| r.starts_with("Transport(") || r == "Open Sides");
        if is_transport && !unit.special_rules.iter().any(|r| r == "Wolfborn") {
            unit.special_rules.push("Wolfborn".to_string());
        }
    }
}

/// Parse the army list from cached HTML
///
/// # Errors
///
/// Returns an error if the cached file cannot be read or parsed
pub async fn parse_army_list(cache: &Cache) -> Result<Vec<Army>> {
    const ARMY_BOOKS_URL: &str = "https://army-forge.onepagerules.com/army-books/grimdark-future";

    let Some(html) = cache.get(ARMY_BOOKS_URL).await? else {
        return Err(CollectorError::ParseError(
            "Army list cache file not found. Run fetch phase first.".to_string(),
        ));
    };
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
    // Try army info URL pattern first: /army-info/grimdark-future/{id}?armyName=...
    if let Some(path_start) = url.find("/army-info/grimdark-future/") {
        let rest = url.get(path_start.checked_add(27)?..)?;
        let id_end = rest.find('?')?;
        let army_id = rest.get(..id_end)?;
        return Some(army_id.to_string());
    }

    // Try preview URL pattern: /armyInfo/{id}/2/preview
    if let Some(path_start) = url.find("/armyInfo/") {
        let rest = url.get(path_start.checked_add(10)?..)?;
        let id_end = rest.find('/')?;
        let army_id = rest.get(..id_end)?;
        return Some(army_id.to_string());
    }

    None
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

#[cfg(test)]
mod tests {
    use super::*;

    fn army(name: &str, units: Vec<Unit>) -> Army {
        Army {
            name: name.to_string(),
            id: "test".to_string(),
            version: Some("v3.5.3".to_string()),
            info_url: String::new(),
            preview_url: String::new(),
            has_subfactions: false,
            units,
        }
    }

    fn unit(name: &str, units_rules: Vec<&str>) -> Unit {
        Unit {
            name: name.to_string(),
            size: 1,
            quality: "4+".to_string(),
            defense: "3+".to_string(),
            tough: None,
            weapons: vec![],
            special_rules: units_rules.into_iter().map(String::from).collect(),
            cost: 100,
            upgrade_categories: Vec::new(),
            fixed_weapons: Vec::new(),
        }
    }

    #[test]
    fn equipment_count_prefix_is_not_split_mid_name() {
        // A `3x` marker starts the entry; `Flux` must not be read as a count.
        let weapons = parse_equipment("3x Rapid Flux Carbines (18\", A4, Surge) 3x CCWs (A2)");
        assert_eq!(weapons.len(), 2);
        assert_eq!(weapons[0].name, "Rapid Flux Carbines");
        assert_eq!(weapons[0].attacks, "A4");
        assert_eq!(weapons[1].name, "CCWs");
        assert_eq!(weapons[1].attacks, "A2");
    }

    #[test]
    fn equipment_single_weapon_without_count_prefix() {
        // Detail-style rows list single weapons without an `1x` prefix; the
        // count prefix still parses off a following entry.
        let weapons = parse_equipment(
            "Heavy Burst Carbine (18\", A2) 1x Twin Heavy Pulse-Gun (18\", A4, AP(1))",
        );
        assert_eq!(weapons.len(), 2);
        assert_eq!(weapons[0].name, "Heavy Burst Carbine");
        assert_eq!(weapons[0].attacks, "A2");
        assert_eq!(weapons[1].name, "Twin Heavy Pulse-Gun");
        assert_eq!(weapons[1].attacks, "A4");
        assert_eq!(weapons[1].ap.as_deref(), Some("AP(1)"));
    }

    #[test]
    fn equipment_count_prefix_falls_back_to_attacks() {
        // No explicit A-number: the count is the attack total.
        let weapons = parse_equipment("2x Bashes (A2)");
        assert_eq!(weapons[0].attacks, "A2");
        let weapons = parse_equipment("3x CCWs (A2)");
        assert_eq!(weapons[0].name, "CCWs");
    }

    #[test]
    fn equipment_keeps_nameless_fragments_out() {
        // A dangling count with no name produces no weapon instead of an
        // empty-name weapon.
        assert!(parse_equipment("3x").is_empty());
        assert_eq!(parse_equipment("3x Stomp (A4, AP(1))").len(), 1);
    }

    #[test]
    fn category_directives_are_not_merged_into_previous_category() {
        // Old code only matched "Upgrade with"/"Replace"/"Add" as
        // substrings, so these sections merged into the previous category.
        let html = concat!(
            "| Name [Size] | Qua | Def | Equipment | Special Rules | Cost |\n",
            "| --- | --- | --- | --- | --- | --- |\n",
            "| Battle Suits [3] | 4+ | 3+ | 9x Suit-Bursts (18\", A1) 3x Bashes (A2) | Flying | 220pts |\n",
            "**Battle Suits [3]**- 220pts\n",
            "Flying, Targeting Visor\n",
            "| Weapon | RNG | ATK | AP | SPE |\n",
            "| --- | --- | --- | --- | --- |\n",
            "| 9x Suit-Bursts | 18\" | A1 | - | Rending |\n",
            "Replace one Suit-Burst\n",
            "Suit-Flamer (12\", A1, Blast(3))+10pts\n",
            "Any model may replace 2x Suit-Burst\n",
            "Suit-Flamer (12\", A1, Blast(3))+5pts\n",
            "Upgrade all models with\n",
            "Drop-Thrusters (Ambush)+25pts\n",
        );
        let units = extract_units_from_table(html).expect("units parse");
        let cats: Vec<&str> = units[0]
            .upgrade_categories
            .iter()
            .map(|c| c.category.as_str())
            .collect();
        // `Replace one Suit-Burst` is remapped onto the unit's actual
        // fixed-weapon name `Suit-Bursts`; the other sections keep their
        // source wording verbatim.
        assert_eq!(
            cats,
            vec![
                "Replace one Suit-Bursts",
                "Any model may replace 2x Suit-Burst",
                "Upgrade all models with",
            ]
        );
        // Each option lands in its own category.
        assert_eq!(units[0].upgrade_categories[0].upgrades.len(), 1);
        assert_eq!(units[0].upgrade_categories[1].upgrades.len(), 1);
        assert_eq!(units[0].upgrade_categories[2].upgrades.len(), 1);
    }

    #[test]
    fn repeated_category_names_are_not_deduped() {
        // Two genuine "Upgrade with one" sections are distinct groups.
        let html = concat!(
            "| Name [Size] | Qua | Def | Equipment | Special Rules | Cost |\n",
            "| --- | --- | --- | --- | --- | --- |\n",
            "| Captain [1] | 4+ | 3+ | 1x Flamer Pistol (6\", A1) | Hero | 60pts |\n",
            "**Captain [1]**- 60pts\n",
            "Hero, Watchborn\n",
            "| Weapon | RNG | ATK | AP | SPE |\n",
            "| --- | --- | --- | --- | --- |\n",
            "| Flamer Pistol | 6\" | A1 | - | Blast(3) |\n",
            "Upgrade with one\n",
            "Artillerist (Re-Position Artillery)+10pts\n",
            "Upgrade with\n",
            "Combat Shield (Shielded)+10pts\n",
            "Upgrade with one\n",
            "Jetpack (Ambush, Flying)+20pts\n",
        );
        let units = extract_units_from_table(html).expect("units parse");
        let cats: Vec<&str> = units[0]
            .upgrade_categories
            .iter()
            .map(|c| c.category.as_str())
            .collect();
        assert_eq!(
            cats,
            vec!["Upgrade with one", "Upgrade with", "Upgrade with one"]
        );
    }

    #[test]
    fn exact_duplicate_options_within_a_category_are_removed() {
        let html = concat!(
            "| Name [Size] | Qua | Def | Equipment | Special Rules | Cost |\n",
            "| --- | --- | --- | --- | --- | --- |\n",
            "| Walker [1] | 4+ | 3+ | 2x Walker Fists (A4, AP(4)) | Fear(2) | 400pts |\n",
            "**Walker [1]**- 400pts\n",
            "Fear(2)\n",
            "Replace one Walker Fist\n",
            "Energy Fist (A4, AP(3))+25pts\n",
            "Energy Fist (A4, AP(3))+25pts\n",
        );
        let units = extract_units_from_table(html).expect("units parse");
        let cat = &units[0].upgrade_categories[0];
        assert_eq!(cat.upgrades.len(), 1);
        assert_eq!(cat.upgrades[0].name, "Energy Fist");
    }

    #[test]
    fn replace_targets_are_remapped_to_fixed_weapon_names() {
        // `Replace any Heavy Hammer` targets the plural `Heavy Hammers` entry.
        let html = concat!(
            "| Name [Size] | Qua | Def | Equipment | Special Rules | Cost |\n",
            "| --- | --- | --- | --- | --- | --- |\n",
            "| Knight Titan [1] | 3+ | 2+ | 2x Heavy Hammers (A3, AP(3)) 1x Titan Hammer (A2, AP(4)) | Fortified | 250pts |\n",
            "**Knight Titan [1]**- 250pts\n",
            "Fortified\n",
            "| Weapon | RNG | ATK | AP | SPE |\n",
            "| --- | --- | --- | --- | --- |\n",
            "| 2x Heavy Hammers | - | A3 | 3 | - |\n",
            "| Titan Hammer | - | A2 | 4 | - |\n",
            "Replace any Heavy Hammer\n",
            "Heavy Missile Launcher (12\", A1, AP(3))+20pts\n",
        );
        let units = extract_units_from_table(html).expect("units parse");
        let cats: Vec<&str> = units[0]
            .upgrade_categories
            .iter()
            .map(|c| c.category.as_str())
            .collect();
        assert_eq!(cats, vec!["Replace any Heavy Hammers"]);
    }

    #[test]
    fn bare_numeric_rule_value_keeps_its_rule_name() {
        // `Transport(6)+20pts` keeps the rule name so the API can parse
        // `Transport(6)`, not the bare number `6`.
        let upgrade = parse_upgrade_option("Transport(6)+20pts").expect("option parses");
        assert_eq!(upgrade.name, "Transport");
        assert_eq!(upgrade.rules.as_deref(), Some("Transport(6)"));
        assert_eq!(upgrade.cost_modifier, "+20pts");
    }

    #[test]
    fn wolf_brothers_transports_gain_wolfborn() {
        let mut wolf = army(
            "Wolf Brothers",
            vec![
                unit(
                    "Wolf Drop Pod",
                    vec!["Ambush", "Fearless", "Immobile", "Transport(11)"],
                ),
                unit(
                    "Wolf Attack Speeder",
                    vec![
                        "Ambush",
                        "Fast",
                        "Fearless",
                        "Impact(3)",
                        "Strider",
                        "Wolfborn",
                    ],
                ),
            ],
        );
        normalize_wolf_brothers_transports(&mut wolf);
        let pod = &wolf.units[0];
        assert!(pod.special_rules.iter().any(|r| r == "Wolfborn"));
        // Non-transport units and already-listed units are untouched.
        assert_eq!(wolf.units[1].special_rules.len(), 6);
        // Non-Wolf armies are untouched.
        let mut dao = army(
            "DAO Union",
            vec![unit(
                "Hover Transport",
                vec!["Fast", "Impact(3)", "Strider", "Transport(11)"],
            )],
        );
        normalize_wolf_brothers_transports(&mut dao);
        assert!(!dao.units[0].special_rules.iter().any(|r| r == "Wolfborn"));
    }

    #[test]
    fn fixed_weapon_table_is_captured() {
        let html = concat!(
            "| Name [Size] | Qua | Def | Equipment | Special Rules | Cost |\n",
            "| --- | --- | --- | --- | --- | --- |\n",
            "| Battle Suits [3] | 4+ | 3+ | 9x Suit-Bursts (18\", A1) | Flying | 220pts |\n",
            "**Battle Suits [3]**- 220pts\n",
            "Flying\n",
            "| Weapon | RNG | ATK | AP | SPE |\n",
            "| --- | --- | --- | --- | --- |\n",
            "| 9x Suit-Bursts | 18\" | A1 | - | Rending |\n",
            "| 3x Bashes | - | A2 | - | - |\n",
            "Upgrade with one\n",
            "Storm Leader (Hit & Run Shooter Aura)+20pts\n",
        );
        let units = extract_units_from_table(html).expect("units parse");
        assert_eq!(units[0].fixed_weapons.len(), 2);
        assert_eq!(units[0].fixed_weapons[0].name, "Suit-Bursts");
        assert_eq!(units[0].fixed_weapons[1].name, "Bashes");
        // The two-column `| Upgrade | SPE |` table is not captured.
        let html = concat!(
            "| Name [Size] | Qua | Def | Equipment | Special Rules | Cost |\n",
            "| --- | --- | --- | --- | --- | --- |\n",
            "| Destroyers [3] | 4+ | 3+ | 3x CCWs (A3) | Ambush | 230pts |\n",
            "**Destroyers [3]**- 230pts\n",
            "Ambush\n",
            "| Upgrade | SPE |\n",
            "| --- | --- |\n",
            "| Combat Shield | Shielded |\n",
            "Upgrade one model with one\n",
            "Banner (Courage Aura)+5pts\n",
        );
        let units = extract_units_from_table(html).expect("units parse");
        assert!(units[0].fixed_weapons.is_empty());
    }
}
