---
type: Game Rule
title: Special Rules Reference
description: Core Grimdark Future special rules and Alien Hives faction rules as implemented by the SpecialRule enum.
tags: [domain, special-rules, grimdark-future, alien-hives]
status: stable
generated: { by: human:acleveland, at: 2026-11-08T22:55:00Z }
sources:
  - id: core-rules
    resource: reference/Grimdark Future - Core Rules v3.5.1.pdf
    title: Grimdark Future Core Rules v3.5.1
    author: OPR Games
  - id: alien-hives
    resource: reference/alien_hives.pdf
    title: Grimdark Future - Alien Hives v3.5.3
    author: OPR Games
---

# Representation

All special rules are modelled as a single Rust enum (`SpecialRule`) in the
`api` crate. Parameterized rules carry their value, e.g. `AP(1)`, `Blast(3)`,
`Tough(12)`. Rules attach at two levels:

- **Unit level** — e.g. Stealth, Regeneration, Fearless, Hive Bond.
- **Weapon level** — e.g. AP, Rending, Blast, Furious, Deadly.

# Core Rules (v3.5.1)

| Rule | Effect |
|------|--------|
| `AP(X)` | Target gets -X to Defense rolls when blocking this weapon's hits. |
| `Blast(X)` | Ignores cover; each hit multiplied by X, capped at target unit size. |
| `Deadly(X)` | Each wound multiplied by X, assigned to a single model. |
| `Fear(X)` | Counts as +X wounds when determining melee winner. |
| `Impact(X)` | On charge, roll X dice (unless fatigued); each 2+ is a hit. |
| `Tough(X)` | Model takes X wounds before being killed. |
| `Caster(X)` | X spell tokens per round (casting out of scope for v1). |
| `Transport(X)` | Transport capacity (out of scope for v1 damage calc). |
| `Bane` | Ignores Regeneration; attacker rerolls unmodified Defense 6s. |
| `Furious` | Charging melee: unmodified 6 to hit deals +1 hit. |
| `Relentless` | Ranged vs >9": unmodified 6 to hit deals +1 hit. |
| `Surge` | Unmodified 6 to hit deals +1 hit. |
| `Rending` | Ignores Regeneration; unmodified 6 to hit gains AP(+4). |
| `Reliable` | Attacks at Quality 2+. |
| `Regeneration` | 5+ ignores each wound (unit-wide rule). |
| `Stealth` | Unit shot from >9": attackers get -1 to hit. |
| `Unstoppable` | Ignores Regeneration and negative modifiers. |
| `Indirect` | -1 to hit after moving; ignores LoS requirement. |
| `Artillery` | Hold only; +1 to hit vs >9"; -2 to be hit from >9". |
| `Slow` / `Fast` | Movement modifiers (no combat effect in v1). |
| `Counter` | Strikes first when charged (melee ordering, TODO). |
| `Fearless`, `Flying`, `Strider`, `Scout`, `Ambush`, `Immobile`, `Limited`, `Takedown`, `Thrust`, `Hero`, `Aircraft` | As per core rules; movement/deployment rules have no direct damage effect. |

See [Combat Sequence](/domain/combat-sequence.md) for the phase at which each
rule is applied.

# Alien Hives Faction Rules (v3.5.3)

| Rule | Effect |
|------|--------|
| `HiveBond` | Army-wide: +1 to morale test rolls from shooting/melee. |
| `Destructive` | Unmodified 6 to hit gains AP(+4). |
| `Resistance` | 6+ ignores each wound (2+ vs spells). |
| `NoRetreat` | Failed Shaken/Rout morale counts as passed, then roll for self-damage. |
| `SelfDestruct(X)` | When killed in melee (or after surviving melee), enemy takes X hits. |
| `Ravage(X)` | In melee, roll X dice; each 6 deals one wound. |
| `Precise` | +1 to hit when attacking. |
| `CasterGroup` | One model gains Caster(X), X = models with the rule. |
| `Infiltrate` | Ambush variant (deployment rule). |
| `Spawn(...)` | Once per game, place a new unit nearby (out of scope for v1). |
| `PiercingTag(X)`, `PiercingGrowth`, `Retaliate(X)`, `RegenerativeStrength`, `PredatorFighter`, `BreathAttack`, `Fortified`, `Shred`, `Strafing`, `SurpriseAttack(X)`, `TakedownStrike`, `Unpredictable`, `UnpredictableFighter`, auras (`FuriousAura`, `RegenerationAura`, ...) | As per the Alien Hives book; mostly activation/aura rules not yet wired into the simulator. |

# Implementation Status

Wired into the simulator today: AP, Blast, Deadly, Tough, Regeneration (+ skip
via Bane/Rending/Unstoppable), Reliable, Furious, Relentless, Surge, fatigue,
cover. Remaining rules are defined in the enum for data completeness and are
applied incrementally; see the TODO markers in `combat/calculator.rs`.