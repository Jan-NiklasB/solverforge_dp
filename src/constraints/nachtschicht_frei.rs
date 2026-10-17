use crate::domain::{Plan, Schicht, Schichtzuordnung};
use solverforge::prelude::*;
use solverforge::stream::joiner::{equal_bi, equal_on};
use solverforge::IncrementalConstraint;
use solverforge::stream::relational::Leaf;
use solverforge::stream::relational::operator::Pair;

type AnchorRow<'a> = Pair<Leaf<'a, Schichtzuordnung>, Leaf<'a, Schicht>>;
type TriRow<'a> = Pair<AnchorRow<'a>, Leaf<'a, Schichtzuordnung>>;

/// MEDIUM: Auf eine Nachtschicht muss ein arbeitsfreier Tag folgen
/// (42 h Arbeitsruhe). Beginnt innerhalb von 42 h nach Ende der Nachtschicht
/// ein *Tagdienst* desselben Mitarbeiters, wird mit ONE_MEDIUM bestraft.
/// Eine Nachtschicht am Folgetag ist ausdruecklich zulaessig.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardMediumSoftScore> {
    ConstraintFactory::<Plan, HardMediumSoftScore>::new()
        .for_each(Plan::schichtzuordnungs())
        .join((
            Plan::schichts(),
            equal_bi(|sz: &Schichtzuordnung| sz.schicht_id, |s: &Schicht| s.index),
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
                    && sz1.id != sz2.id
                    && is_nachtschicht(s1)
                    && is_tagdienst(s2)
                    && s2.start >= s1.ende
                    && s2.start < s1.ende + chrono::Duration::hours(42)
            },
        )
        .penalize(HardMediumSoftScore::ONE_MEDIUM)
        .named("42h nach Nachtschicht")
}

fn sz2_schicht_id(row: &TriRow<'_>) -> usize {
    row.right.entity.schicht_id
}

fn leaf_schicht_index(s: &Leaf<'_, Schicht>) -> usize {
    s.index
}

fn is_nachtschicht(s: &Schicht) -> bool {
    s.ende.date() > s.start.date()
}

fn is_tagdienst(s: &Schicht) -> bool {
    s.ende.date() == s.start.date()
}
