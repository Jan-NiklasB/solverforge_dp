use crate::domain::{Plan, Schicht, Schichtzuordnung};
use solverforge::prelude::*;
use solverforge::stream::joiner::{equal_bi, equal_on};
use solverforge::stream::relational::operator::Pair;
use solverforge::stream::relational::Leaf;
use solverforge::IncrementalConstraint;

pub fn constraint() -> impl IncrementalConstraint<Plan, HardMediumSoftScore> {
    ConstraintFactory::<Plan, HardMediumSoftScore>::new()
        .for_each(Plan::schichtzuordnungs())
        .join((
            Plan::schichts(),
            equal_bi(|sz: &Schichtzuordnung| sz.schicht_id, |s: &Schicht| s.index),
        ))
        .join((
            Plan::schichtzuordnungs(),
            equal_on(
                |row: &Pair<Leaf<'_, Schichtzuordnung>, Leaf<'_, Schicht>>| {
                    row.left.entity.mitarbeiter_idx
                },
                |sz: &Leaf<'_, Schichtzuordnung>| sz.entity.mitarbeiter_idx,
            ),
        ))
        .join((
            Plan::schichts(),
            equal_on(
                |row: &Pair<
                    Pair<Leaf<'_, Schichtzuordnung>, Leaf<'_, Schicht>>,
                    Leaf<'_, Schichtzuordnung>,
                >| Some(row.right.entity.schicht_id),
                |s: &Leaf<'_, Schicht>| Some(s.entity.index),
            ),
        ))
        .filter(
            |sz1: &Schichtzuordnung, s1: &Schicht, sz2: &Schichtzuordnung, s2: &Schicht| {
                sz1.mitarbeiter_idx.is_some()
                    && sz1.id < sz2.id
                    && s1.start < s2.ende
                    && s2.start < s1.ende
            },
        )
        .penalize(HardMediumSoftScore::ONE_HARD)
        .named("ueberlappende_schichten")
}
