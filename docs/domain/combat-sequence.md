---
type: Game Rule
title: Combat Sequence
description: The 8-phase combat resolution pipeline used by the calculator, mapping each special rule to the phase where it applies.
tags: [domain, combat, rules, grimdark-future]
status: stable
generated: { by: human:acleveland, at: 2026-11-08T22:55:00Z }
sources:
  - id: core-rules
    resource: reference/Grimdark Future - Core Rules v3.5.1.pdf
    title: Grimdark Future Core Rules v3.5.1
    author: OPR Games
    last_modified: 2024-01-01
  - id: beginners-guide
    resource: reference/Grimdark Future - Beginner's Guide v3.5.1.pdf
    title: Grimdark Future Beginner's Guide v3.5.1
    author: OPR Games
    last_modified: 2024-01-01
---

# Overview

Combat in Grimdark Future follows a fixed sequence: determine attacks, roll to
hit (Quality tests), roll to block (Defense tests), then remove casualties. The
calculator implements this as an 8-phase pipeline so that each special rule has
a well-defined hook point.[^core-rules]

**Attack types:**

- **Ranged (shooting):** one-way damage; the defender only blocks.
- **Melee (charge):** the attacker strikes, and the defender may strike back.
- **Melee (return strikes):** the defender fights back, usually fatigued.

**Core roll rules:** roll a D6; hit on Quality+ (attacker) or block on Defense+
(defender). An unmodified 6 always succeeds; an unmodified 1 always fails,
regardless of modifiers.

# The 8 Phases

| Phase | Name | Rules that apply |
|-------|------|------------------|
| 1 | Attack count | `Impact(X)` — extra dice on a charge (each 2+ is a hit) |
| 2 | Hit roll setup | `Reliable` (attack at Q2+), `Artillery` (+1 vs >9"), `Indirect` (-1 after moving), `Thrust` (+1 when charging), `Stealth` (defender: -1 to attacker from >9"), fatigue (hit only on unmodified 6) |
| 3 | Hit roll resolution | `Furious` (charging melee 6 = +1 hit), `Relentless` (ranged >9" 6 = +1 hit), `Surge` (any 6 = +1 hit), `Rending` (6 to hit gains AP(+4)) |
| 4 | Hit multiplication | `Blast(X)` — each hit multiplied by X, capped at models in target unit |
| 5 | Defense roll setup | `AP(X)` — defender needs X higher to block; cover (+1 Defense vs shooting) |
| 6 | Defense roll resolution | `Bane` (reroll unmodified defense 6s), `Unstoppable` (ignore negative modifiers) |
| 7 | Wound application | `Deadly(X)` — wounds multiplied by X, assigned to one model; `Tough(X)` — model needs X wounds to die |
| 8 | Regeneration | `Regeneration` (5+ ignores each wound), skipped if the weapon has `Bane`, `Rending`, or `Unstoppable` |

# Implementation Mapping

| Phase | Code location |
|-------|---------------|
| 1 | `api` crate: `combat/calculator.rs` — attack count stage |
| 2 | `api` crate: `compute_hit_modifier()` + `effective_quality()` |
| 3 | `api` crate: `count_extra_hits()` over unmodified sixes |
| 4 | `api` crate: `apply_blast()` |
| 5 | `api` crate: `effective_defense()` |
| 6 | `api` crate: defense roll resolution (Bane reroll is a TODO) |
| 7 | `api` crate: `apply_deadly()` + `compute_models_removed()` |
| 8 | `api` crate: `apply_regeneration()` |

See [Special Rules](/domain/special-rules.md) for the full rule reference, and
[Damage Simulation](/computations/damage-simulation.md) for how repeated runs
are aggregated.