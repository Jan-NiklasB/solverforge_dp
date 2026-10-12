use crate::domain::{Plan, Schichtzuordnung, Mitarbeiter};
use solverforge::prelude::*;
use solverforge::stream::joiner::equal_bi;
use solverforge::IncrementalConstraint;
use crate::domain::types::Qualifikation;

/// HARD: Prüft, ob die benötigte Qualifikation des Mitarbeiters gegeben ist
pub fn constraint() -> impl IncrementalConstraint<Plan, HardMediumSoftScore> {
    ConstraintFactory::<Plan, HardMediumSoftScore>::new()
        .for_each(Plan::schichtzuordnungs())
        .join((
            Plan::mitarbeiters(),
            equal_bi(
                entity_join_key,
                fact_join_key,
            ),
        ))
        .penalize(hard_weight(join_weight))
        .named("benoetigte_qualifikation")
}

fn entity_join_key(entity: &Schichtzuordnung) -> Option<usize> {
    entity.mitarbeiter_idx
}

fn fact_join_key(_fact: &Mitarbeiter) -> Option<usize> {
    Some(_fact.index)
}

fn join_condition(_entity: &Schichtzuordnung, _fact: &Mitarbeiter) -> bool {
    !_fact.qualifikation.contains(&_entity.qualifikation)
}

fn join_weight(entity: &Schichtzuordnung, fact: &Mitarbeiter) -> HardMediumSoftScore {
    if join_condition(entity, fact) {
        <HardMediumSoftScore as Score>::one_hard()
    } else {
        <HardMediumSoftScore as Score>::zero()
    }
}
