---
type: Module
title: Frontend Module
description: The frontend crate — axum web application serving the unit configuration UI and JSON simulation API.
tags: [architecture, frontend, axum, web, json]
status: stable
generated: { by: human:acleveland, at: 2026-11-08T22:55:00Z }
---

# Purpose

The `frontend` crate is a binary crate that serves both the web UI and the
JSON API from a single axum server (default port 3000). It depends only on
the public surface of the [API Module](/architecture/api.md).

# UI

Vanilla HTML/JS/CSS served from `static/` — no build step, no framework:

- **Attacker/defender panels:** army dropdown → unit dropdown → unit card
  showing stats, weapons, and special rules.
- **Options row:** attack type (ranged / melee charge), distance, defender
  in cover, iterations (number input, min 1, max 100000, default 1000).
- **Run Simulation button:** POSTs to `/api/simulate` and renders the
  aggregated results table.

# JSON API

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/armies` | GET | List available army names. |
| `/api/armies/{name}/units` | GET | Full unit roster for the dropdowns and unit cards. |
| `/api/simulate` | POST | Run the simulation N times and return aggregates. |

## Simulate request

```json
{
  "attacker": { "army": "alien_hives", "unit": "Shooter Grunts" },
  "defender": { "army": "alien_hives", "unit": "Hive Warriors" },
  "attack_type": "ranged",
  "distance": 12,
  "defender_in_cover": false,
  "iterations": 1000
}
```

`attack_type` is `"ranged"` or `"melee_charge"`. `iterations` is validated to
the range 1..=100000; out-of-range values return a 422 with an error message.

## Simulate response

```json
{
  "iterations": 1000,
  "avg_net_wounds": 3.42,
  "avg_models_removed": 1.1,
  "min_net_wounds": 0,
  "max_net_wounds": 8,
  "weapons": [
    {
      "name": "Bio-Spiners",
      "avg_hits": 7.9,
      "avg_blocked": 3.2,
      "avg_net_wounds": 3.42
    }
  ]
}
```

Aggregation method is the sanctioned computation:
[Damage Simulation](/computations/damage-simulation.md).

# Behaviour Notes

- Unknown army or unit names return 404 with an error message.
- The handler runs synchronously on a tokio blocking thread; at 100000
  iterations the response is slower but still bounded (Rust dice simulation
  is fast), so no background job queue is needed.
- Melee simulations resolve the attacker's charge strikes only; defender
  return strikes are a planned extension.

# Dependencies

- `axum` — HTTP server and routing.
- `tokio` — async runtime (full features).
- `serde` / `serde_json` — request/response bodies.
- `api` (path dependency) — simulation core.