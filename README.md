# OPR Damage Calculator

A damage calculator for [One Page Rules](https://onepagerules.com)' **Grimdark Future**, written in Rust. Put two units in — attacker and defender — pick a ranged or melee attack, and the calculator resolves the combat sequence including special rules like AP, Blast, Deadly, Tough, and Regeneration.

## Disclaimer

This is an unofficial fan project. It is **not affiliated with, endorsed by, or sponsored by OPR Games** in any way. Grimdark Future and all associated names, units, and rules are the intellectual property of OPR Games (www.onepagerules.com). This tool is made by a fan, for the community, to help players quickly work out combat outcomes. Please support the official release by downloading the free rules from the OPR website.

## Features

- Unit and weapon modelling with Quality, Defense, Tough, and points
- Full special rules system (enum-based, parameterized rules like `AP(1)`, `Blast(3)`)
- Combat context: ranged vs melee charge, distance, cover, fatigue
- Dice simulation with OPR rules (unmodified 6 always hits, unmodified 1 always fails)
- Alien Hives army list included (35 base unit profiles)

## Project Structure

```
src/
├── main.rs              # Entry point + demo
├── models/
│   ├── rules.rs         # SpecialRule enum (core + Alien Hives rules)
│   ├── unit.rs          # Unit struct with builder pattern
│   └── weapons.rs       # Weapon struct (melee / ranged)
├── combat/
│   ├── calculator.rs    # Combat resolution pipeline (8 phases)
│   ├── context.rs       # CombatContext (attack type, distance, cover, etc.)
│   └── dice.rs          # Dice rolling utilities
└── armies/
    └── alien_hives.rs   # Alien Hives unit profiles
```

## Usage

```bash
cargo run
```

This loads the Alien Hives roster, prints all units, and runs a demo combat (Shooter Grunts vs Hive Warriors, both ranged and melee).

## Tests

```bash
cargo test
```

## License

This project is licensed under the [MIT License](LICENSE).