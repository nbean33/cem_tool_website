use serde::{Deserialize, Serialize};

/// Result of an impedance calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpedanceResult {
    /// Characteristic impedance (Ohms).
    pub zo: f64,
    /// Effective dielectric constant (dimensionless).
    pub er_eff: f64,
    /// Propagation delay (ps/in).
    pub tpd_ps_per_in: f64,
    /// Inductance per unit length (nH/in).
    pub lo_nh_per_in: f64,
    /// Capacitance per unit length (pF/in).
    pub co_pf_per_in: f64,
    /// Warnings about a cross-section that has left the model's published accuracy band.
    ///
    /// Empty for a geometry inside the band, and for the topologies that have no published
    /// band to check against. A closed-form approximation outside its stated range still
    /// returns the best number available, so these are reported rather than turned into
    /// errors — but they have to be *reported*, which is why they travel with the result
    /// instead of sitting in a function nothing calls.
    ///
    /// `skip_serializing_if` keeps the JSON byte-identical when there is nothing to say, so
    /// adding this field did not change any existing output.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accuracy_warnings: Vec<String>,
}
