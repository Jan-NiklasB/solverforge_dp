/* Constraint definitions.

Add constraint modules with `solverforge generate constraint ...`.
The neutral shell starts with an empty constraint set. */

use crate::domain::Plan;
use solverforge::prelude::*;

pub use self::assemble::create_constraints;

// @solverforge:begin constraint-modules
mod shift_assigned;
mod benoetigte_qualifikation;
mod balancing;
mod ueberlappende_schichten;
mod ruhezeit;
mod selber_block;
mod max_60h;
mod nachtschicht_frei;
mod durchschnittszeit;
// @solverforge:end constraint-modules

mod assemble {
    use super::*;

    pub fn create_constraints() -> impl ConstraintSet<Plan, HardMediumSoftScore> {
        // @solverforge:begin constraint-calls
        (
            shift_assigned::constraint(),
            benoetigte_qualifikation::constraint(),
            balancing::constraint(),
            ueberlappende_schichten::constraint(),
            ruhezeit::constraint(),
            selber_block::constraint(),
            max_60h::constraint(),
            nachtschicht_frei::constraint(),
            durchschnittszeit::constraint(),
        )
        // @solverforge:end constraint-calls
    }
}
