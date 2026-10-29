use chrono::NaiveDateTime;
use crate::domain::{Plan, Schicht, Schichtzuordnung};
use solverforge::prelude::*;
use solverforge::stream::joiner::equal_bi;
use solverforge::IncrementalConstraint;

/// SOFT: Bestraft Fahrzeugwechsel (Wechsel des Fahrzeugs) innerhalb eines
/// Dienstblocks. Ein Dienstblock ist eine zusammenhaengende Folge von
/// Arbeitstagen eines Mitarbeiters (fehlender Kalendertag trennt Bloecke, wie in
/// `dienst_block`). Die Einsaetze werden je Mitarbeiter nach `Schicht.start`
/// sortiert; jedes zeitlich benachbarte Paar im SELBEN Dienstblock mit
/// unterschiedlichem `Schicht.fahrzeug` kostet 1 SOFT. Wechsel ueber eine
/// Blockgrenze werden nicht bestraft. Nicht zugewiesene Zuordnungen
/// (Schluessel None) werden nicht bewertet.
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
                (s.start, s.index, s.fahrzeug.clone())
            }),
        )
        .penalize(
            |emp: &Option<usize>, seq: &CollectedVec<(NaiveDateTime, usize, String)>| {
                if emp.is_none() {
                    HardMediumSoftScore::ZERO
                } else {
                    HardMediumSoftScore::of_soft(wechsel_im_block(seq))
                }
            },
        )
        .named("fahrzeugwechsel")
}

fn wechsel_im_block(seq: &CollectedVec<(NaiveDateTime, usize, String)>) -> i64 {
    let mut v: Vec<&(NaiveDateTime, usize, String)> = seq.iter().collect();
    v.sort_by_key(|x| (x.0, x.1));
    v.windows(2)
        .filter(|w| {
            let tag_differenz = (w[1].0.date() - w[0].0.date()).num_days();
            tag_differenz <= 1 && w[0].2 != w[1].2
        })
        .count() as i64
}
