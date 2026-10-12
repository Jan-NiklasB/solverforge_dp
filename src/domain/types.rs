use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Eq, PartialEq, Deserialize, Serialize, Hash, Debug, Default)]
pub enum Qualifikation{
    #[default]
    RH,
    RS,
    RA,
    NFS,
    PAL,
    BR,
}