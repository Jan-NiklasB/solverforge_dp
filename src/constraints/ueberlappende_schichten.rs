use crate::domain::{Plan, Schichtzuordnung, Mitarbeiter, Schicht};
use solverforge::prelude::*;
use solverforge::stream::joiner::{equal_bi, equal_on};
use solverforge::IncrementalConstraint;
use solverforge::stream::relational::Leaf;
use solverforge::stream::{source, ChangeSource};
use solverforge::stream::relational::operator::Pair;

/// HARD: TODO — describe what this constraint enforces.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardMediumSoftScore> {
    ConstraintFactory::<Plan, HardMediumSoftScore>::new()
        .for_each(Plan::schichtzuordnungs())
        .join((
            Plan::schichts(),
            equal_bi(
                |sz: &Schichtzuordnung| sz.schicht_id,
                |s: &Schicht| s.index
            )
        ))
        .join((
            source(Plan::mitarbeiters as fn(&Plan) -> &[Mitarbeiter], ChangeSource::Descriptor(2)),
            equal_on(
                |row: &Pair<Leaf<'_, Schichtzuordnung>, Leaf<'_, Schicht>>| row.left.entity.mitarbeiter_idx,
                |m: &Leaf<Mitarbeiter>| Some(m.entity.index),
            ),
        ))
        .penalize(hard_weight(join_weight))
        .named("ueberlappende_schichten")
}

fn entity_join_key(entity: &Schichtzuordnung) -> Option<usize> {
    entity.mitarbeiter_idx
}

fn fact_join_key(_fact: &Mitarbeiter) -> Option<usize> {
    panic!("replace placeholder join key extractor before enabling this constraint")
}

fn join_condition(_sz: &Schichtzuordnung, _m: &Mitarbeiter, _s: &Schicht) -> bool {
    panic!("replace placeholder join condition before enabling this constraint")
}

fn join_weight(sz: &Schichtzuordnung, m: &Mitarbeiter, s: &Schicht) -> HardMediumSoftScore {
    if join_condition(sz, m, s) {
        <HardMediumSoftScore as Score>::one_hard()
    } else {
        <HardMediumSoftScore as Score>::zero()
    }
}
