//! Dice rolling utilities for simulation-based calculation.

use rand::Rng;

/// Result of a single die roll, tracking both the raw roll and any modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DieRoll {
    /// The unmodified die result (1-6)
    pub raw: u8,
    /// The final result after modifiers are applied
    pub modified: i32,
}

impl DieRoll {
    /// Check if the raw (unmodified) roll is a 6
    #[must_use]
    pub fn is_unmodified_six(&self) -> bool {
        self.raw == 6
    }

    /// Check if the raw (unmodified) roll is a 1
    #[must_use]
    pub fn is_unmodified_one(&self) -> bool {
        self.raw == 1
    }
}

/// Roll a single d6 and apply a modifier.
/// Note: A roll of 1 always fails, a roll of 6 always succeeds,
/// but the modified value is still recorded for threshold checks.
#[must_use]
pub fn roll_d6(modifier: i32) -> DieRoll {
    let raw = rand::rng().random_range(1..=6);
    DieRoll {
        raw,
        modified: i32::from(raw).saturating_add(modifier),
    }
}

/// Roll multiple d6 with the same modifier.
#[must_use]
pub fn roll_d6s(count: u32, modifier: i32) -> Vec<DieRoll> {
    (0..count).map(|_| roll_d6(modifier)).collect()
}

/// Count how many rolls meet or exceed the target value.
/// A raw roll of 6 always succeeds, a raw roll of 1 always fails,
/// regardless of modifiers.
#[must_use]
pub fn count_successes(rolls: &[DieRoll], target: u8) -> u32 {
    rolls
        .iter()
        .filter(|r| {
            if r.raw == 6 {
                true // 6 always succeeds
            } else if r.raw == 1 {
                false // 1 always fails
            } else {
                r.modified >= i32::from(target)
            }
        })
        .count()
        .try_into()
        .unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roll_bounds() {
        for _ in 0..100 {
            let roll = roll_d6(0);
            assert!((1..=6).contains(&roll.raw));
            assert_eq!(roll.modified, i32::from(roll.raw));
        }
    }

    #[test]
    fn test_always_succeed_on_six() {
        let roll = DieRoll { raw: 6, modified: 0 }; // even with -6
        assert_eq!(count_successes(&[roll], 6), 1);
    }

    #[test]
    fn test_always_fail_on_one() {
        let roll = DieRoll { raw: 1, modified: 10 }; // even with +10
        assert_eq!(count_successes(&[roll], 1), 0);
    }

    #[test]
    fn test_success_threshold() {
        let rolls = vec![
            DieRoll { raw: 3, modified: 3 },
            DieRoll { raw: 4, modified: 4 },
            DieRoll { raw: 5, modified: 5 },
        ];
        assert_eq!(count_successes(&rolls, 4), 2);
    }
}