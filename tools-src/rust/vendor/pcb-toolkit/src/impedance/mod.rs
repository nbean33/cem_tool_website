//! Transmission line impedance calculators.
//!
//! Supports multiple topologies: microstrip, stripline (centred and offset), embedded
//! microstrip, coplanar waveguide. Each topology module computes Zo, Er_eff, Tpd, Lo, Co
//! through the shared [`common::finish`] post-condition.
//!
//! Primary reference: Hammerstad & Jensen, "Accurate Models for Microstrip
//! Computer-Aided Design", IEEE MTT-S 1980.

pub mod common;
pub mod coplanar;
pub mod embedded;
pub mod microstrip;
pub mod offset_stripline;
pub mod stripline;
pub mod types;
