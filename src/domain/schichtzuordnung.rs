use serde::{Deserialize, Serialize};
use solverforge::prelude::*;
use crate::domain::Plan;
use crate::domain::types::Qualifikation;

/// TODO — describe this entity.
#[planning_entity]
#[derive(Serialize, Deserialize)]
pub struct Schichtzuordnung {
    #[planning_id]
    pub id: String,
    pub schicht_id: usize,
    pub qualifikation: Qualifikation,
    // @solverforge:begin entity-variables
    #[planning_variable(
        value_range_provider = "mitarbeiters",
        allows_unassigned = false,
        candidate_values = "crate::domain::qualifikations_kandidaten::qualifizierte_kandidaten",
        nearby_value_candidates = "crate::domain::qualifikations_kandidaten::qualifizierte_mitarbeiter",
        nearby_entity_candidates = "crate::domain::qualifikations_kandidaten::gleiche_qualifikation_sz",
    )]
    pub mitarbeiter_idx: Option<usize>,
    // @solverforge:end entity-variables
}

impl Schichtzuordnung {
    pub fn new(id: impl Into<String>, schicht_id: usize, qualifikation: Qualifikation) -> Self {
        Self {
            id: id.into(),
            schicht_id,
            qualifikation,
            // @solverforge:begin entity-variable-init
            mitarbeiter_idx: None,
            // @solverforge:end entity-variable-init
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schichtzuordnung_construction() {
        let entity = Schichtzuordnung::new(
            "test-id",
            Default::default(),
            Default::default());
        assert_eq!(entity.id, "test-id");
        let _ = &entity.schicht_id;
        let _ = &entity.qualifikation;
    }
}
