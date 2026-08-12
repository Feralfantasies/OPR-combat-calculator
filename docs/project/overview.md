---
type: Project
title: OPR Combat Calculator
description: Unofficial fan-made damage calculator for One Page Rules' Grimdark Future, built in Rust.
tags: [project, grimdark-future, one-page-rules, damage-calculator]
status: stable
generated: { by: human:acleveland, at: 2026-11-08T22:55:00Z }
sources:
  - id: core-rules
    resource: reference/Grimdark Future - Core Rules v3.5.1.pdf
    title: Grimdark Future Core Rules v3.5.1
    author: OPR Games
  - id: beginners-guide
    resource: reference/Grimdark Future - Beginner's Guide v3.5.1.pdf
    title: Grimdark Future Beginner's Guide v3.5.1
    author: OPR Games
---

# Purpose

The OPR Combat Calculator resolves combat between two units in Grimdark Future:
an attacker and a defender. Given an attack type (ranged or melee), distance,
and situational modifiers, it simulates the full combat sequence and reports
expected wounds and models removed.

The calculator is built as a **modular monolith** in Rust:

- `api` — the simulation core (models, combat engine, army data). See
  [API Module](/architecture/api.md).
- `frontend` — an axum-based web app for configuring units and running
  simulations. See [Frontend Module](/architecture/frontend.md).

See [Modular Monolith](/architecture/modular-monolith.md) for the layout
decision, and [Damage Simulation](/computations/damage-simulation.md) for the
sanctioned computation.

# Disclaimer

This is an **unofficial fan project**. It is not affiliated with, endorsed by,
or sponsored by OPR Games in any way. Grimdark Future and all associated names,
units, and rules are the intellectual property of OPR Games
(www.onepagerules.com). This tool is made by a fan, for the community, to help
players quickly work out combat outcomes. Please support the official release
by downloading the free rules from the OPR website.[^core-rules]

[^core-rules]: Grimdark Future Core Rules v3.5.1, OPR Games

# License

The project source is licensed under the MIT License. See `LICENSE` in the
repository root. The OPR game rules and unit data referenced remain the
property of OPR Games.

# Scope

- **In scope:** unit stats, weapons, special rules, the combat sequence,
  dice simulation with aggregation over many iterations.
- **Out of scope:** movement, terrain placement, morale/routing outcomes,
  army list building validation, points-cost optimisation.