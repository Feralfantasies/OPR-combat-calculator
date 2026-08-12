pub mod calculator;
pub mod context;
pub mod dice;

pub use calculator::{AttackResult, CombatResult, resolve_attack};
pub use context::{AttackType, CombatContext};
pub use dice::DieRoll;
