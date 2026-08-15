---
okf_version: "0.2"
---

# OPR Combat Calculator — Knowledge Bundle

Knowledge corpus for the OPR Combat Calculator, an unofficial fan-made damage
calculator for One Page Rules' Grimdark Future.

# Project

* [Project Overview](project/overview.md) - Purpose, scope, licensing, and the fan-project disclaimer.

# Domain

* [Combat Sequence](domain/combat-sequence.md) - The 8-phase combat resolution pipeline and where each special rule applies.
* [Special Rules](domain/special-rules.md) - Reference for core Grimdark Future rules and Alien Hives faction rules.
* [Loadout Customization](domain/loadout-customization.md) - Upgrade groups, weapon swaps, and how loadouts are applied before simulation.

# Armies

* [Alien Hives](armies/alien-hives.md) - Faction overview, army-wide Hive Bond rule, and the 35-unit base roster.

# Architecture

* [Modular Monolith](architecture/modular-monolith.md) - Workspace layout decision and module boundaries.
* [API Module](architecture/api.md) - The core simulation library crate: models, combat engine, army data.
* [Frontend Module](architecture/frontend.md) - The axum web application: UI behaviour and JSON API surface.

* [Data Collector Module](architecture/data-collector.md) - The data scraping pipeline: fetch, parse, and YAML generation.

# Computations

* [Damage Simulation](computations/damage-simulation.md) - The sanctioned damage simulation: inputs, iteration bounds, and aggregation method.
