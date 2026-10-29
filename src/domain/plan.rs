use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

// @solverforge:neutral-solution
// @solverforge:begin solution-imports
use super::Mitarbeiter;
use super::Schicht;
use super::Schichtzuordnung;
use super::{
    BalanceOffset, BlockMembership, QualifikationsKandidaten, ShiftMinutes, ShiftRelation,
    WindowMinutes,
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
    #[problem_fact_collection]
    #[serde(skip)]
    pub balance_offset: Vec<BalanceOffset>,
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
            balance_offset: Vec::new(),
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
        for mitarbeiter in &mut self.mitarbeiters {
            mitarbeiter.stammwache = wache_ohne_station(&mitarbeiter.stammwache);
        }
        for schicht in &mut self.schichts {
            schicht.wache = wache_ohne_station(&schicht.wache);
            schicht.fahrzeug_aus_name_ergaenzen();
        }
        if !self.provided_kandidaten_are_usable() {
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
        }
        for assignment in &mut self.schichtzuordnungs {
            let Some(employee_index) = assignment.mitarbeiter_idx else {
                continue;
            };
            let candidates = &self.kandidaten
                [super::qualifikations_index(assignment.qualifikation)]
                .mitarbeiter;
            if !candidates.contains(&employee_index) {
                assignment.mitarbeiter_idx = None;
            }
        }
        self.shift_minutes = super::temporal_index::minutes(&self.schichts);
        self.window_minutes = super::temporal_index::windows(&self.schichts);
        self.block_memberships = super::temporal_index::blocks(&self.schichts);
        self.shift_relations = super::temporal_index::relations(&self.schichts);
        self.balance_offset = super::balance_offset::offsets(
            self.schichtzuordnungs.len(),
            self.mitarbeiters.len(),
        );
        Ok(())
    }

    fn provided_kandidaten_are_usable(&self) -> bool {
        if self.kandidaten.len() != super::ALL_QUALIFIKATIONEN.len() {
            return false;
        }
        let mut ids = HashSet::new();
        for (index, candidate) in self.kandidaten.iter().enumerate() {
            if candidate.qualifikation != super::ALL_QUALIFIKATIONEN[index]
                || candidate.id.is_empty()
                || !ids.insert(candidate.id.as_str())
            {
                return false;
            }

            let mut employees = HashSet::new();
            for &employee_index in &candidate.mitarbeiter {
                let Some(employee) = self.mitarbeiters.get(employee_index) else {
                    return false;
                };
                if !employee.qualifikation.contains(&candidate.qualifikation)
                    || !employees.insert(employee_index)
                {
                    return false;
                }
            }

            let mut assignments = HashSet::new();
            for &assignment_index in &candidate.schichtzuordnungen {
                let Some(assignment) = self.schichtzuordnungs.get(assignment_index) else {
                    return false;
                };
                if assignment.qualifikation != candidate.qualifikation
                    || !assignments.insert(assignment_index)
                {
                    return false;
                }
            }
        }
        true
    }
}

fn wache_ohne_station(wache: &str) -> String {
    wache
        .strip_prefix("Station ")
        .unwrap_or(wache)
        .to_string()
}
