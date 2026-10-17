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
use solverforge::HardMediumSoftScore;
use solverforge::IncrementalConstraint;
use std::time::Instant;

const ITERS: u32 = 30;
const INCR_ITERS: u32 = 5;

fn bench(name: &str, f: impl Fn() -> i64) {
    let _ = f();
    let start = Instant::now();
    for _ in 0..ITERS {
        std::hint::black_box(f());
    }
    let elapsed = start.elapsed().as_secs_f64() * 1000.0 / ITERS as f64;
    println!("{name:28} {elapsed:9.3} ms/eval");
}

fn bench_incr<C>(name: &str, mut cs: C, plan: &mut Plan, m: usize)
where
    C: IncrementalConstraint<Plan, HardMediumSoftScore>,
{
    cs.initialize(plan);
    let n = plan.schichtzuordnungs.len();
    let start = Instant::now();
    let mut ops = 0u64;
    for _ in 0..INCR_ITERS {
        for i in 0..n {
            let Some(old) = plan.schichtzuordnungs[i].mitarbeiter_idx else {
                continue;
            };
            cs.on_retract(plan, i, 0);
            plan.schichtzuordnungs[i].mitarbeiter_idx = Some((old + 1) % m);
            cs.on_insert(plan, i, 0);
            cs.on_retract(plan, i, 0);
            plan.schichtzuordnungs[i].mitarbeiter_idx = Some(old);
            cs.on_insert(plan, i, 0);
            ops += 2;
        }
    }
    let us = start.elapsed().as_secs_f64() * 1e6 / ops as f64;
    println!("{name:28} {us:8.2} us/move (incr)");
}

#[test]
fn perf_standard() {
    let plan = generate(DemoData::Standard);
    let m = plan.mitarbeiters.len();
    println!(
        "plan: {} mitarbeiter, {} schichten, {} zuordnungen",
        m,
        plan.schichts.len(),
        plan.schichtzuordnungs.len()
    );
    bench("shift_assigned", || shift_assigned::constraint().evaluate(&plan).hard());
    bench("ueberlappende_schichten", || ueberlappende_schichten::constraint().evaluate(&plan).hard());
    bench("ruhezeit", || ruhezeit::constraint().evaluate(&plan).hard());
    bench("max_60h", || max_60h::constraint().evaluate(&plan).hard());
    bench("selber_block", || {
        let s = selber_block::constraint().evaluate(&plan);
        s.hard() + s.medium()
    });
    bench("balancing", || balancing::constraint().evaluate(&plan).soft());
    bench("nachtschicht_frei", || nachtschicht_frei::constraint().evaluate(&plan).medium());
    bench("durchschnittszeit", || durchschnittszeit::constraint().evaluate(&plan).medium());
    println!("durchschnittszeit medium(seed) = {}", durchschnittszeit::constraint().evaluate(&plan).medium());
    println!("nachtschicht_frei medium(seed) = {}", nachtschicht_frei::constraint().evaluate(&plan).medium());

    println!("--- incremental ---");
    let mut plan = plan;
    bench_incr("shift_assigned", shift_assigned::constraint(), &mut plan, m);
    bench_incr("ueberlappende_schichten", ueberlappende_schichten::constraint(), &mut plan, m);
    bench_incr("ruhezeit", ruhezeit::constraint(), &mut plan, m);
    bench_incr("max_60h", max_60h::constraint(), &mut plan, m);
    bench_incr("selber_block", selber_block::constraint(), &mut plan, m);
    bench_incr("balancing", balancing::constraint(), &mut plan, m);
    bench_incr("nachtschicht_frei", nachtschicht_frei::constraint(), &mut plan, m);
    bench_incr("durchschnittszeit", durchschnittszeit::constraint(), &mut plan, m);
}
