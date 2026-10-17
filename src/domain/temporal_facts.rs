use serde::{Deserialize, Serialize};
use solverforge::prelude::*;

#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct ShiftMinutes {
    #[planning_id]
    pub id: usize,
    pub minutes: i64,
    pub average_limit: i64,
}

#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct WindowMinutes {
    #[planning_id]
    pub id: usize,
    pub shift: usize,
    pub window: usize,
    pub minutes: i64,
}

#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct BlockMembership {
    #[planning_id]
    pub id: usize,
    pub shift: usize,
    pub block: usize,
}

#[problem_fact]
#[derive(Serialize, Deserialize)]
pub struct ShiftRelation {
    #[planning_id]
    pub id: usize,
    pub first: usize,
    pub second: usize,
    pub overlap: bool,
    pub rest: bool,
    pub night_to_day: bool,
}
