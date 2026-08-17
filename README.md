# OPR Combat Calculator

A damage calculator for [One Page Rules](https://onepagerules.com)' **Grimdark Future**, written in Rust. Configure an attacker and a defender in the web UI, choose ranged or melee, hit **Run Simulation**, and get aggregated combat results over thousands of dice rolls.

## Disclaimer

This is an unofficial fan project. It is **not affiliated with, endorsed by, or sponsored by OPR Games** in any way. Grimdark Future and all associated names, units, and rules are the intellectual property of OPR Games ([the OPR website](https://onepagerules.com)). This tool is made by a fan, for the community, to help players quickly work out combat outcomes. Please support the official release by downloading the free rules from the OPR website.

## Architecture — Modular Monolith

```
├── Cargo.toml            # workspace + shared strict clippy lints
├── crates/
│   ├── api/              # library crate: simulation core
│   │   ├── src/
│   │   │   ├── models/   # Unit, Weapon, SpecialRule (serde)
│   │   │   ├── combat/   # 8-phase calculator, dice, context
│   │   │   └── armies/   # faction rosters (Alien Hives, 35 units)
│   │   └── tests/        # integration tests
│   ├── frontend/         # binary crate: axum web app
│   │   ├── static/       # index.html, app.js, style.css (vanilla JS)
│   │   └── src/main.rs   # /api/armies, /api/armies/{id}/units, /api/simulate
│   └── data-collector/   # binary crate: army data scraper
│       ├── src/
│       │   ├── main.rs           # CLI with fetch/parse subcommands
│       │   ├── fetch.rs          # Jina Reader API integration
│       │   ├── parser.rs         # HTML parsing & YAML generation
│       │   ├── cache.rs          # local HTML caching with metadata
│       │   ├── http_client.rs    # rate-limited HTTP client
│       │   └── error.rs          # error types
│       └── data/
│           ├── cache/            # cached HTML from Army Forge (git-ignored)
│           └── armies/           # 43 generated army data files (committed)
└── docs/                 # OKF v0.2 knowledge bundle (rules, architecture, computation)
```

The project consists of three crates:

- **`opr-api`** — simulation core with unit/weapon models, combat phases, and Monte Carlo simulation
- **`opr-frontend`** — web UI that depends only on the public surface of `opr-api`
- **`data-collector`** — standalone tool that scrapes army data from the OPR Army Forge website, caches HTML locally, and generates versioned YAML files for all 43 Grimdark Future armies (including subfactions)

The frontend depends only on the public surface of `opr-api`; the crate boundary is enforced at compile time. The data-collector is independent and generates the YAML files that populate the army rosters. See `docs/` for the full knowledge bundle, including the sanctioned damage simulation definition (`docs/computations/damage-simulation.md`) and data-collector architecture (`docs/architecture/data-collector.md`).

## Running

```bash
cargo run -p opr-frontend
```

Then open <http://127.0.0.1:3000> — pick attacker/defender units, set attack type/distance/cover/iterations (default 1000, max 100000), and click **Run Simulation**.

## Tests

```bash
cargo test
```

## Features

- Unit/weapon modelling: Quality, Defense, Tough, points, builder pattern
- Special rules as a parameterized enum (`AP(1)`, `Blast(3)`, `Tough(12)`, …)
- Combat phases: AP, Blast, Deadly, Tough, Regeneration (+ Bane/Rending/Unstoppable skip), Reliable, Furious/Relentless/Surge extra hits, cover, fatigue
- Monte Carlo simulation with mean/min/max aggregation over N iterations
- **Loadout customization**: pick-one rule upgrades, weapon swaps ("Replace Shredder Cannon" → 9 ranged/melee options), and optional add-ons per army book; rendered as dropdowns/checkboxes in the UI. Hive Lord fully populated; other units incrementally
- Alien Hives roster (35 units)

## License

MIT — see [LICENSE](LICENSE). OPR game rules and unit data remain the property of OPR Games.
