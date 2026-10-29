use crate::domain::{BalanceOffset, Plan};
use solverforge::prelude::*;
use solverforge::IncrementalConstraint;

/// SOFT: Verschiebt die `balancing`-Penalty (`SUM(count_i^2)`) um den
/// datenabhaengig berechneten Offset `n^2 / m` (siehe [`BalanceOffset`]).
///
/// Dadurch ergibt eine perfekte Gleichverteilung den Score 0. Der Offset ist
/// waehrend einer Loesung konstant, aendert also die Optimierung nicht, sondern
/// nur den absoluten Score. Er wird nicht hartkodiert, sondern in
/// `Plan::prepare()` aus der Anzahl an Schichtzuordnungen und Mitarbeitern
/// bestimmt.
pub fn constraint() -> impl IncrementalConstraint<Plan, HardMediumSoftScore> {
    ConstraintFactory::<Plan, HardMediumSoftScore>::new()
        .for_each(Plan::balance_offset())
        .reward(|offset: &BalanceOffset| HardMediumSoftScore::of_soft(offset.offset))
        .named("balancing_offset")
}
