use crate::domain::{Plan, Schichtzuordnung, Mitarbeiter, Schicht};
use solverforge::prelude::*;
use solverforge::stream::joiner::{equal_bi, equal_on};
use solverforge::IncrementalConstraint;
use solverforge::stream::relational::Leaf;
use solverforge::stream::relational::operator::Pair;

/// HARD: TODO — describe what this constraint enforces.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardMediumSoftScore> {
    ConstraintFactory::<Plan, HardMediumSoftScore>::new()
        .for_each(Plan::schichtzuordnungs())
        .join((
            Plan::mitarbeiters(),
            equal_bi(
                |sz: &Schichtzuordnung| sz.mitarbeiter_idx,
                |m: &Mitarbeiter| Some(m.index)
            )
        ))
        .join((
            Plan::schichts(),
            equal_on(
                |row: &Pair<Leaf<'_, Schichtzuordnung>, Leaf<'_, Mitarbeiter>>| row.left.entity.schicht_id,
                |s: &Leaf<'_, Schicht>| s.entity.index
            )
        ))
        .group_by(
            |sz: &Schichtzuordnung, m: &Mitarbeiter, s: &Schicht| (s.index, m.schichtgruppe),
            collect_vec(|m: &Mitarbeiter| m.schichtgruppe.clone())
        )
        .penalize(hard_weight(join_weight))
        .named("selber_block")
}
