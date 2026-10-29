#[path = "../src/constraints/fahrzeugwechsel.rs"]
mod fahrzeugwechsel;

use chrono::{Duration, NaiveDate};
use dp::data::{generate, DemoData};
use dp::domain;
use dp::domain::{Mitarbeiter, Plan, Schicht, Schichtzuordnung};
use solverforge::IncrementalConstraint;

fn schicht(index: usize, tag: i64, name: &str) -> Schicht {
    let base = NaiveDate::from_ymd_opt(2030, 1, 1).unwrap();
    let start = (base + Duration::days(tag)).and_hms_opt(8, 0, 0).unwrap();
    let ende = (base + Duration::days(tag)).and_hms_opt(16, 0, 0).unwrap();
    Schicht::new(
        format!("s{index}"),
        index,
        name,
        start,
        ende,
        "W1".to_string(),
        name.to_string(),
        Vec::new(),
    )
}

fn zuordnung(id: usize, schicht_id: usize, mitarbeiter: Option<usize>) -> Schichtzuordnung {
    let mut z = Schichtzuordnung::new(format!("z{id}"), schicht_id, Default::default());
    z.mitarbeiter_idx = mitarbeiter;
    z
}

fn bau(names: &[(&str, i64)]) -> Plan {
    let mitarbeiter = Mitarbeiter::new(
        "m0",
        0,
        "M0",
        "W1".to_string(),
        0,
        false,
        vec![Default::default()],
        false,
        100,
    );
    let schichts: Vec<Schicht> = names
        .iter()
        .enumerate()
        .map(|(i, (name, tag))| schicht(i, *tag, name))
        .collect();
    let zuordnungen: Vec<Schichtzuordnung> =
        (0..names.len()).map(|i| zuordnung(i, i, Some(0))).collect();
    Plan::new(vec![mitarbeiter], schichts, zuordnungen, Vec::new())
}

#[test]
fn wechsel_nur_im_dienstblock() {
    let c = fahrzeugwechsel::constraint();

    let gleich = bau(&[("A", 0), ("A", 1), ("A", 2)]);
    assert_eq!(c.evaluate(&gleich).soft(), 0, "gleicher Name: kein Wechsel");

    let wechsel_block = bau(&[("A", 0), ("A", 1), ("B", 2)]);
    assert_eq!(c.evaluate(&wechsel_block).soft(), -1, "Wechsel im Block");

    let grenze = bau(&[("A", 0), ("A", 1), ("B", 2), ("C", 5)]);
    assert_eq!(c.evaluate(&grenze).soft(), -1, "Wechsel ueber Blockgrenze frei");

    let grenze_kein_block = bau(&[("A", 0), ("A", 1), ("B", 2), ("C", 3)]);
    assert_eq!(
        c.evaluate(&grenze_kein_block).soft(),
        -2,
        "ein Block, zwei Wechsel"
    );
}

#[test]
fn unzugeordnet_zaehlt_nicht() {
    let mut plan = bau(&[("A", 0), ("B", 1)]);
    for z in plan.schichtzuordnungs.iter_mut() {
        z.mitarbeiter_idx = None;
    }
    assert_eq!(fahrzeugwechsel::constraint().evaluate(&plan).soft(), 0);
}

#[test]
fn reihenfolge_egal() {
    let c = fahrzeugwechsel::constraint();
    let mut plan = bau(&[("A", 0), ("B", 1), ("C", 2)]);
    plan.schichtzuordnungs.reverse();
    assert_eq!(c.evaluate(&plan).soft(), -2);
}

#[test]
fn fahrzeug_entscheidet_nicht_der_name() {
    let c = fahrzeugwechsel::constraint();

    let mut gleicher_name = bau(&[("A", 0), ("A", 1)]);
    gleicher_name.schichts[1].fahrzeug = "B".to_string();
    assert_eq!(
        c.evaluate(&gleicher_name).soft(),
        -1,
        "gleicher Name, anderes Fahrzeug: Wechsel"
    );

    let mut gleiches_fahrzeug = bau(&[("A", 0), ("B", 1)]);
    gleiches_fahrzeug.schichts[1].fahrzeug = "A".to_string();
    assert_eq!(
        c.evaluate(&gleiches_fahrzeug).soft(),
        0,
        "anderer Name, gleiches Fahrzeug: kein Wechsel"
    );
}

#[test]
fn fahrzeugwechsel_incremental_matches_full() {
    let mut plan: Plan = generate(DemoData::Small);
    let m = plan.mitarbeiters.len().max(1);
    let mut cs = fahrzeugwechsel::constraint();
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
