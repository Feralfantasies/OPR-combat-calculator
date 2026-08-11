//! OPR Grimdark Future Damage Calculator
//!
//! A damage calculator for One Page Rules' Grimdark Future.
//! Structure:
//! - models/  : Unit, Weapon, SpecialRule definitions
//! - combat/  : Combat resolution (context, dice, calculator)
//! - armies/  : Faction army lists (Alien Hives, etc.)

mod armies;
mod combat;
mod models;

use armies::alien_hives::alien_hives;
use combat::{resolve_attack, CombatContext};

fn main() {
    println!("=== OPR Grimdark Future Damage Calculator ===\n");

    // Load the Alien Hives roster
    let roster = alien_hives();
    println!("Loaded {} Alien Hives units:\n", roster.len());
    for unit in &roster {
        println!(
            "  {} [{} models] - {}pts | Q{}+ D{}+",
            unit.name, unit.quantity, unit.points, unit.quality, unit.defense
        );
    }

    // Example combat: Shooter Grunts vs Hive Warriors
    let attacker = find_unit(&roster, "Shooter Grunts");
    let defender = find_unit(&roster, "Hive Warriors");

    match (attacker, defender) {
        (Some(attacker), Some(defender)) => run_demo(&attacker, &defender),
        _ => println!("Could not find demo units in roster."),
    }
}

/// Find a unit in the roster by name (cloned).
fn find_unit(roster: &[models::Unit], name: &str) -> Option<models::Unit> {
    roster.iter().find(|u| u.name == name).cloned()
}

/// Run a ranged + melee demo between two units.
fn run_demo(attacker: &models::Unit, defender: &models::Unit) {
    // --- Ranged attack ---
    println!("\nAttacker: {} [{} models]", attacker.name, attacker.quantity);
    println!("Defender: {} [{} models]", defender.name, defender.quantity);
    println!("\n--- Ranged Attack (24\" away) ---");

    let ranged_context = CombatContext::ranged(24);
    match resolve_attack(attacker, defender, &ranged_context) {
        Ok(result) => print_result(&result),
        Err(e) => println!("Error: {e}"),
    }

    // --- Melee attack ---
    println!("\n--- Melee Attack (Charge) ---");
    let melee_context = CombatContext::melee_charge();
    match resolve_attack(attacker, defender, &melee_context) {
        Ok(result) => print_result(&result),
        Err(e) => println!("Error: {e}"),
    }
}

fn print_result(result: &combat::CombatResult) {
    for attack in &result.attacks {
        println!(
            "  {}: {} attacks -> {} hits ({} blocked) -> {} wounds ({} regen) = {} net",
            attack.weapon_name,
            attack.base_attacks,
            attack.total_hits,
            attack.blocked_hits,
            attack.wounds_inflicted,
            attack.regenerated_wounds,
            attack.net_wounds
        );
    }
    println!(
        "  Total net wounds: {}, models removed: {}",
        result.total_net_wounds, result.models_removed
    );
}