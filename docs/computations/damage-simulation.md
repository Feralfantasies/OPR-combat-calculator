---
type: Attested Computation
title: Damage Simulation
description: Sanctioned Monte Carlo damage simulation — resolve the attacker vs defender attack N times and aggregate wounds/models removed.
tags: [computation, simulation, monte-carlo]
status: stable
runtime: rust
parameters:
  - { name: attacker_army, type: string, required: true }
  - { name: attacker_unit, type: string, required: true }
  - { name: defender_army, type: string, required: true }
  - { name: defender_unit, type: string, required: true }
  - { name: attack_type, type: string, required: true }
  - { name: distance, type: integer, required: false }
  - { name: defender_in_cover, type: boolean, required: false }
  - { name: iterations, type: integer, required: false }
executor:
  resource: crates/api/src/combat/calculator.rs
  receipt: [iterations, avg_net_wounds, avg_models_removed, min_net_wounds, max_net_wounds, weapons]
generated: { by: human:acleveland, at: 2026-11-08T22:55:00Z }
sources:
  - id: core-rules
    resource: reference/Grimdark Future - Core Rules v3.5.1.pdf
    title: Grimdark Future Core Rules v3.5.1
    author: OPR Games
---

# Computation

```rust
// Pseudocode of the sanctioned computation (api crate, frontend::routes).
// Bind parameters: attacker/defender looked up from army rosters by name.

let attacker = army_registry[attacker_army].find(attacker_unit)?;
let defender = army_registry[defender_army].find(defender_unit)?;
let context = match attack_type {
    "ranged"       => CombatContext::ranged(distance),
    "melee_charge" => CombatContext::melee_charge(),
};
let context = context.with_cover(defender_in_cover.unwrap_or(false));

let iterations = iterations.unwrap_or(1000).clamp(1, 100_000);

for _ in 0..iterations {
    let result = resolve_attack(&attacker, &defender, &context)?; // one full 8-phase run
    totals += result.total_net_wounds;
    models += result.models_removed;
    per_weapon += result.attacks;   // keyed by weapon name
    min/max tracked
}
// Aggregates: mean = total / iterations (f64), min, max.
```

# Contract

- **iterations**: defaults to **1000**; accepted range **1..=100000**
  (clamped, out-of-range request values rejected with 422). Higher values
  give a more accurate average at the cost of response time.
- **attack_type**: `"ranged"` or `"melee_charge"` only.
- **distance**: inches, only meaningful for ranged (affects Stealth /
  Relentless / Artillery thresholds at >9"). Defaults to 12.
- **defender_in_cover**: +1 Defense vs shooting. Defaults to false.

# Aggregation Method

Each iteration is an independent Monte Carlo run of the
[Combat Sequence](/domain/combat-sequence.md). The receipt contains:

- `avg_net_wounds` — mean net wounds per run.
- `avg_models_removed` — mean models removed per run.
- `min_net_wounds` / `max_net_wounds` — observed extremes.
- `weapons[]` — per-weapon mean hits, blocked, and net wounds.

Consumers MUST present results as aggregates over the stated `iterations`
count, never as a single deterministic value.

# Verification

The deterministic parts (AP raising the defense target, Blast multiplication
capped by unit size, Deadly multiplication, dice bounds, 6-always-hits /
1-always-fails) are covered by unit tests in the api crate. Each API change
to this computation should re-run those tests.