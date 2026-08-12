---
type: Game Rule
title: Loadout Customization
description: How unit upgrades from the army books are modelled (groups, options, selection modes) and applied before simulation.
tags: [domain, upgrades, loadout, alien-hives]
status: stable
generated: { by: human:acleveland, at: 2026-11-08T23:30:00Z }
sources:
  - id: alien-hives
    resource: reference/alien_hives.pdf
    title: Grimdark Future - Alien Hives v3.5.3
    author: OPR Games
---

# Overview

Army books let you customize a unit's loadout: swap weapons, add rules,
attach upgrades. The calculator models these as **upgrade groups** attached
to a unit, mirroring the book's wording.[^alien-hives]

[^alien-hives]: Grimdark Future - Alien Hives v3.5.3, OPR Games

# Data Model (`api::models::upgrades`)

| Type | Role |
|------|------|
| `UpgradeGroup` | A named group with a `SelectionMode` and optional `target_weapon` |
| `UpgradeOption` | One choice: name, description, cost, optional weapon change, rules to add |
| `WeaponChange` | `Replace(Weapon)` or `Add(Weapon)` |
| `SelectionMode` | `PickOne` ("Upgrade with one", "Replace X") or `Multiple` ("Upgrade with") |
| `ReplaceCount` | `One` ("Replace any X" — decrement quantity) or `All` ("Replace all X") |
| `UpgradeSelection` | User's choice: `{ group, option }` |

`apply_upgrades(&Unit, &[UpgradeSelection])` produces a new `Unit`: it adds
option costs to points, appends rules (deduplicated), and applies weapon
changes. Pick-one groups reject a second selection from the same group;
replacing a weapon the unit does not have is an error.

# Request Shape

`POST /api/simulate` accepts optional `upgrades` on each side:

```json
{
  "attacker": {
    "army": "alien_hives",
    "unit": "Hive Lord",
    "upgrades": [
      { "group": "Replace Shredder Cannon", "option": "Heavy Ravager Cannon" },
      { "group": "Upgrade with", "option": "Wings" }
    ]
  },
  ...
}
```

# Frontend Behaviour

When a unit has `upgrade_groups`, the panel renders **Loadout Options**:
pick-one groups become dropdowns with a `Default` choice; multiple groups
become checkboxes. Selections are collected into `upgrades` on the simulate
request.

# Implementation Status

Hive Lord is fully populated with its book options (rule upgrades, "Replace
any Heavy Razor Claw", "Replace Shredder Cannon", Wings). The framework
supports every group shape in the book; remaining units get their options
added incrementally.

See [API Module](/architecture/api.md) for the public surface and
[Combat Sequence](/domain/combat-sequence.md) for how the resulting loadout
is resolved.