#[path = "../src/constraints/shift_assigned.rs"]
mod shift_assigned;
#[path = "../src/constraints/balancing.rs"]
mod balancing;
#[path = "../src/constraints/ueberlappende_schichten.rs"]
mod ueberlappende_schichten;
#[path = "../src/constraints/ruhezeit.rs"]
mod ruhezeit;
#[path = "../src/constraints/selber_block.rs"]
mod selber_block;
#[path = "../src/constraints/max_60h.rs"]
mod max_60h;
#[path = "../src/constraints/nachtschicht_frei.rs"]
mod nachtschicht_frei;
#[path = "../src/constraints/durchschnittszeit.rs"]
mod durchschnittszeit;

use dp::data::{generate, DemoData};
use dp::domain;
use dp::domain::Plan;
use solverforge::{HardMediumSoftScore, IncrementalConstraint};

fn check<C>(name: &str, mut cs: C, plan: &mut Plan, m: usize)
where
    C: IncrementalConstraint<Plan, HardMediumSoftScore>,
{
    let mut score = cs.initialize(plan);
    let n = plan.schichtzuordnungs.len();
    let mut mismatches = 0u32;
    for _round in 0..2 {
        for i in 0..n {
            let old = plan.schichtzuordnungs[i].mitarbeiter_idx;
            let new = if old == Some(0) {
                None
            } else {
                Some((old.map(|x| x + 1).unwrap_or(0)) % m)
            };

            score = score + cs.on_retract(plan, i, 0);
            plan.schichtzuordnungs[i].mitarbeiter_idx = new;
            score = score + cs.on_insert(plan, i, 0);

            let full = cs.evaluate(plan);
            if score != full {
                mismatches += 1;
                if mismatches <= 3 {
                    println!("  {name} i={i} old={old:?} new={new:?} incr={score} full={full}");
                }
            }

            score = score + cs.on_retract(plan, i, 0);
            plan.schichtzuordnungs[i].mitarbeiter_idx = old;
            score = score + cs.on_insert(plan, i, 0);
        }
    }
    println!("{name:28} mismatches={mismatches}");
    assert_eq!(mismatches, 0, "{name}: incremental != full evaluation");
}

#[test]
fn incremental_matches_full() {
    let m: usize;
    {
        let mut plan = generate(DemoData::Standard);
        m = plan.mitarbeiters.len();
        let _ = &mut plan;
    }
    let mut plan = generate(DemoData::Standard);
    check("shift_assigned", shift_assigned::constraint(), &mut plan, m);
    check("balancing", balancing::constraint(), &mut plan, m);
    check("ueberlappende_schichten", ueberlappende_schichten::constraint(), &mut plan, m);
    check("ruhezeit", ruhezeit::constraint(), &mut plan, m);
    check("selber_block", selber_block::constraint(), &mut plan, m);
    check("max_60h", max_60h::constraint(), &mut plan, m);
    check("nachtschicht_frei", nachtschicht_frei::constraint(), &mut plan, m);
    check("durchschnittszeit", durchschnittszeit::constraint(), &mut plan, m);
}
