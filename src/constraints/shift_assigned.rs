use crate::domain::{Plan, Schichtzuordnung};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: TODO — describe what this constraint enforces.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardMediumSoftScore> {
    ConstraintFactory::<Plan, HardMediumSoftScore>::new()
        .for_each(Plan::schichtzuordnungs())
        .filter(|sz: &Schichtzuordnung| {
            sz.mitarbeiter_idx.is_none()
        })
        .penalize(HardMediumSoftScore::one_hard())
        .named("Kein Mitarbeiter zur Schicht zugewiesen")
}
