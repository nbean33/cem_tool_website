pub mod constants;
pub mod copper;
pub mod error;
pub mod materials;
pub mod units;

// Calculators
pub mod crosstalk;
pub mod current;
pub mod differential;
pub mod fusing;
pub mod impedance;
pub mod inductor;
pub mod ohms_law;
pub mod padstack;
pub mod pdn;
pub mod ppm;
pub mod reactance;
pub mod resistance;
pub mod signal;
pub mod spacing;
pub mod thermal;
pub mod via;
pub mod wavelength;
pub mod wire_gauge;
pub mod wires;

pub use error::CalcError;
pub(crate) use error::all_finite;
pub use units::{Capacitance, Freq, Inductance, Length, Temperature, UnitParseError};
