#[path = "../src/constraints/wachenwechsel.rs"]
mod wachenwechsel;

use chrono::{Datelike, Duration, NaiveDate};
use dp::data::{generate, DemoData};
use dp::domain;
use dp::domain::{Mitarbeiter, Plan, Schicht, Schichtzuordnung};
use solverforge::IncrementalConstraint;

fn schicht(index: usize, tag: i64, wache: &str) -> Schicht {
    let anchor = NaiveDate::from_ymd_opt(2030, 1, 1).unwrap();
    let rest = anchor.num_days_from_ce().rem_euclid(31);
    let base = anchor + Duration::days(((31 - rest) % 31) as i64);
    let start = (base + Duration::days(tag)).and_hms_opt(8, 0, 0).unwrap();
    let ende = (base + Duration::days(tag)).and_hms_opt(16, 0, 0).unwrap();
    Schicht::new(
        format!("s{index}"),
        index,
        "S".to_string(),
        start,
        ende,
        wache.to_string(),
        "S".to_string(),
        Vec::new(),
    )
}

fn bau(items: &[(&str, i64, usize)]) -> Plan {
    let anzahl = items.iter().map(|(_, _, m)| *m).max().unwrap_or(0) + 1;
    let mitarbeiters = (0..anzahl)
        .map(|i| {
            Mitarbeiter::new(
                format!("m{i}"),
                i,
                format!("M{i}"),
                "W1".to_string(),
                0,
                false,
                vec![Default::default()],
                false,
                100,
            )
        })
        .collect();
    let schichts: Vec<Schicht> = items
        .iter()
        .enumerate()
        .map(|(i, (wache, tag, _))| schicht(i, *tag, wache))
        .collect();
    let zuordnungen: Vec<Schichtzuordnung> = items
        .iter()
        .enumerate()
        .map(|(i, (_, _, m))| {
            let mut z = Schichtzuordnung::new(format!("z{i}"), i, Default::default());
            z.mitarbeiter_idx = Some(*m);
            z
        })
        .collect();
    Plan::new(mitarbeiters, schichts, zuordnungen, Vec::new())
}

#[test]
fn hoechstens_ein_wechsel_pro_31_tage() {
    let c = wachenwechsel::constraint();

    let einer = bau(&[("A", 0, 0), ("A", 1, 0), ("B", 2, 0), ("B", 3, 0)]);
    assert_eq!(c.evaluate(&einer).medium(), 0, "ein Wechsel erlaubt");

    let hin_und_zurueck = bau(&[("A", 0, 0), ("B", 1, 0), ("A", 2, 0)]);
    assert_eq!(
        c.evaluate(&hin_und_zurueck).medium(),
        -1,
        "A-B-A = 2 Wechsel, einer zu viel"
    );

    let drei = bau(&[("A", 0, 0), ("B", 1, 0), ("C", 2, 0)]);
    assert_eq!(c.evaluate(&drei).medium(), -1, "2 Wechsel");

    let vier = bau(&[("A", 0, 0), ("B", 1, 0), ("C", 2, 0), ("D", 3, 0)]);
    assert_eq!(c.evaluate(&vier).medium(), -2, "3 Wechsel, zwei zu viel");
}

#[test]
fn rollierendes_fenster() {
    let c = wachenwechsel::constraint();

    let einmal = bau(&[("A", 0, 0), ("B", 31, 0)]);
    assert_eq!(c.evaluate(&einmal).medium(), 0, "ein Wechsel");

    let genau_31_tage = bau(&[("A", 0, 0), ("B", 1, 0), ("C", 32, 0)]);
    assert_eq!(
        c.evaluate(&genau_31_tage).medium(),
        0,
        "Wechsel 31 Tage auseinander: erlaubt"
    );

    let unter_31_tagen = bau(&[("A", 0, 0), ("B", 1, 0), ("C", 31, 0)]);
    assert_eq!(
        c.evaluate(&unter_31_tagen).medium(),
        -1,
        "Wechsel 30 Tage auseinander: Strafe"
    );

    let abstain = bau(&[("A", 0, 0), ("B", 1, 0), ("C", 32, 0), ("D", 63, 0)]);
    assert_eq!(
        c.evaluate(&abstain).medium(),
        0,
        "drei Wechsel, jeweils 31 Tage Abstand"
    );
}

#[test]
fn mitarbeiter_getrennt() {
    let c = wachenwechsel::constraint();
    let plan = bau(&[
        ("A", 0, 0),
        ("B", 1, 0),
        ("A", 2, 0),
        ("C", 0, 1),
        ("C", 1, 1),
    ]);
    assert_eq!(c.evaluate(&plan).medium(), -1);
}

#[test]
fn unzugeordnet_zaehlt_nicht() {
    let mut plan = bau(&[("A", 0, 0), ("B", 1, 0)]);
    for z in plan.schichtzuordnungs.iter_mut() {
        z.mitarbeiter_idx = None;
    }
    assert_eq!(wachenwechsel::constraint().evaluate(&plan).medium(), 0);
}

#[test]
fn reihenfolge_egal() {
    let c = wachenwechsel::constraint();
    let mut plan = bau(&[("A", 0, 0), ("B", 1, 0), ("A", 2, 0)]);
    plan.schichtzuordnungs.reverse();
    assert_eq!(c.evaluate(&plan).medium(), -1);
}

#[test]
fn wachenwechsel_incremental_matches_full() {
    let mut plan: Plan = generate(DemoData::Small);
    let m = plan.mitarbeiters.len().max(1);
    let mut cs = wachenwechsel::constraint();
    let mut score = cs.initialize(&plan);
    assert_eq!(score, cs.evaluate(&plan));

    let n = plan.schichtzuordnungs.len();
    for _round in 0..2 {
        for i in 0..n {
            let old = plan.schichtzuordnungs[i].mitarbeiter_idx;
            let new = if old == Some(0) {
                None
            } else {
                Some(old.map(|x| x + 1).unwrap_or(0) % m)
            };
            score = score + cs.on_retract(&plan, i, 0);
            plan.schichtzuordnungs[i].mitarbeiter_idx = new;
            score = score + cs.on_insert(&plan, i, 0);
            assert_eq!(score, cs.evaluate(&plan), "i={i} old={old:?} new={new:?}");
            score = score + cs.on_retract(&plan, i, 0);
            plan.schichtzuordnungs[i].mitarbeiter_idx = old;
            score = score + cs.on_insert(&plan, i, 0);
        }
    }
}
