use crate::domain::{Plan, Schichtzuordnung};
use solverforge::prelude::*;
use solverforge::stream::collector::LoadBalance;
use solverforge::IncrementalConstraint;

/// SOFT: TODO — describe what this constraint optimizes.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardMediumSoftScore> {
    ConstraintFactory::<Plan, HardMediumSoftScore>::new()
        .for_each(Plan::schichtzuordnungs())
        .group_by(
            balance_scope,
            load_balance(balance_group_key, balance_metric),
        )
        .penalize(balance_weight)
        .named("balancing")
}

fn balance_scope(_entity: &Schichtzuordnung) -> usize {
    0
}

fn balance_group_key(entity: &Schichtzuordnung) -> Option<usize> {
    entity.mitarbeiter_idx
}

fn balance_metric(_entity: &Schichtzuordnung) -> i64 {
    1
}

fn balance_weight(scope: &usize, load: &LoadBalance<Option<usize>>) -> HardMediumSoftScore {
        HardMediumSoftScore::of_medium(load.unfairness())
}
