pub(crate) mod types;

solverforge::planning_model! {
    root = "src/domain";

    // @solverforge:begin domain-exports
mod mitarbeiter;
mod schicht;
mod schichtzuordnung;
mod plan;

pub use mitarbeiter::Mitarbeiter;
pub use schicht::Schicht;
pub use schichtzuordnung::Schichtzuordnung;
pub use plan::Plan;
    // @solverforge:end domain-exports
}
