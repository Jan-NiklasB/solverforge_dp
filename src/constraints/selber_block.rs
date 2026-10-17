use chrono::NaiveDateTime;
use crate::domain::{Plan, Schichtzuordnung, Mitarbeiter, Schicht};
use solverforge::prelude::*;
use solverforge::stream::joiner::{equal_bi, equal_on};
use solverforge::IncrementalConstraint;
use solverforge::stream::relational::Leaf;
use solverforge::stream::relational::operator::Pair;

/// MEDIUM: An einem Standort (Wache) müssen alle zeitgleich (zeitlich
/// überlappend) arbeitenden Mitarbeiter derselben Schichtgruppe angehören.
/// Geprüft wird je Wache und Zeitfenster. Sind mehrere Schichtgruppen
/// anwesend, wird für jede weitere Schichtgruppe eine Medium-Penalty vergeben.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardMediumSoftScore> {
    ConstraintFactory::<Plan, HardMediumSoftScore>::new()
        // Anker: Schicht. Join auf dieselbe Wache (Schluessel statt Kreuzprodukt).
        .for_each(Plan::schichts())
        .join((
            Plan::schichts(),
            equal_bi(
                |s1: &Schicht| (s1.wache.clone(), s1.start.date()),
                |s2: &Schicht| (s2.wache.clone(), s2.start.date()),
            ),
        ))
        // Zuordnung der zweiten Schicht
        .join((
            Plan::schichtzuordnungs(),
            equal_on(sz2_schicht_id, sz_schicht_id),
        ))
        // Mitarbeiter der zweiten Zuordnung (liefert die Schichtgruppe)
        .join((
            Plan::mitarbeiters(),
            equal_on(sz2_mitarbeiter_idx, leaf_mitarbeiter_index),
        ))
        .filter(
            |s1: &Schicht,
             s2: &Schicht,
             sz2: &Schichtzuordnung,
             _m2: &Mitarbeiter| {
                sz2.mitarbeiter_idx.is_some()
                    && s1.start < s2.ende
                    && s2.start < s1.ende
                    && (s1.start < s2.start
                        || (s1.start == s2.start && s1.id < s2.id))
            },
        )
        .group_by(
            |s1: &Schicht,
             _s2: &Schicht,
             _sz2: &Schichtzuordnung,
             _m2: &Mitarbeiter| (s1.wache.clone(), s1.start),
            collect_vec(
                |(_s1, _s2, _sz2, m2): (&Schicht, &Schicht, &Schichtzuordnung, &Mitarbeiter)| {
                    m2.schichtgruppe
                },
            ),
        )
        .penalize(|_key: &(String, NaiveDateTime), sgv: &CollectedVec<i32>| {
            let mut seen: Vec<i32> = Vec::new();
            for sg in sgv.iter() {
                if !seen.contains(sg) {
                    seen.push(*sg);
                }
            }
            HardMediumSoftScore::of_medium(seen.len().saturating_sub(1) as i64)
        })
        .named("selber_block")
}

type ShiftPair<'a> = Pair<Leaf<'a, Schicht>, Leaf<'a, Schicht>>;
type SzRow<'a> = Pair<ShiftPair<'a>, Leaf<'a, Schichtzuordnung>>;

fn sz2_schicht_id(row: &ShiftPair<'_>) -> usize {
    row.right.entity.index
}

fn sz_schicht_id(sz: &Leaf<'_, Schichtzuordnung>) -> usize {
    sz.entity.schicht_id
}

fn sz2_mitarbeiter_idx(row: &SzRow<'_>) -> Option<usize> {
    row.right.entity.mitarbeiter_idx
}

fn leaf_mitarbeiter_index(m: &Leaf<'_, Mitarbeiter>) -> Option<usize> {
    Some(m.index)
}
