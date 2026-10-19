#[path = "../src/constraints/dienst_block.rs"]
mod dienst_block;

use dp::data::{generate, DemoData};
use dp::domain;
use dp::domain::Plan;
use solverforge::{ConstraintSet, HardMediumSoftScore};

#[test]
fn dienst_block_incremental_matches_full() {
    let mut plan: Plan = generate(DemoData::Small);
    let m = plan.mitarbeiters.len().max(1);
    let mut cs = dienst_block::constraint();
    let mut score: HardMediumSoftScore = cs.initialize_all(&plan);
    assert_eq!(score, cs.evaluate_all(&plan));

    let n = plan.schichtzuordnungs.len();
    for _round in 0..2 {
        for i in 0..n {
            let old = plan.schichtzuordnungs[i].mitarbeiter_idx;
            let new = if old == Some(0) {
                None
            } else {
                Some(old.map(|x| x + 1).unwrap_or(0) % m)
            };
            score = score + cs.on_retract_all(&plan, i, 0);
            plan.schichtzuordnungs[i].mitarbeiter_idx = new;
            score = score + cs.on_insert_all(&plan, i, 0);
            assert_eq!(score, cs.evaluate_all(&plan), "i={i} old={old:?} new={new:?}");
            score = score + cs.on_retract_all(&plan, i, 0);
            plan.schichtzuordnungs[i].mitarbeiter_idx = old;
            score = score + cs.on_insert_all(&plan, i, 0);
        }
    }
}
