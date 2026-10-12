use serde::{Deserialize, Serialize};
use solverforge::prelude::*;
use crate::domain::types::Qualifikation;

/// TODO — describe this fact.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct Mitarbeiter {
    #[planning_id]
    pub id: String,
    pub index: usize,
    pub name: String,
    pub stammwache: String,
    pub schichtgruppe: i32,
    pub is_br: bool,
    pub qualifikation: Vec<Qualifikation>,
    pub keine_nacht: bool,
    pub stellenanteil: i32,
}

impl Mitarbeiter {
    pub fn new(id: impl Into<String>, index: usize, name: impl Into<String>, stammwache: String, schichtgruppe: i32, is_br: bool, qualifikation: Vec<Qualifikation>, keine_nacht: bool, stellenanteil: i32) -> Self {
        Self { id: id.into(),index: index, name: name.into(), stammwache, schichtgruppe, is_br, qualifikation, keine_nacht, stellenanteil }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mitarbeiter_construction() {
        let fact = Mitarbeiter::new("test-id", Default::default(), "test", "test".to_string(), Default::default(), false, Default::default(), false, Default::default());
        assert_eq!(fact.id, "test-id");
        assert_eq!(fact.name, "test");
        let _ = &fact.stammwache;
        let _ = &fact.schichtgruppe;
        let _ = &fact.is_br;
        let _ = &fact.qualifikation;
        let _ = &fact.keine_nacht;
        let _ = &fact.stellenanteil;
    }
}
