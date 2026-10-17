# AGENTS.md — SolverForge context for the `dp` project

Purpose: enable fast, correct code improvements and generation in this project by
capturing the SolverForge framework facts that matter here. `dp` is a
CLI-scaffolded **web-shell** SolverForge app for German shift scheduling
(Dienstplan). Sources of truth: the local clones under `/tmp/opencode/sf/`
(`solverforge`, `solverforge-cli`, `solverforge-ui`, `solverforge-maps`,
`solverforge-usecases`) and each repo's `WIREFRAME.md` / `AGENTS.md`.

## Versions & dependency shape

- Runtime: `solverforge` 0.21.3 (features `serde`, `console`, `verbose-logging`).
- UI: `solverforge-ui` 0.9.0; Maps: `solverforge-maps` 2.1.4; CLI: `solverforge-cli` 3.5.x.
- Rust: 1.95+. Web shell uses axum 0.8, tokio 1.52, tower-http 0.6, serde, uuid, parking_lot.
- Do NOT bump crate versions without explicit user confirmation.

## This project's structure

```
Cargo.toml, Cargo.lock, solver.toml, solverforge.app.toml
src/domain/    mitarbeiter, schicht, schichtzuordnung, plan + mod.rs (planning_model!)
src/constraints/ shift_assigned, benoetigte_qualifikation, balancing,
                 ueberlappende_schichten, ruhezeit, selber_block + mod.rs
src/data/      data_seed.rs (demo sizes: Small 8/8, Standard 45/152, Large 120/640)
src/api/, src/solver/, src/main.rs, src/lib.rs
static/        generated UI (ui-model.json) — web shell only
```

### Managed blocks (canonical CLI edit points — never delete)
`// @solverforge:begin X` … `// @solverforge:end X`. Current regions:
- `src/domain/mod.rs`: `domain-exports`
- `src/domain/plan.rs`: `solution-imports`, `solution-collections`,
  `solution-constructor-params`, `solution-constructor-init`
- entity files: `entity-variables`, `entity-variable-init`
- `src/constraints/mod.rs`: `constraint-modules`, `constraint-calls`
- `solver.toml`: `# @solverforge:begin solver-config` / `# @solverforge:end solver-config`

`planning_model!` in `src/domain/mod.rs` is the model-owned metadata manifest:
```rust
solverforge::planning_model! { root = "src/domain"; /* domain-exports block */ }
```

## Hard rules (framework)

- `IncrementalConstraintSealed` is **sealed**: constraints can only be built via
  the fluent API and finalizers (`penalize`/`reward`/`named`). Never implement it.
- Zero-erasure: no `Box<dyn>`/`Arc`/`Rc` on scoring/move paths; avoid `.clone()`
  in scoring; keep concrete generic types.
- `#[derive(Clone)]` only via manual impls for generic types (avoid extra bounds).
- Keep source/test files < 500 LOC; `mod.rs` = declarations/re-exports only.
- No compat shims/aliases; delete dead code; no external-solver references.
- Don't add comments unless asked (project convention).

## Domain attributes (macros crate)

- `#[planning_solution]`, `#[planning_entity]`, `#[problem_fact]`
- `#[planning_variable]` (scalar; options incl. `value_range_provider`,
  `allows_unassigned`, `candidate_values`, nearby/*, construction order keys)
- `#[planning_list_variable]` (sequence/route; `domain = "cvrp"` profile or custom
  `distance_meter`, `route_hooks`, `savings_hooks`, `element_owner_fn`, …)
- `#[planning_id]`, `#[planning_score]`, `#[problem_fact_collection]`,
  `#[planning_entity_collection]`
- Generated accessors, e.g. `Plan::mitarbeiters()`, `Plan::schichtzuordnungs()`.

## Constraint API essentials (scoring crate)

Entry: `ConstraintFactory::new().for_each(...)`. Streams by arity:
`Uni/Bi/Tri/Quad/Penta`, cross joins `stream::cross::{Bi,Tri,Quad,Penta}`,
grouped/complemented/projected/existence/flattened variants.
Key traits:
- `IncrementalConstraint<S, Sc>`: `evaluate`, `match_count`, `initialize`,
  `on_insert`, `on_retract`, `reset`, `name`, `is_hard`, `constraint_ref`,
  `get_matches`, `weight`.
- `ConstraintSet<S, Sc>`: `evaluate_all`, `evaluate_each`, `evaluate_detailed`,
  `initialize_all`, `on_insert_all`, `on_retract_all`, `constraint_metadata`, …
  Implemented for tuples `()`..`(C0..C31)`; compose via `ConstraintSetChain` /
  `OrderedConstraintSetChain`.
- `Director<S>` / `ScoreDirector<S, C>` for runtime scoring.

Fluent ops & gotchas:
- `join(target, joiner)`: `equal(...)`, `equal_bi(left_key, right_key)`,
  `equal_on(...)`, comparisons (`less_than`/`greater_than`), `overlapping(...)`,
  `.and(filtering(...))`.
- `equal_bi` is only valid on the **first** join from a leaf stream.
- `group_by(key, collector)` is only implemented for **keyed** joins
  (`BiUnaryPlan`), NOT for predicate cross-joins (`BiPredicatePlan`). For a
  cross-keyed join use `stream::cross::Bi` + `equal_bi(|_a| 0usize, |_b| 0usize)`
  (constant key) then `group_by`.
- Grouped terminals chain multiple scorers via `.penalize(...).named(...)` /
  `.reward(...).named(...)` (node sharing by `#[solverforge_constraints]`).
- Weighting: dynamic closure weights are non-hard by default; use `hard_weight(w)`
  to force hard metadata, `fixed_weight(score)` for custom score types.

Collectors (`Collector<Input>`/`Accumulator<V,R>`):
- `count()`, `sum()`, `collect_vec()` (→ `CollectedVec<T>`, no Clone/Copy needed),
  `load_balance()`, `consecutive_runs()`, `indexed_presence()`.
- `load_balance` **skips metric == 0**, so unassigned/zero-shift entities are
  absent from `loads()`; the `1 + match` offset trick makes every entity appear
  (offset cancels in the deviation).
- Direct cross-join grouping passes `(&A, &B)` to the collector; unary/projected
  grouping passes `&A`/`&Out`. E.g. `sum(|(shift, employee): (&Shift, &Employee)| ...)`.
- Accumulators own values: `accumulate(V) -> Retraction` (retraction token cached
  for exact retraction after mutation). Grouped nodes retract-by-entity then
  re-insert, so constant-key joins still update metrics correctly.

Analysis: `ScoreExplanation`, `ConstraintAnalysis`, `Indictment(Map)`,
`ConstraintWeightOverrides`, `WeightProvider`.

`balancing.rs` in this project is the reference for a constant-key cross join +
`group_by` + `load_balance` + `penalize` computing `Σ (count − min)²`.

## solver.toml schema (solverforge-config)

Top-level: `environment_mode` (`non_reproducible` default | `reproducible` |
`fast_assert` | `full_assert`), `random_seed` (u64?), `move_thread_count`
(`auto` | `none` | integer), `[termination]`, `[score_director]`, `[[phases]]`,
optional `[candidate_trace] { max_entries }`.

`[termination]`: `seconds_spent_limit`, `minutes_spent_limit`, `best_score_limit`
(string, e.g. `"0hard/0soft"`), `step_count_limit`, `unimproved_step_count_limit`,
`unimproved_seconds_spent_limit`.

`[[phases]]` `type` (tagged): `construction_heuristic`, `local_search`,
`partitioned_search`, `custom`.

- construction_heuristic: `construction_heuristic_type`
  (`first_fit` default | `first_fit_decreasing` | `weakest_fit` |
  `weakest_fit_decreasing` | `strongest_fit` | `strongest_fit_decreasing` |
  `cheapest_insertion` | `allocate_entity_from_queue` | `allocate_to_value_from_queue` |
  `list_round_robin` | `list_cheapest_insertion` | `list_regret_insertion` |
  `list_clarke_wright` | `list_k_opt`), `construction_obligation`
  (`preserve_unassigned` default | `assign_when_candidate_exists`),
  `entity_class`/`variable_name` (flattened target), `k`, `value_candidate_limit`,
  `group_name`, `group_candidate_limit`, `[phases.termination]`.
- local_search: `local_search_type` (`acceptor_forager` default |
  `variable_neighborhood_descent`), `[phases.acceptor]`, `[phases.forager]`,
  `score_tie_break` (`random` default | `first`), `[phases.move_selector]` or
  `[[phases.move_selector.selectors]]` or `[[phases.move_selector_union]]`,
  `[[phases.neighborhoods]]` (VND only), `[phases.termination]`.
  - Acceptor `type`: `hill_climbing`, `tabu_search` (`entity_tabu_size`,
    `value_tabu_size`, `move_tabu_size`, `undo_move_tabu_size`, `aspiration_enabled`;
    bare tabu → move_tabu_size 10 + aspiration), `step_counting_hill_climbing`
    (`step_count_limit`), `simulated_annealing` (`level_temperatures`, `decay_rate`,
    `hill_climbing_temperature`, `hard_regression_policy`, `calibration`),
    `late_acceptance` (`late_acceptance_size`), `diversified_late_acceptance`
    (`late_acceptance_size`, `tolerance`), `great_deluge`
    (`water_level_increase_ratio`).
  - Forager `type`: `accepted_count` (`limit`), `best_score`, `first_accepted`,
    `first_best_score_improving`, `first_last_step_score_improving`.
  - Selectors (`type`): `change_move_selector` (`value_candidate_limit`,
    `entity_class`, `variable_name`), `swap_move_selector` (`variable_name` /
    `variable_names`), `nearby_change_move_selector`, `nearby_swap_move_selector`,
    `pillar_change_move_selector`, `pillar_swap_move_selector`,
    `ruin_recreate_move_selector` (`min_ruin_count`, `max_ruin_count`,
    `moves_per_step`, `value_candidate_limit`, `recreate_heuristic_type`
    `first_fit`|`cheapest_insertion`), `grouped_scalar_move_selector`
    (`group_name`, `value_candidate_limit`, `max_moves_per_step`,
    `require_hard_improvement`), list selectors (`list_change`, `nearby_list_change`,
    `list_swap`, `list_permute` (`min/max_window_size`), `list_precedence`,
    `nearby_list_swap`, `sublist_change`, `sublist_swap`, `list_reverse`,
    `k_opt_move_selector` (`k`, `min_segment_len`, `max_nearby`),
    `list_ruin_move_selector` (`min/max_ruin_count`, `moves_per_step`,
    `max_source_list_len`, `skip_empty_destinations`)),
    `conflict_repair_move_selector` / `compound_conflict_repair_move_selector`
    (`constraints` (exact IDs), `max_matches_per_step`, `max_repairs_per_match`,
    `max_moves_per_step`, `require_hard_improvement`, `include_soft_matches`),
    `limited_neighborhood` (`selected_count_limit`, `selector`),
    `union_move_selector` (`selection_order`: `sequential`|`round_robin`|
    `rotating_round_robin`|`random`|`stratified_random` default; `weighting`:
    `equal`|`fixed`|`candidate_count`; `weights`; `selectors`),
    `cartesian_product_move_selector` (`require_hard_improvement`, `selectors`).
  - Common leaf fields: `selection_order` (`original`|`random`|`shuffled`|
    `sorted`|`probabilistic`) and `selection_metric` (required for sorted/
    probabilistic).
- Config loads TOML/YAML via `SolverConfig::load/from_toml_file/from_toml_str/
  from_yaml_*`; `#[planning_solution]` may point at it. Omitting `phases` uses the
  canonical model-aware default profile.

Note on this repo's `solver.toml`: it uses `[[phases.move_selector.selectors]]`
and also `[[phases.move_selector_union]]` (legacy-ish alias for union children).
Keep it valid; `solverforge check` (CLI) validates generated refs and graph.

## CLI (solverforge-cli) — generation & maintenance

- `solverforge new <name> [--shell web|api|cli|mcp] [--skip-git] [--skip-readme]`
- `solverforge generate fact|entity|variable|constraint|solution|score|data`
  - `generate entity shift --planning-variable employee_idx`
  - `generate variable resource_idx --entity Task --kind scalar --range resources
    [--allows-unassigned] [--candidate-values ...] [--nearby-* ...]`
  - `generate variable hour --entity Shift --kind scalar --countable-range 0..24`
  - `generate variable visit_order --entity Route --kind list --elements visits
    [--domain cvrp] [--distance-meter ...] [--element-owner-fn ...]`
  - `generate constraint no_overlap --pair --hard` (also `--pair`/`--join`/…, `--hard`)
  - `generate data [--size small|standard|large] [--mode sample|stub]`
  - `generate scalar-group|conflict-repair` (opt-in, exact IDs)
- `solverforge destroy <kind> [--yes]` — removes resource + rewrites spec/UI projection
- `solverforge check` | `info` | `routes` | `test [-- args]`
- `solverforge server [--port N] [--debug]` (web/api via cargo; mcp stdio/`--http`)
- `solverforge connect [--write opencode|claude|cursor|vscode] [--port N]` (MCP)
- `solverforge config show|set` (non-phase solver.toml settings; ordered `phases`
  are manual). `.solverforgerc` supports `port`, `no_color`, `quiet`.

Generation contract: all mutations go through the managed blocks above; keep them
intact. `generate data` owns `src/data/mod.rs` (stable wrapper) and rewrites
`src/data/data_seed.rs`. `solverforge.app.toml` + `static/generated/ui-model.json`
are CLI-projections (web shell only).

## Build & validation

- `cargo build`, `cargo test`, `cargo clippy --all-targets`
- `solverforge check`, `solverforge info`, `solverforge test`
- Test pattern for constraints: `tests/<name>.rs` with
  `#[path = "../src/constraints/<file>.rs"] mod <name>; use dp::domain;`
  (see `tests/balancing.rs`, `tests/overlap.rs`). Validate both `evaluate` and the
  incremental path.

## UI / maps (only when touching the web shell)

- `solverforge-ui`: mounted via `solverforge_ui::routes()` serving `/sf/*`; JS
  entry `SF.createBackend`, `SF.createSolver`, `SF.createHeader`,
  `SF.createStatusBar`, `SF.createTabs`, `SF.rail.createTimeline`. Assets are
  `include_dir!`-embedded; CSS/JS at `/sf/sf.css`, `/sf/sf.js`.
- `solverforge-maps`: VRP/routing utilities — `Coord`, `BoundingBox`,
  `NetworkConfig`, `RoadNetwork` (`load_or_fetch`, `compute_matrix`, `route`),
  polyline encoding, OSM caching. Only relevant if `dp` gains routing.

## Reference clones

`/tmp/opencode/sf/solverforge` (core/scoring/solver/config/macros + examples +
`docs/extend-{domain,scoring,solver}.md`), `/tmp/opencode/sf/solverforge-cli`,
`/tmp/opencode/sf/solverforge-ui`, `/tmp/opencode/sf/solverforge-maps`,
`/tmp/opencode/sf/solverforge-usecases` (incl. `uc-hospital`, the closest
scheduling analog). Re-clone these shallowly if `/tmp` was cleared.

## Known gaps (not yet captured)

- solverforge-solver runtime wireframe (1789 lines) — SolverManager lifecycle,
  realtime `ProblemChange`, partitioner details, candidate-trace specifics.
- solverforge-core wireframe (759) — exact `Score`/`HardMediumSoftScore` API.
- Full example walkthroughs in `solverforge/examples/**`.
