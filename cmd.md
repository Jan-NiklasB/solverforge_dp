solverforge generate score HardMediumSoftScore

solverforge generate fact mitarbeiter --field stammwache:String --field schichtgruppe:i32 --field is_br:bool --field "qualifikation:Vec<Qualifikation>" --field keine_nacht:bool --field stellenanteil:i32

solverforge generate fact schicht --field "start:NaiveDateTime" --field "ende:NaiveDateTime" --field wache:String --field "qualifikation:Vec<Qualifikation>"

solverforge generate entity schichtzuordnung --field schicht_id:usize --field "qualifikation:Qualifikation"

solverforge generate variable mitarbeiter_idx --entity Schichtzuordnung --kind scalar --range mitarbeiter

solverforge generate constraint shift_assigned --unary --hard