use chrono::{Duration, NaiveDateTime};
use crate::domain::{Plan, Schicht, Schichtzuordnung};
use solverforge::prelude::*;
use solverforge::stream::joiner::equal_bi;
use solverforge::IncrementalConstraint;

/// MEDIUM: Erlaubt je Mitarbeiter hoechstens EINEN Wachenwechsel (Standort) in
/// einem rollierenden 31-Tage-Fenster.
///
/// Die Einsaetze werden je Mitarbeiter nach `Schicht.start` sortiert; benachbarte
/// Einsaetze mit unterschiedlicher `Schicht.wache` sind Wechsel. Liegen zwei
/// aufeinanderfolgende Wechsel weniger als 31 Tage auseinander, kostet das je
/// Paar 1 MEDIUM. (Aequivalent zu: in keinem 31-Tage-Fenster mehr als ein
/// Wechsel.) Nicht zugewiesene Zuordnungen (Schluessel None) werden nicht
/// bewertet.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardMediumSoftScore> {
    ConstraintFactory::<Plan, HardMediumSoftScore>::new()
        .for_each(Plan::schichtzuordnungs())
        .join((
            Plan::schichts(),
            equal_bi(
                |sz: &Schichtzuordnung| sz.schicht_id,
                |s: &Schicht| s.index,
            ),
        ))
        .group_by(
            |sz: &Schichtzuordnung, _s: &Schicht| sz.mitarbeiter_idx,
            collect_vec(|(_sz, s): (&Schichtzuordnung, &Schicht)| {
                (s.start, s.index, s.wache.clone())
            }),
        )
        .penalize(
            |emp: &Option<usize>, seq: &CollectedVec<(NaiveDateTime, usize, String)>| {
                if emp.is_none() {
                    HardMediumSoftScore::ZERO
                } else {
                    HardMediumSoftScore::of_medium(wechsel_im_fenster(seq))
                }
            },
        )
        .named("wachenwechsel")
}

fn wechsel_im_fenster(seq: &CollectedVec<(NaiveDateTime, usize, String)>) -> i64 {
    let mut v: Vec<&(NaiveDateTime, usize, String)> = seq.iter().collect();
    v.sort_by_key(|x| (x.0, x.1));
    let mut voriger_wechsel: Option<NaiveDateTime> = None;
    let mut strafe = 0i64;
    for w in v.windows(2) {
        if w[0].2 != w[1].2 {
            if let Some(vorher) = voriger_wechsel {
                if w[1].0 - vorher < Duration::days(31) {
                    strafe += 1;
                }
            }
            voriger_wechsel = Some(w[1].0);
        }
    }
    strafe
}
