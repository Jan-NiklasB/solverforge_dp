use chrono::Datelike;
use crate::domain::{Mitarbeiter, Plan, Schicht, Schichtzuordnung};
use solverforge::prelude::*;
use solverforge::stream::joiner::equal_bi;

/// MEDIUM: Fasst die Einsatztage je Mitarbeiter zu zusammenhaengenden
/// Dienstbloecken (aufeinanderfolgende Kalendertage) zusammen. Bestraft wird die
/// Anzahl der Blockunterbrechungen (Anzahl Bloecke minus eins).
///
/// SOFT: Belohnt jeden Mitarbeiter mit mindestens einer vollstaendig freien
/// Woche, d.h. mindestens sieben aufeinanderfolgenden freien Kalendertagen
/// zwischen zwei Dienbloecken.
///
/// Die Zuordnungen werden ueber ihre Schicht auf den Einsatztag abgebildet und
/// je Mitarbeiter als eindeutige Tagesmenge gesammelt. Nicht zugewiesene
/// Zuordnungen (Schluessel None) werden nicht bewertet. Mitarbeiter ohne
/// Einsaetze werden per `complement` ergaenzt und erhalten weder Strafe noch
/// Belohnung.
pub fn constraint() -> impl ConstraintSet<Plan, HardMediumSoftScore> {
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
            indexed_presence(|(_sz, s): (&Schichtzuordnung, &Schicht)| {
                s.start.date().num_days_from_ce() as i64
            }),
        )
        .complement(
            Plan::mitarbeiters(),
            |m: &Mitarbeiter| Some(m.index),
            |_m: &Mitarbeiter| IndexedPresence::default(),
        )
        .reward(|emp: &Option<usize>, p: &IndexedPresence| {
            if emp.is_some() && hat_freie_woche(p) {
                HardMediumSoftScore::of_soft(1)
            } else {
                HardMediumSoftScore::ZERO
            }
        })
        .named("dienst_block_freie_woche")
        .penalize(|emp: &Option<usize>, p: &IndexedPresence| {
            if emp.is_none() {
                HardMediumSoftScore::ZERO
            } else {
                HardMediumSoftScore::of_medium(block_unterbrechungen(p))
            }
        })
        .named("dienst_block")
}

fn block_unterbrechungen(p: &IndexedPresence) -> i64 {
    p.runs().len().saturating_sub(1) as i64
}

fn hat_freie_woche(p: &IndexedPresence) -> bool {
    let runs = p.runs();
    runs.runs()
        .windows(2)
        .any(|w| w[1].start() - w[0].end() - 1 >= 7)
}
