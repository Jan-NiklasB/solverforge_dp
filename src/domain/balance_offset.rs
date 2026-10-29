use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

/// Datenabhaengiger Offset fuer die `balancing`-Constraint.
///
/// Der Offset wird in `Plan::prepare()` aus der aktuellen Anzahl an
/// Schichtzuordnungen und Mitarbeitern berechnet und nie hartkodiert. Er
/// verschiebt die `SUM(count_i^2)`-Penalty so, dass eine perfekte
/// Gleichverteilung (alle Mitarbeiter gleich viele Einsaetze) den Score 0
/// ergibt. Der Offset ist waehrend einer Loesung konstant und beeinflusst
/// daher nur den absoluten Score, nicht die Optimierung.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct BalanceOffset {
    #[planning_id]
    pub id: usize,
    pub offset: i64,
}

/// Berechnet `n^2 / m` mit `n` = Anzahl Schichtzuordnungen und `m` = Anzahl
/// Mitarbeiter. Bei perfekter Gleichverteilung gilt `SUM(count_i^2) = n^2/m`;
/// der ganzzahlige Quotient ist dort exakt, weil `m` dann `n` teilt.
pub fn offsets(assignments: usize, employees: usize) -> Vec<BalanceOffset> {
    let offset = if employees == 0 {
        0
    } else {
        let n = assignments as i64;
        let m = employees as i64;
        (n * n) / m
    };
    vec![BalanceOffset { id: 0, offset }]
}
