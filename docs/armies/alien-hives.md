---
type: Army List
title: Alien Hives
description: Alien Hives faction overview, army-wide Hive Bond rule, and the 35-unit base roster implemented in the api crate.
tags: [army, alien-hives, grimdark-future]
status: stable
generated: { by: human:acleveland, at: 2026-11-08T22:55:00Z }
sources:
  - id: alien-hives
    resource: reference/alien_hives.pdf
    title: Grimdark Future - Alien Hives v3.5.3
    author: OPR Games
---

# Overview

The Alien Hives are a faction of biotechnology-using aliens made up of
subspecies from the remote frontiers, seeking to recreate the Mother that
shaped them.[^alien-hives]

**Army-wide special rule — Hive Bond:** units where all models have this rule
get +1 to morale test rolls from shooting/melee. Every unit in the roster
carries it (Spores carry faction-specific rules instead).

[^alien-hives]: Grimdark Future - Alien Hives v3.5.3, OPR Games

# Base Roster (35 units, default loadouts)

Upgrades and weapon exchanges from the book are **not** modelled; each unit is
stored with its default loadout. Named heroes (Drekhor, Ksi'adoz, Druzhak,
Trooseis, Khuurkhi, Nisuv, Vradhez) are not yet included.

## Heroes

| Unit | Size | Q | D | Tough | Points |
|------|------|---|---|-------|--------|
| Hive Lord | 1 | 3+ | 2+ | 12 | 360 |
| Prime Warrior | 1 | 4+ | 4+ | 6 | 80 |
| Snatcher Lord | 1 | 3+ | 4+ | 3 | 85 |
| Grunt Veteran | 1 | 5+ | 5+ | 3 | 25 |

## Troops

| Unit | Size | Q | D | Points |
|------|------|---|---|--------|
| Assault Grunts | 10 | 5+ | 5+ | 105 |
| Shooter Grunts | 10 | 5+ | 5+ | 115 |
| Psycho-Grunts | 10 | 5+ | 5+ | 125 |
| Winged Grunts | 10 | 5+ | 5+ | 125 |
| Support Grunts | 3 | 5+ | 5+ | 145 |

## Specialists

| Unit | Size | Q | D | Tough | Points |
|------|------|---|---|-------|--------|
| Soul-Snatchers | 5 | 3+ | 4+ | - | 160 |
| Hive Swarms | 3 | 5+ | 6+ | 3 | 75 |
| Hive Warriors | 3 | 4+ | 4+ | 3 | 115 |
| Ravenous Beasts | 3 | 4+ | 4+ | 3 | 155 |
| Venom Beasts | 3 | 4+ | 4+ | 3 | 150 |
| Hive Guardians | 3 | 3+ | 3+ | 3 | 155 |
| Shadow Leapers | 3 | 3+ | 4+ | 3 | 230 |
| Synapse Beasts | 3 | 4+ | 4+ | 3 | 200 |
| Spores | 5 | 6+ | 6+ | - | 60 |
| Massive Spores | 3 | 6+ | 6+ | 3 | 115 |

## Monsters

| Unit | Q | D | Tough | Points |
|------|---|---|-------|--------|
| Shadow Hunter | 3+ | 4+ | 6 | 180 |
| Mortar Beast | 4+ | 3+ | 6 | 155 |
| Synapse Tyrant | 4+ | 4+ | 6 | 190 |
| Flamer Beast | 4+ | 3+ | 6 | 175 |
| Invasion Carrier Spore | 4+ | 3+ | 6 | 135 |
| Carnivo-Rex | 4+ | 2+ | 12 | 295 |
| Toxico-Rex | 4+ | 2+ | 12 | 360 |
| Psycho-Rex | 4+ | 2+ | 12 | 420 |
| Hive Burrower | 4+ | 2+ | 15 | 420 |
| Tyrant Great Beast | 4+ | 2+ | 15 | 470 |
| Spawning Great Beast | 4+ | 2+ | 15 | 505 |
| Devourer Great Beast | 4+ | 2+ | 15 | 445 |
| Artillery Great Beast | 4+ | 2+ | 15 | 550 |
| Invasion Artillery Spore | 4+ | 3+ | 6 | 195 |
| Hive Titan | 3+ | 2+ | 18 | 685 |
| Rapacious Beast | 4+ | 2+ | 6 | 215 |

# Code Location

All units are defined in `armies/alien_hives.rs` of the `api` crate, built via
`Unit::new(...)` builders with chained `.with_rule()` / `.with_weapon()` calls.
See [Special Rules](/domain/special-rules.md) for the faction rule reference.