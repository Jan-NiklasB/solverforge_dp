use crate::domain::{BlockMembership, Mitarbeiter, Plan, Schichtzuordnung};
use solverforge::IncrementalConstraint;
use solverforge::prelude::*;
use solverforge::stream::joiner::{equal_bi, equal_on};
use solverforge::stream::relational::{Leaf, operator::Pair};

pub fn constraint() -> impl IncrementalConstraint<Plan, HardMediumSoftScore> {
    ConstraintFactory::<Plan, HardMediumSoftScore>::new()
        .for_each(Plan::schichtzuordnungs())
        .join((
            Plan::mitarbeiters(),
            equal_bi(
                |a: &Schichtzuordnung| a.mitarbeiter_idx,
                |m: &Mitarbeiter| Some(m.index),
            ),
        ))
        .join((
            Plan::block_memberships(),
            equal_on(
                |row: &Pair<Leaf<'_, Schichtzuordnung>, Leaf<'_, Mitarbeiter>>| {
                    row.left.entity.schicht_id
                },
                |b: &Leaf<'_, BlockMembership>| b.entity.shift,
            ),
        ))
        .group_by(
            |_: &Schichtzuordnung, _: &Mitarbeiter, b: &BlockMembership| b.block,
            collect_vec(
                |(_, m, _): (&Schichtzuordnung, &Mitarbeiter, &BlockMembership)| m.schichtgruppe,
            ),
        )
        .penalize(|_: &usize, groups: &CollectedVec<i32>| {
            let mut seen = Vec::new();
            for group in groups.iter() {
                if !seen.contains(group) {
                    seen.push(*group);
                }
            }
            HardMediumSoftScore::of_medium(seen.len().saturating_sub(1) as i64)
        })
        .named("selber_block")
}
