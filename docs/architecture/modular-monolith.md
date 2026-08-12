---
type: Architecture Decision
title: Modular Monolith Layout
description: Decision to structure the project as a Cargo workspace with two crates (api, frontend) enforcing module boundaries at compile time.
tags: [architecture, modular-monolith, cargo-workspace, rust]
status: stable
generated: { by: human:acleveland, at: 2026-11-08T22:55:00Z }
---

# Decision

The project is structured as a **Cargo workspace modular monolith**:

```
opr-damage-calculator/
├── Cargo.toml              # [workspace] + [workspace.lints.clippy] (shared strict lints)
├── crates/
│   ├── api/                # library crate — simulation core
│   │   └── src/
│   │       ├── lib.rs      # pub mod armies; pub mod combat; pub mod models;
│   │       ├── armies/     # faction army lists (alien_hives)
│   │       ├── combat/     # calculator, context, dice
│   │       └── models/     # rules, unit, weapons (+ serde derives)
│   └── frontend/           # binary crate — axum web app
│       ├── static/         # index.html, app.js, style.css (vanilla, no build step)
│       └── src/            # main.rs, dto.rs, routes.rs
└── docs/                   # OKF knowledge bundle (this directory)
```

# Rationale

1. **Compile-time boundary.** Separate crates prevent the frontend from
   reaching into api internals; the frontend depends only on the api crate's
   public surface. This is the core value of a modular monolith over a
   single-crate module layout.
2. **Shared strictness.** The project's strict clippy lint set (pedantic +
   nursery deny, no unwrap/expect/panic, no `as` conversions, saturating
   arithmetic) lives in `[workspace.lints.clippy]` and is inherited by both
   crates via `[lints] workspace = true`.
3. **Single deployable.** The frontend binary serves the static UI and the
   JSON API from one process — no inter-service calls, no deployment
   orchestration. The boundary is structural, not physical.
4. **Future optionality.** If the simulation core ever needs to be published
   as a library or split into a service, the crate boundary is already in the
   right place.

# Module Ownership

| Module | Crate | Depends on | Description |
|--------|-------|------------|-------------|
| Models | `api::models` | — | `Unit`, `Weapon`, `SpecialRule` |
| Combat | `api::combat` | models | Dice, context, 8-phase calculator |
| Armies | `api::armies` | models | Faction rosters |
| Frontend | `frontend` | `api` | axum server, JSON routes, static UI |

See [API Module](/architecture/api.md) and
[Frontend Module](/architecture/frontend.md) for details.