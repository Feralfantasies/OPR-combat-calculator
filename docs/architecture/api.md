---
type: Module
title: API Module
description: The api crate — simulation core containing models, the combat engine, and faction army data, exposed as a library.
tags: [architecture, api, rust, crate]
status: stable
generated: { by: human:acleveland, at: 2026-11-08T22:55:00Z }
---

# Purpose

The `api` crate is the simulation core. It owns all game knowledge (units,
weapons, special rules, army rosters) and the combat resolution engine. It is
a **library crate** with no I/O: the frontend (or any future consumer) drives
it through its public API.

# Public Surface

```rust
// Re-exported from api::
pub use models::{SpecialRule, Unit, Weapon};
pub use combat::{resolve_attack, CombatContext, AttackType, CombatResult, AttackResult};
pub use armies::alien_hives::alien_hives;   // fn() -> Vec<Unit>
```

## Key types

| Type | Role |
|------|------|
| `Unit` | Name, quantity, quality, defense, tough, points, weapons, special rules. Builder pattern (`with_weapon`, `with_rule`). |
| `Weapon` | Name, quantity, attacks, `range: Option<u8>` (None = melee), special rules. |
| `SpecialRule` | Enum of all core + Alien Hives rules; parameterized variants carry their value. See [Special Rules](/domain/special-rules.md). |
| `CombatContext` | Attack type, distance, charging, moved, cover, fatigue. |
| `CombatResult` / `AttackResult` | Per-weapon breakdown and totals for one resolution. |
| `resolve_attack(attacker, defender, context)` | Resolves one attack; the single entry point. See [Combat Sequence](/domain/combat-sequence.md). |

# Serialization

All public types derive `serde::Serialize` / `Deserialize` so the frontend can
transport unit data and simulation requests/responses as JSON without
duplicating type definitions.

# Testing

Unit tests live alongside modules (`combat/dice.rs`, `combat/calculator.rs`);
the original CLI demo behaviour is preserved as an integration test
(`tests/`) that resolves a ranged and a melee attack between two roster units.

# Dependencies

- `rand` — dice simulation.
- `serde` (+ `serde` derive) — serialization.

See [Modular Monolith](/architecture/modular-monolith.md) for how this crate
relates to [Frontend Module](/architecture/frontend.md).