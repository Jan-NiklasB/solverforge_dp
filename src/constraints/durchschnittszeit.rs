use chrono::Datelike;
use crate::domain::{Plan, Schicht, Schichtzuordnung};
use solverforge::prelude::*;
use solverforge::stream::collector::{load_balance, LoadBalance};
use solverforge::stream::joiner::equal_bi;
use solverforge::IncrementalConstraint;

/// MEDIUM: Die durchschnittliche Wochenarbeitszeit je Mitarbeiter darf ueber
/// den gesamten Zeitraum 43 h/Woche nicht ueberschreiten.
///
/// Bezugsgroesse sind ALLE Kalenderwochen (ISO) des Zeitraums, abgeleitet aus
/// den tatsaechlich belegten Wochen aller Schichten. Je Mitarbeiter wird die
/// Gesamtstundenzahl durch die Anzahl dieser Wochen geteilt; bestraft wird die
/// ueberschreitende Stundenzahl (aufgerundet) als MEDIUM.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardMediumSoftScore> {
    ConstraintFactory::<Plan, HardMediumSoftScore>::new()
        .for_each(Plan::schichtzuordnungs())
        .join((
            Plan::schichts(),
            equal_bi(|sz: &Schichtzuordnung| sz.schicht_id, |s: &Schicht| s.index),
        ))
        .group_by(
            |_sz: &Schichtzuordnung, _s: &Schicht| 0usize,
            load_balance(
                |(sz, s): (&Schichtzuordnung, &Schicht)| {
                    let kw = s.start.date().iso_week();
                    (sz.mitarbeiter_idx, (kw.year(), kw.week()))
                },
                |(_sz, s): (&Schichtzuordnung, &Schicht)| (s.ende - s.start).num_minutes(),
            ),
        )
        .penalize(
            |_key: &usize, lb: &LoadBalance<(Option<usize>, (i32, u32))>| {
                let loads = lb.loads();

                let mut weeks: Vec<(i32, u32)> = Vec::with_capacity(32);
                let mut max_employee = 0usize;
                for ((emp, week), _) in loads.iter() {
                    weeks.push(*week);
                    if let Some(employee) = emp {
                        max_employee = max_employee.max(*employee);
                    }
                }
                weeks.sort_unstable();
                weeks.dedup();
                let limit = 43 * 60 * weeks.len().max(1) as i64;

                let mut totals = vec![0i64; max_employee + 1];
                for ((emp, _), minutes) in loads.iter() {
                    if let Some(employee) = emp {
                        totals[*employee] += *minutes;
                    }
                }

                let penalty: i64 = totals
                    .iter()
                    .map(|total| ((total - limit).max(0) + 59) / 60)
                    .sum();
                HardMediumSoftScore::of_medium(penalty)
            },
        )
        .named("durchschnittszeit")
}
