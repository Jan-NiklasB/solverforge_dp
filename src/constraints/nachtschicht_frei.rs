use crate::domain::{Plan, Schichtzuordnung, ShiftRelation};
use solverforge::IncrementalConstraint;
use solverforge::prelude::*;
use solverforge::stream::joiner::{equal_bi, equal_on};
use solverforge::stream::relational::{Leaf, operator::Pair};

pub fn constraint() -> impl IncrementalConstraint<Plan, HardMediumSoftScore> {
    ConstraintFactory::<Plan, HardMediumSoftScore>::new()
        .for_each(Plan::shift_relations())
        .filter(|r: &ShiftRelation| r.night_to_day)
        .join((
            Plan::schichtzuordnungs(),
            equal_bi(
                |r: &ShiftRelation| r.first,
                |a: &Schichtzuordnung| a.schicht_id,
            ),
        ))
        .filter(|_: &ShiftRelation, a: &Schichtzuordnung| a.mitarbeiter_idx.is_some())
        .join((
            Plan::schichtzuordnungs(),
            equal_on(
                |row: &Pair<Leaf<'_, ShiftRelation>, Leaf<'_, Schichtzuordnung>>| {
                    (row.left.entity.second, row.right.entity.mitarbeiter_idx)
                },
                |a: &Leaf<'_, Schichtzuordnung>| (a.entity.schicht_id, a.entity.mitarbeiter_idx),
            ),
        ))
        .filter(
            |r: &ShiftRelation, a: &Schichtzuordnung, b: &Schichtzuordnung| {
                r.first != r.second || a.id < b.id
            },
        )
        .penalize(HardMediumSoftScore::ONE_MEDIUM)
        .named("42h nach Nachtschicht")
}
