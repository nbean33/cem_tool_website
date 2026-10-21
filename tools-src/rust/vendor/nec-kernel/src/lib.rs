//! NEC-2 wire electromagnetics for `nec-rs`.
//!
//! This crate ports NEC-2's wire formulation — the current basis functions, the thin-wire kernel
//! and its numerical integration — from the public-domain NEC-2D Fortran, keeping NEC's numerics
//! but not its structure. Each module names the Fortran routines it corresponds to, so any result
//! can be traced back to the original for comparison.
//!
//! | module | Fortran | role |
//! |---|---|---|
//! | [`consts`] | `DATA` statements | NEC's own truncated constants |
//! | [`scaled`] | `DATAGN`, main program | segments in wavelength units |
//! | [`basis`] | `TBF`, `SBF`, `TRIO` | three-term sinusoidal basis functions |
//! | [`intx`] | `INTX`, `GF`, `TEST` | adaptive Romberg integration |
//! | [`eksc`] | `EKSC`, `GX`, `EKSCX`, `GXX` | thin-wire kernel, and the extended one for fat wires (`EK`) |
//! | [`efld`](mod@efld) | `EFLD` | near field of a segment, including its image over ground |
//! | [`ground`] | main program, `EFLD` | ground models; Fresnel weighting and radial screen |
//! | [`somnec`] | `SOM2D`, `EVLUA`, `GSHANK`, `ROM1`, `SAOA`, `BESSEL`, `HANKEL`, `INTRP` | Sommerfeld interpolation grid |
//! | [`sommerfeld`] | `EFLD`, `SFLDS`, `ROM2`, `GWAVE`, `FBAR` | Sommerfeld correction to a segment's field |
//! | [`cmath`] | — | complex division and square root as gfortran evaluates them |
//!
//! Not yet ported: surface patches.

pub mod basis;
pub mod cmath;
pub mod consts;
pub mod efld;
pub mod eksc;
pub mod ground;
pub mod hfield;
pub mod intx;
pub mod load;
pub mod scaled;
pub mod sommerfeld;
pub mod somnec;
pub mod surface;

pub use basis::{Contribution, Piece, basis_function, basis_function_with, pieces_by_segment};
pub use efld::{SegmentField, efld};
pub use eksc::{ExtendedEnd, extended_ends, extended_ends_off_matrix};
pub use ground::{Ground, ImageGround, RadialScreen};
pub use scaled::{WireSeg, scale};
pub use surface::{PatchField, SurfacePatch, scale_patches};
