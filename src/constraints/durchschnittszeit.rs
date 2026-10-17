use crate::domain::{Plan, Schichtzuordnung, ShiftMinutes};
use solverforge::IncrementalConstraint;
use solverforge::prelude::*;
use solverforge::stream::joiner::equal_bi;

pub fn constraint() -> impl IncrementalConstraint<Plan, HardMediumSoftScore> {
    ConstraintFactory::<Plan, HardMediumSoftScore>::new()
        .for_each(Plan::schichtzuordnungs())
        .filter(|a: &Schichtzuordnung| a.mitarbeiter_idx.is_some())
        .join((
            Plan::shift_minutes(),
            equal_bi(|a: &Schichtzuordnung| a.schicht_id, |s: &ShiftMinutes| s.id),
        ))
        .group_by(
            |a: &Schichtzuordnung, s: &ShiftMinutes| (a.mitarbeiter_idx, s.average_limit),
            sum(|(_, s): (&Schichtzuordnung, &ShiftMinutes)| s.minutes),
        )
        .penalize(|(_, limit): &(Option<usize>, i64), minutes: &i64| {
            HardMediumSoftScore::of_medium(((*minutes - limit).max(0) + 59) / 60)
        })
        .named("durchschnittszeit")
}
