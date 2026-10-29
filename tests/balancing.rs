use chrono::NaiveDate;
#[path = "../src/constraints/balancing.rs"]
mod balancing;
#[path = "../src/constraints/balancing_offset.rs"]
mod balancing_offset;
use dp::domain;
use dp::domain::{Mitarbeiter, Plan, Schicht, Schichtzuordnung};
use solverforge::IncrementalConstraint;

fn dt(h: u32) -> chrono::NaiveDateTime {
    NaiveDate::from_ymd_opt(2026, 10, 12)
        .unwrap()
        .and_hms_opt(h, 0, 0)
        .unwrap()
}

fn mkplan(assignment: &[usize]) -> Plan {
    let m = (0..assignment.len())
        .map(|i| {
            Mitarbeiter::new(
                format!("m{i}"),
                i,
                format!("m{i}"),
                "w".to_string(),
                0,
                false,
                vec![Default::default()],
                false,
                100,
            )
        })
        .collect();
    let s = (0..assignment.len())
        .map(|i| {
            Schicht::new(
                format!("s{i}"),
                i,
                format!("s{i}"),
                dt(6),
                dt(18),
                "w".to_string(),
                String::new(),
                vec![],
            )
        })
        .collect();
    let z = assignment
        .iter()
        .enumerate()
        .map(|(i, &emp)| Schichtzuordnung {
            id: format!("z{i}"),
            schicht_id: i,
            qualifikation: Default::default(),
            mitarbeiter_idx: Some(emp),
        })
        .collect();
    Plan::new(m, s, z, Vec::new())
}

fn combined_soft(plan: &Plan) -> i64 {
    balancing::constraint().evaluate(plan).soft()
        + balancing_offset::constraint().evaluate(plan).soft()
}

#[test]
fn perfect_distribution_scores_zero() {
    assert_eq!(combined_soft(&mkplan(&[0, 1, 2, 3])), 0);
}

#[test]
fn employees_without_shifts_are_included() {
    let spread = combined_soft(&mkplan(&[0, 0, 1, 1]));
    let concentrated = combined_soft(&mkplan(&[0, 0, 0, 0]));
    assert!(spread < 0);
    assert!(concentrated < spread);
}

#[test]
fn incremental_matches_full_evaluation() {
    let mut cs = balancing::constraint();
    let mut plan = mkplan(&[0, 0, 1, 1]);
    let mut score = cs.initialize(&plan);
    score = score + cs.on_retract(&plan, 1, 0);
    plan.schichtzuordnungs[1].mitarbeiter_idx = Some(2);
    score = score + cs.on_insert(&plan, 1, 0);
    assert_eq!(score, cs.evaluate(&plan));
}
