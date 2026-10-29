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
    #[serde(default)]
    pub fahrzeug: String,
    pub qualifikation: Vec<Qualifikation>,
}

impl Schicht {
    pub fn new(id: impl Into<String>, index: usize, name: impl Into<String>, start: NaiveDateTime, ende: NaiveDateTime, wache: String, fahrzeug: String, qualifikation: Vec<Qualifikation>) -> Self {
        Self { id: id.into(), index: index, name: name.into(), start, ende, wache, fahrzeug, qualifikation }
    }

    pub fn fahrzeug_fuer_name(name: &str) -> String {
        if name.ends_with('N') {
            "40-82-1".to_string()
        } else if name.contains('M') {
            "41-83-1".to_string()
        } else if let Some(digit) = Self::fahrzeugnummer(name) {
            format!("40-83-{digit}")
        } else {
            String::new()
        }
    }

    pub fn fahrzeug_aus_name_ergaenzen(&mut self) {
        if self.fahrzeug.is_empty() {
            self.fahrzeug = Self::fahrzeug_fuer_name(&self.name);
        }
    }

    fn fahrzeugnummer(name: &str) -> Option<char> {
        name.as_bytes()
            .windows(2)
            .find_map(|pair| match pair {
                [b'V', digit] if digit.is_ascii_digit() => Some(*digit as char),
                _ => None,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schicht_construction() {
        let fact = Schicht::new("test-id", Default::default(), "test", Default::default(), Default::default(), "test".to_string(), String::new(), Default::default());
        assert_eq!(fact.id, "test-id");
        assert_eq!(fact.name, "test");
        let _ = &fact.start;
        let _ = &fact.ende;
        let _ = &fact.wache;
        let _ = &fact.fahrzeug;
        let _ = &fact.qualifikation;
    }

    #[test]
    fn fehlendes_fahrzeug_wird_aus_name_abgeleitet() -> Result<(), serde_json::Error> {
        let mut schicht: Schicht = serde_json::from_str(
            r#"{"id":"s0","index":0,"name":"TV2","start":"2026-10-05T06:00:00","ende":"2026-10-05T18:00:00","wache":"40","qualifikation":["RS"]}"#,
        )?;
        assert!(schicht.fahrzeug.is_empty());
        schicht.fahrzeug_aus_name_ergaenzen();
        assert_eq!(schicht.fahrzeug, "40-83-2");
        Ok(())
    }
}
