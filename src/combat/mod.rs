pub mod calculator;
pub mod context;
pub mod dice;

pub use calculator::{resolve_attack, AttackResult, CombatResult};
pub use context::{AttackType, CombatContext};
pub use dice::DieRoll;