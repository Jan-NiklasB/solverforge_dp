use serde::{Deserialize, Serialize};
use solverforge::prelude::*;
use crate::domain::types::Qualifikation;
use chrono::NaiveDateTime;

/// TODO — describe this fact.
#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct Schicht {
    #[planning_id]
    pub id: String,
    pub index: usize,
    pub name: String,
    pub start: NaiveDateTime,
    pub ende: NaiveDateTime,
    pub wache: String,
    pub qualifikation: Vec<Qualifikation>,
}

impl Schicht {
    pub fn new(id: impl Into<String>, index: usize, name: impl Into<String>, start: NaiveDateTime, ende: NaiveDateTime, wache: String, qualifikation: Vec<Qualifikation>) -> Self {
        Self { id: id.into(), index: index, name: name.into(), start, ende, wache, qualifikation }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schicht_construction() {
        let fact = Schicht::new("test-id", Default::default(), "test", Default::default(), Default::default(), "test".to_string(), Default::default());
        assert_eq!(fact.id, "test-id");
        assert_eq!(fact.name, "test");
        let _ = &fact.start;
        let _ = &fact.ende;
        let _ = &fact.wache;
        let _ = &fact.qualifikation;
    }
}
