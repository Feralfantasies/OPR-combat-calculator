# Bundle Update Log

## 2026-11-08
* **Update**: Added a comprehensive special-rule test suite (50 tests via `cargo test`): deterministic phase-level unit tests in `calculator.rs` plus 17 end-to-end statistical tests in `crates/api/tests/special_rules.rs` covering AP, Blast, Deadly, Impact, Tough, Bane, Furious, Relentless, Surge, Reliable, Rending, Regeneration, Shred, Unstoppable, and fatigue. Also implemented Impact(X), Rending AP(+4) pool, Bane defense rerolls, and Shred; fixed cover to grant +1 to defense rolls (lower target, not higher).
* **Update**: Added loadout customization — [Loadout Customization](/domain/loadout-customization.md) documents the upgrade data model; Hive Lord carries full upgrade options from the Alien Hives book.
* **Initialization**: Created the OKF v0.2 knowledge bundle with `index.md`, project overview, domain rules (combat sequence, special rules), Alien Hives army list, architecture decisions (modular monolith, api, frontend), and the damage simulation Attested Computation.
