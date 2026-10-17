use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

// @solverforge:neutral-solution
// @solverforge:begin solution-imports
use super::Mitarbeiter;
use super::Schicht;
use super::Schichtzuordnung;
use super::{
    BlockMembership, QualifikationsKandidaten, ShiftMinutes, ShiftRelation, WindowMinutes,
};
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
    #[problem_fact_collection]
    pub kandidaten: Vec<QualifikationsKandidaten>,
    #[problem_fact_collection]
    #[serde(skip)]
    pub shift_minutes: Vec<ShiftMinutes>,
    #[problem_fact_collection]
    #[serde(skip)]
    pub window_minutes: Vec<WindowMinutes>,
    #[problem_fact_collection]
    #[serde(skip)]
    pub block_memberships: Vec<BlockMembership>,
    #[problem_fact_collection]
    #[serde(skip)]
    pub shift_relations: Vec<ShiftRelation>,
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
        kandidaten: Vec<QualifikationsKandidaten>,
        // @solverforge:end solution-constructor-params
    ) -> Self {
        let mut plan = Self{
            // @solverforge:begin solution-constructor-init
            mitarbeiters,
            schichts,
            schichtzuordnungs,
            kandidaten,
            // @solverforge:end solution-constructor-init
            shift_minutes: Vec::new(),
            window_minutes: Vec::new(),
            block_memberships: Vec::new(),
            shift_relations: Vec::new(),
            score: None,
        };
        plan.prepare().expect("invalid plan");
        plan
    }

    pub fn prepare(&mut self) -> Result<(), String> {
        if self
            .mitarbeiters
            .iter()
            .enumerate()
            .any(|(i, m)| m.index != i)
            || self.schichts.iter().enumerate().any(|(i, s)| {
                s.index != i
                    || s.ende <= s.start
                    || s.start
                        .checked_sub_signed(chrono::Duration::days(7))
                        .is_none()
                    || s.ende
                        .checked_add_signed(chrono::Duration::days(7))
                        .is_none()
            })
            || self.schichtzuordnungs.iter().any(|a| {
                a.schicht_id >= self.schichts.len()
                    || a.mitarbeiter_idx
                        .is_some_and(|i| i >= self.mitarbeiters.len())
            })
        {
            return Err("invalid shift interval or planning index".into());
        }
        self.kandidaten = super::ALL_QUALIFIKATIONEN
            .iter()
            .map(|q| QualifikationsKandidaten {
                id: format!("{q:?}"),
                qualifikation: *q,
                mitarbeiter: self
                    .mitarbeiters
                    .iter()
                    .filter(|m| m.qualifikation.contains(q))
                    .map(|m| m.index)
                    .collect(),
                schichtzuordnungen: self
                    .schichtzuordnungs
                    .iter()
                    .enumerate()
                    .filter(|(_, a)| a.qualifikation == *q)
                    .map(|(i, _)| i)
                    .collect(),
            })
            .collect();
        self.shift_minutes = super::temporal_index::minutes(&self.schichts);
        self.window_minutes = super::temporal_index::windows(&self.schichts);
        self.block_memberships = super::temporal_index::blocks(&self.schichts);
        self.shift_relations = super::temporal_index::relations(&self.schichts);
        Ok(())
    }
}
