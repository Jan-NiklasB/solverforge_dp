use crate::domain::{Plan, Schichtzuordnung};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// HARD: TODO — describe what this constraint enforces.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardMediumSoftScore> {
    ConstraintFactory::<Plan, HardMediumSoftScore>::new()
        .for_each(Plan::schichtzuordnungs())
        .penalize(hard_weight(unary_weight))
        .named("shift_assigned")
}

fn unary_condition(_entity: &Schichtzuordnung) -> bool {
    _entity.mitarbeiter_idx.is_none()
}

fn unary_weight(entity: &Schichtzuordnung) -> HardMediumSoftScore {
    if unary_condition(entity) {
        <HardMediumSoftScore as Score>::one_hard()
    } else {
        <HardMediumSoftScore as Score>::zero()
    }
}
