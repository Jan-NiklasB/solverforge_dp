use crate::domain::{Mitarbeiter, Plan, Schichtzuordnung};
use solverforge::prelude::*;
use solverforge::stream::collector::count;
use solverforge::IncrementalConstraint;

/// MEDIUM: Verteilt die Schichten ueber ALLE Mitarbeiter - auch ueber jene ohne
/// jede Zuweisung (0 Schichten). Bestraft wird SUM(count_i^2) als MEDIUM.
///
/// Die Zuweisungen werden je Mitarbeiter gezaehlt und per `complement` um alle
/// Mitarbeiter ohne Zuweisung (Default 0) ergaenzt. Dadurch gehen auch untaetige
/// Mitarbeiter in die Bilanz ein. SUM(count_i^2) entspricht (bis auf eine
/// loesungsunabhaengige Konstante) SUM((count_i - Mittelwert)^2) und hat damit
/// dasselbe Optimum wie die Varianz. Nicht zugewiesene Schichtzuordnungen
/// (Key None) werden nicht bestraft.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardMediumSoftScore> {
    ConstraintFactory::<Plan, HardMediumSoftScore>::new()
        .for_each(Plan::schichtzuordnungs())
        .group_by(|sz: &Schichtzuordnung| sz.mitarbeiter_idx, count())
        .complement(
            Plan::mitarbeiters(),
            |m: &Mitarbeiter| Some(m.index),
            |_m: &Mitarbeiter| 0usize,
        )
        .penalize(|emp: &Option<usize>, cnt: &usize| {
            if emp.is_none() {
                HardMediumSoftScore::ZERO
            } else {
                HardMediumSoftScore::of_medium((*cnt as i64) * (*cnt as i64))
            }
        })
        .named("balancing")
}
