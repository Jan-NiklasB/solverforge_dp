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
        )
        // @solverforge:end constraint-calls
    }
}
/* mod ueberlappende_schichten; */
/* ueberlappende_schichten::constraint(),*/
