use std::any::Any;
use serde::{Deserialize, Serialize};
use solverforge::prelude::*;
use super::Plan;
use super::types::Qualifikation;

/// Problem-Fact mit den zulaessigen Kandidaten je Qualifikation.
/// Begrenzt die Variable `mitarbeiter_idx` in allen Phasen auf qualifizierte Mitarbeiter.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct QualifikationsKandidaten {
    #[planning_id]
    pub id: String,
    pub qualifikation: Qualifikation,
    pub mitarbeiter: Vec<usize>,
    pub schichtzuordnungen: Vec<usize>,
}

pub const ALL_QUALIFIKATIONEN: [Qualifikation; 6] = [
    Qualifikation::RH,
    Qualifikation::RS,
    Qualifikation::RA,
    Qualifikation::NFS,
    Qualifikation::PAL,
    Qualifikation::BR,
];

pub fn qualifikations_index(qualifikation: Qualifikation) -> usize {
    ALL_QUALIFIKATIONEN
        .iter()
        .position(|q| *q == qualifikation)
        .unwrap_or(0)
}

pub fn qualifizierte_kandidaten(
    plan: &Plan,
    entity_index: usize,
    _variable_index: usize,
) -> &[usize] {
    let qualifikation = plan.schichtzuordnungs[entity_index].qualifikation;
    &plan.kandidaten[qualifikations_index(qualifikation)].mitarbeiter
}

pub fn qualifizierte_mitarbeiter(
    solution: &dyn Any,
    entity_index: usize,
    variable_index: usize,
) -> &[usize] {
    let plan = solution.downcast_ref::<Plan>().expect("Plan erwartet");
    qualifizierte_kandidaten(plan, entity_index, variable_index)
}

pub fn gleiche_qualifikation_sz(
    solution: &dyn Any,
    entity_index: usize,
    _variable_index: usize,
) -> &[usize] {
    let plan = solution.downcast_ref::<Plan>().expect("Plan erwartet");
    let qualifikation = plan.schichtzuordnungs[entity_index].qualifikation;
    &plan.kandidaten[qualifikations_index(qualifikation)].schichtzuordnungen
}
