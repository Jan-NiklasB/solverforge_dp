#[path = "../src/constraints/max_60h.rs"]
mod max_60h;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use dp::domain;
use dp::domain::{Mitarbeiter, Plan, Schicht, Schichtzuordnung};
use solverforge::IncrementalConstraint;

fn base() -> NaiveDateTime {
    NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2026, 10, 1).unwrap(),
        NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
    )
}

fn mkemp(i: usize) -> Mitarbeiter {
    Mitarbeiter::new(
        format!("m{i}"),
        i,
        format!("m{i}"),
        "Station 40".to_string(),
        0,
        false,
        vec![],
        false,
        100,
    )
}

fn mkschicht(i: usize, start_day: i64) -> Schicht {
    let start = base() + chrono::Duration::days(start_day);
    Schicht::new(
        format!("s{i}"),
        i,
        format!("s{i}"),
        start,
        start + chrono::Duration::hours(12),
        "Station 40".to_string(),
        vec![],
    )
}

fn mkzuordnung(i: usize, emp: Option<usize>) -> Schichtzuordnung {
    Schichtzuordnung {
        id: format!("z{i}"),
        schicht_id: i,
        qualifikation: Default::default(),
        mitarbeiter_idx: emp,
    }
}

fn mkplan(days: &[i64], emp: Option<usize>) -> Plan {
    let schichts = days
        .iter()
        .enumerate()
        .map(|(i, day)| mkschicht(i, *day))
        .collect();
    let zuordnungen = days
        .iter()
        .enumerate()
        .map(|(i, _)| mkzuordnung(i, emp))
        .collect();
    Plan::new(vec![mkemp(0)], schichts, zuordnungen, Vec::new())
}

#[test]
fn max_60h_scores() {
    let cs = max_60h::constraint();

    // 6 Schichten à 12h in 7 Tagen -> 72h -> ein Verstoß
    assert_eq!(cs.evaluate(&mkplan(&[0, 1, 2, 3, 4, 5], Some(0))).hard(), -1);

    // 5 Schichten à 12h in 7 Tagen -> 60h -> kein Verstoß
    assert_eq!(cs.evaluate(&mkplan(&[0, 1, 2, 3, 4], Some(0))).hard(), 0);

    // Fenster ist [start, start + 7 Tage): Tag 7 liegt außerhalb
    assert_eq!(cs.evaluate(&mkplan(&[0, 7], Some(0))).hard(), 0);

    // Ohne Zuordnung keine Prüfung
    assert_eq!(cs.evaluate(&mkplan(&[0, 1, 2, 3, 4, 5], None)).hard(), 0);
}
