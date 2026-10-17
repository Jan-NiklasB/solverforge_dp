use chrono::NaiveDateTime;
use crate::domain::{Plan, Schicht, Schichtzuordnung};
use solverforge::prelude::*;
use solverforge::stream::joiner::{equal_bi, equal_on};
use solverforge::IncrementalConstraint;
use solverforge::stream::relational::Leaf;
use solverforge::stream::relational::operator::Pair;

/// HARD: Ein Mitarbeiter darf innerhalb von 7 Tagen nicht mehr als 60 Stunden
/// arbeiten. Für jedes 7-Tage-Fenster (Anker = jeweiliger Schichtbeginn) wird
/// pro Mitarbeiter die Summe der Schichtdauern gebildet und bei Überschreitung
/// mit ONE_HARD bestraft.
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
        .join((
            Plan::schichtzuordnungs(),
            equal_on(
                |row: &AnchorRow<'_>| row.left.entity.mitarbeiter_idx,
                |sz: &Leaf<'_, Schichtzuordnung>| sz.entity.mitarbeiter_idx,
            ),
        ))
        .join((
            Plan::schichts(),
            equal_on(sz2_schicht_id, leaf_schicht_index),
        ))
        .filter(
            |sz1: &Schichtzuordnung, s1: &Schicht, sz2: &Schichtzuordnung, s2: &Schicht| {
                sz1.mitarbeiter_idx.is_some()
                    && sz1.mitarbeiter_idx == sz2.mitarbeiter_idx
                    && s2.start < s1.start + chrono::Duration::days(7)
                    && s2.ende > s1.start
            },
        )
        .group_by(
            |sz1: &Schichtzuordnung, s1: &Schicht, _sz2: &Schichtzuordnung, _s2: &Schicht| {
                (sz1.mitarbeiter_idx, s1.start)
            },
            collect_vec(
                |(_sz1, _s1, _sz2, s2): (
                    &Schichtzuordnung,
                    &Schicht,
                    &Schichtzuordnung,
                    &Schicht,
                )| (s2.ende - s2.start).num_minutes(),
            ),
        )
        .penalize(
            |_key: &(Option<usize>, NaiveDateTime), minutes: &CollectedVec<i64>| {
                let total: i64 = minutes.iter().copied().sum();
                if total > 60 * 60 {
                    HardMediumSoftScore::ONE_HARD
                } else {
                    HardMediumSoftScore::ZERO
                }
            },
        )
        .named("60h_max")
}

type AnchorRow<'a> = Pair<Leaf<'a, Schichtzuordnung>, Leaf<'a, Schicht>>;
type TriRow<'a> = Pair<AnchorRow<'a>, Leaf<'a, Schichtzuordnung>>;

fn sz2_schicht_id(row: &TriRow<'_>) -> usize {
    row.right.entity.schicht_id
}

fn leaf_schicht_index(s: &Leaf<'_, Schicht>) -> usize {
    s.index
}
