use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

// @solverforge:neutral-solution
// @solverforge:begin solution-imports
use super::Mitarbeiter;
use super::Schicht;
use super::Schichtzuordnung;
// @solverforge:end solution-imports

/// The root planning solution.
///
/// Fresh projects start as a neutral shell. Add fact collections, planning
/// entity collections, and variable fields through the CLI as your domain
/// takes shape.
#[planning_solution(
    constraints = "crate::constraints::create_constraints",
    solver_toml = "../../solver.toml"
)]
#[derive(Serialize, Deserialize)]
pub struct Plan {
    // @solverforge:begin solution-collections
    #[problem_fact_collection]
    pub mitarbeiters: Vec<Mitarbeiter>,
    #[problem_fact_collection]
    pub schichts: Vec<Schicht>,
    #[planning_entity_collection]
    pub schichtzuordnungs: Vec<Schichtzuordnung>,
    // @solverforge:end solution-collections
    #[planning_score]
    pub score: Option<HardMediumSoftScore>,
}

impl Plan {
    #[rustfmt::skip]
    pub fn new(
        // @solverforge:begin solution-constructor-params
        mitarbeiters: Vec<Mitarbeiter>,
        schichts: Vec<Schicht>,
        schichtzuordnungs: Vec<Schichtzuordnung>,
        // @solverforge:end solution-constructor-params
    ) -> Self {
        Self {
            // @solverforge:begin solution-constructor-init
            mitarbeiters,
            schichts,
            schichtzuordnungs,
            // @solverforge:end solution-constructor-init
            score: None,
        }
    }
}
