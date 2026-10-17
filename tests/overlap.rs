use chrono::NaiveDate;
#[path = "../src/constraints/ueberlappende_schichten.rs"]
mod overlap;
use dp::domain;
use dp::domain::{Mitarbeiter, Plan, Schicht, Schichtzuordnung};
use solverforge::IncrementalConstraint;

fn dt(h: u32) -> chrono::NaiveDateTime {
    NaiveDate::from_ymd_opt(2026, 10, 12)
        .unwrap()
        .and_hms_opt(h, 0, 0)
        .unwrap()
}

fn mkplan(s0: (u32, u32), s1: (u32, u32), same_emp: bool) -> Plan {
    let m = vec![
        Mitarbeiter::new("m0", 0, "a", "w".to_string(), 0, false, vec![], false, 100),
        Mitarbeiter::new("m1", 1, "b", "w".to_string(), 0, false, vec![], false, 100),
    ];
    let s = vec![
        Schicht::new("s0", 0, "s0", dt(s0.0), dt(s0.1), "w".to_string(), vec![]),
        Schicht::new("s1", 1, "s1", dt(s1.0), dt(s1.1), "w".to_string(), vec![]),
    ];
    let z = vec![
        Schichtzuordnung {
            id: "z0".to_string(),
            schicht_id: 0,
            qualifikation: Default::default(),
            mitarbeiter_idx: Some(0),
        },
        Schichtzuordnung {
            id: "z1".to_string(),
            schicht_id: 1,
            qualifikation: Default::default(),
            mitarbeiter_idx: Some(if same_emp { 0 } else { 1 }),
        },
    ];
    Plan::new(m, s, z, Vec::new())
}

#[test]
fn overlap_scores() {
    let cs = overlap::constraint();
    assert_eq!(cs.evaluate(&mkplan((8, 16), (12, 20), true)).hard(), -1);
    assert_eq!(cs.evaluate(&mkplan((8, 12), (14, 20), true)).hard(), 0);
    assert_eq!(cs.evaluate(&mkplan((8, 16), (12, 20), false)).hard(), 0);
    assert_eq!(cs.evaluate(&mkplan((8, 12), (12, 20), true)).hard(), 0);
    let mut plan = mkplan((8, 16), (12, 20), true);
    plan.schichtzuordnungs
        .iter_mut()
        .for_each(|z| z.mitarbeiter_idx = None);
    assert_eq!(cs.evaluate(&plan).hard(), 0);
    let mut cs = overlap::constraint();
    let mut plan = mkplan((8, 16), (12, 20), true);
    let mut score = cs.initialize(&plan);
    assert_eq!(score.hard(), -1);
    score = score + cs.on_retract(&plan, 1, 0);
    plan.schichtzuordnungs[1].mitarbeiter_idx = Some(1);
    score = score + cs.on_insert(&plan, 1, 0);
    assert_eq!(score, cs.evaluate(&plan));
    assert_eq!(score.hard(), 0);
    score = score + cs.on_retract(&plan, 1, 0);
    plan.schichtzuordnungs[1].mitarbeiter_idx = Some(0);
    score = score + cs.on_insert(&plan, 1, 0);
    assert_eq!(score, cs.evaluate(&plan));
    assert_eq!(score.hard(), -1);
}
