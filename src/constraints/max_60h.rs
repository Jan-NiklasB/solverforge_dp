use crate::domain::{Mitarbeiter, Plan, Schichtzuordnung, WindowMinutes};
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
            Plan::window_minutes(),
            equal_on(
                |row: &Pair<Leaf<'_, Schichtzuordnung>, Leaf<'_, Mitarbeiter>>| {
                    row.left.entity.schicht_id
                },
                |w: &Leaf<'_, WindowMinutes>| w.entity.shift,
            ),
        ))
        .group_by(
            |_: &Schichtzuordnung, m: &Mitarbeiter, w: &WindowMinutes| (m.index, w.window),
            sum(|(_, _, w): (&Schichtzuordnung, &Mitarbeiter, &WindowMinutes)| w.minutes),
        )
        .penalize(hard_weight(|_: &(usize, usize), minutes: &i64| {
            if *minutes > 60 * 60 {
                HardMediumSoftScore::ONE_HARD
            } else {
                HardMediumSoftScore::ZERO
            }
        }))
        .named("60h_max")
}
