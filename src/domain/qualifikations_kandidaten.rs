use std::any::Any;
use serde::{Deserialize, Serialize};
use solverforge::prelude::*;
use super::Plan;
use super::types::Qualifikation;

/// Problem-Fact mit den zulaessigen Kandidaten je Qualifikation.
/// Wird von den `nearby_*`-Hooks der Variable `mitarbeiter_idx` genutzt, damit
/// nach hard = 0 nur noch Moves/Swaps innerhalb derselben Qualifikation
/// evaluiert werden.
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

pub fn qualifizierte_mitarbeiter(
    solution: &dyn Any,
    entity_index: usize,
    _variable_index: usize,
) -> &[usize] {
    let plan = solution.downcast_ref::<Plan>().expect("Plan erwartet");
    let qualifikation = plan.schichtzuordnungs[entity_index].qualifikation;
    &plan.kandidaten[qualifikations_index(qualifikation)].mitarbeiter
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
