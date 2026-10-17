pub(crate) mod types;

solverforge::planning_model! {
    root = "src/domain";

    // @solverforge:begin domain-exports
mod mitarbeiter;
mod schicht;
mod schichtzuordnung;
mod plan;
mod qualifikations_kandidaten;
mod temporal_facts;
mod temporal_index;

pub use mitarbeiter::Mitarbeiter;
pub use schicht::Schicht;
pub use schichtzuordnung::Schichtzuordnung;
pub use plan::Plan;
pub use qualifikations_kandidaten::QualifikationsKandidaten;
pub use qualifikations_kandidaten::ALL_QUALIFIKATIONEN;
pub use qualifikations_kandidaten::qualifikations_index;
pub use temporal_facts::{ShiftMinutes, WindowMinutes, BlockMembership, ShiftRelation};
    // @solverforge:end domain-exports
}
