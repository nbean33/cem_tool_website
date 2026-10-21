//! A deck made ready to solve.

use crate::{EngineError, Result};
use nec_cards::{
    Deck, Item, SymbolTable, Variable, lower_4nec2, parse_deck, uses_4nec2_extensions, variables,
};
use nec_core::Frequency;
use nec_geom::Geometry;
use nec_solve::{
    GreensFunction, InputParameters, Program, Solution, SolveOptions, read_program_on, solve,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

/// Where numerical Green's functions (`WG`, `GF`) are kept between runs.
///
/// A `WG` run's function is held in memory under its file name, and — when the store has a
/// directory, normally the deck's own — written there too, so that a later deck's `GF` can read
/// it as with 4nec2. Clones share the same memory.
#[derive(Debug, Clone, Default)]
pub struct NgfStore {
    dir: Option<PathBuf>,
    memory: Arc<Mutex<HashMap<String, Arc<GreensFunction>>>>,
}

impl NgfStore {
    /// A store that also reads and writes files in `dir`.
    #[must_use]
    pub fn in_dir(dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: Some(dir.into()),
            memory: Arc::default(),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, Arc<GreensFunction>>> {
        self.memory.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The function named `name`: from memory, else from the store's directory.
    pub fn get(&self, name: &str) -> Result<Arc<GreensFunction>> {
        let key = name.to_ascii_uppercase();
        if let Some(gf) = self.lock().get(&key) {
            return Ok(Arc::clone(gf));
        }
        let path = self.dir.as_ref().map(|d| d.join(name)).ok_or_else(|| {
            EngineError::Ngf(format!(
                "no numerical Green's function {name}: run its WG deck first"
            ))
        })?;
        let bytes = std::fs::read(&path).map_err(|e| {
            EngineError::Ngf(format!("{}: {e} (run its WG deck first)", path.display()))
        })?;
        let gf = Arc::new(
            GreensFunction::from_bytes(&bytes)
                .map_err(|e| EngineError::Ngf(format!("{}: {e}", path.display())))?,
        );
        self.lock().insert(key, Arc::clone(&gf));
        Ok(gf)
    }

    /// Compute the function of a `WG` run and keep it under `name`, writing the file when the
    /// store has a directory. An unchanged run reuses the function already held.
    pub fn put(&self, name: &str, run_text: &str) -> Result<Arc<GreensFunction>> {
        let key = name.to_ascii_uppercase();
        if let Some(gf) = self.lock().get(&key).filter(|gf| gf.deck == run_text) {
            return Ok(Arc::clone(gf));
        }
        let gf = Arc::new(
            GreensFunction::compute(run_text).map_err(|e| EngineError::Ngf(e.to_string()))?,
        );
        if let Some(dir) = &self.dir {
            let path = dir.join(name);
            std::fs::write(&path, gf.to_bytes())
                .map_err(|e| EngineError::Ngf(format!("{}: {e}", path.display())))?;
        }
        self.lock().insert(key, Arc::clone(&gf));
        Ok(gf)
    }
}

/// Tags from here up are dummy wires, hidden and left out of model checks, as in 4nec2
/// (">= 9800 is invisible; >= 9900 is 4nec2 specific").
pub const AUXILIARY_TAG: i32 = nec_geom::AUXILIARY_TAG;

/// How a deck is turned into a model besides what its cards say: 4nec2's auto-segmentation and
/// stepped-radius correction, both off by default.
#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ModelOptions {
    /// Auto-segmentation: at least this many segments per half wavelength on every wire card
    /// (`GW`, `GA`, `GH`) whose segment count is not pinned with a leading `+`. Wires that
    /// already have more keep theirs. Sources, loads, lines and networks keep their place along
    /// their wires ([`crate::segmentation`]). The model is rebuilt for each frequency solved.
    pub autoseg: Option<u32>,
    /// Replace stepped-radius (telescoping) elements with their uniform equivalents
    /// ([`crate::steprad`]), judged at the deck's first frequency.
    pub step_radius: bool,
    /// The frequency auto-segmentation is done for, MHz; the deck's first if not given.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mhz: Option<f64>,
}

/// What a model with [`ModelOptions`] was built from, to build it again at another frequency.
#[derive(Debug)]
struct Source {
    text: String,
    overrides: SymbolTable,
    store: NgfStore,
}

/// A deck parsed, its symbols resolved, its 4nec2 extensions lowered and its structure built.
#[derive(Debug)]
pub struct Model {
    /// The structure.
    pub geom: Geometry,
    /// What the deck asks the solver to do, or why it cannot be solved.
    pub program: Result<Program>,
    /// The deck's `SY` variables, with the values they resolved to.
    pub variables: Vec<Variable>,
    /// Whether 4nec2 extension cards were translated to standard NEC-2.
    pub lowered: bool,
    /// The numerical Green's function this run is built on (`GF`), if any.
    pub base: Option<Arc<GreensFunction>>,
    /// The file name the `GF` card reads it from.
    pub base_name: Option<String>,
    /// The run as resolved numeric deck text (what a `WG` card saves).
    pub run_text: String,
    /// How many runs (`NX`) the deck has; this model is the last.
    pub runs: usize,
    /// Matching networks written into the deck ([`crate::matching`]): the 0-based segment of
    /// each one's source-side port, and the network, whose admittances are recomputed at every
    /// frequency solved.
    pub matching: Vec<(usize, crate::matching::Network)>,
    /// The options the model was built with.
    pub options: ModelOptions,
    /// The frequency auto-segmentation was done for, MHz.
    pub segmented_for: Option<f64>,
    /// Auto-segmentation's changes, one per wire card it refined (4nec2's "Auto-segm log").
    pub autoseg_log: Vec<crate::segmentation::Change>,
    /// The stepped-radius correction's account of each stepped element (4nec2's "Step-rad log").
    pub steprad_log: Vec<String>,
    /// The stepped-radius correction's new wire-card values.
    pub steprad_changes: Vec<crate::steprad::CardChange>,
    source: Option<Arc<Source>>,
}

impl Model {
    /// Prepare deck text as written.
    pub fn new(text: &str) -> Result<Self> {
        Self::with_context(text, &SymbolTable::new(), &NgfStore::default())
    }

    /// Prepare deck text with some `SY` variables pinned (see
    /// [`nec_cards::Deck::resolve_with`]).
    pub fn with_overrides(text: &str, overrides: &SymbolTable) -> Result<Self> {
        Self::with_context(text, overrides, &NgfStore::default())
    }

    /// Prepare deck text with some `SY` variables pinned and numerical Green's functions kept in
    /// `store`.
    ///
    /// A deck of several runs (`NX`) is prepared run by run: an earlier run that writes a
    /// Green's function (`WG`) has it computed for the later ones, and the model is the last run.
    /// A last run's own `WG` is left to [`Model::write_greens_function`].
    ///
    /// An error means not even the structure could be built; a structure whose program cards
    /// are unusable still prepares, with the reason in [`Model::program`].
    pub fn with_context(text: &str, overrides: &SymbolTable, store: &NgfStore) -> Result<Self> {
        Self::with_options(text, overrides, store, &ModelOptions::default())
    }

    /// Prepare deck text as [`Model::with_context`] does, with auto-segmentation or the
    /// stepped-radius correction when `options` asks for them.
    pub fn with_options(
        text: &str,
        overrides: &SymbolTable,
        store: &NgfStore,
        options: &ModelOptions,
    ) -> Result<Self> {
        let Some(k) = options.autoseg else {
            let mut m = Self::build(text, overrides, store, options.step_radius)?;
            m.options = *options;
            return Ok(m);
        };
        let base = Self::build(text, overrides, store, options.step_radius)?;
        let mhz = options.mhz.or_else(|| {
            base.program.as_ref().ok().and_then(|p| p.frequencies.first()).map(|f| f.mhz())
        });
        let mut m = match mhz {
            Some(mhz) => {
                let counts = autoseg_counts(text, &base.geom, k, mhz)?;
                let (refined, changes) = crate::segmentation::resegment_with(text, &counts, overrides)?;
                let mut m = if changes.is_empty() {
                    base
                } else {
                    Self::build(&refined, overrides, store, options.step_radius)?
                };
                m.autoseg_log = changes;
                m.segmented_for = Some(mhz);
                m
            }
            None => base,
        };
        m.options = *options;
        m.source = Some(Arc::new(Source {
            text: text.to_string(),
            overrides: overrides.clone(),
            store: store.clone(),
        }));
        Ok(m)
    }

    /// With auto-segmentation on, the model segmented for `mhz` when this one was segmented for
    /// another frequency; otherwise `None`, and this model serves.
    ///
    /// # Errors
    /// The deck no longer builds at `mhz`.
    pub fn for_frequency(&self, mhz: f64) -> Result<Option<Self>> {
        let (Some(source), Some(done)) = (&self.source, self.segmented_for) else { return Ok(None) };
        if (mhz / done - 1.0).abs() <= 1e-9 {
            return Ok(None);
        }
        let options = ModelOptions { mhz: Some(mhz), ..self.options };
        Self::with_options(&source.text, &source.overrides, &source.store, &options).map(Some)
    }

    fn build(text: &str, overrides: &SymbolTable, store: &NgfStore, step_radius: bool) -> Result<Self> {
        let deck_err = |e: nec_cards::CardError| EngineError::Deck(e.to_string());
        let mut deck = parse_deck(text).map_err(deck_err)?;
        deck.resolve_with(overrides).map_err(deck_err)?;
        let vars = variables(&deck);
        for (name, _) in overrides.iter() {
            if !vars.iter().any(|v| v.name.eq_ignore_ascii_case(name)) {
                return Err(EngineError::UnknownVariable(name.to_string()));
            }
        }
        let lowered = uses_4nec2_extensions(&deck);
        let runs = deck.runs();
        let count = runs.len();
        let mut model = None;
        for (i, run) in runs.iter().enumerate() {
            let m = Self::run(run, store, lowered, &vars, count, step_radius)?;
            if i + 1 < count {
                if let Ok(Some(name)) = m.program.as_ref().map(|p| p.write_ngf.clone()) {
                    store.put(&name, &m.run_text)?;
                }
            } else {
                model = Some(m);
            }
        }
        model.ok_or_else(|| EngineError::Deck("the deck is empty".into()))
    }

    fn run(
        run: &Deck,
        store: &NgfStore,
        lowered: bool,
        vars: &[Variable],
        runs: usize,
        step_radius: bool,
    ) -> Result<Self> {
        let deck_err = |e: nec_cards::CardError| EngineError::Deck(e.to_string());
        let mut deck = lower_4nec2(run).map_err(deck_err)?;
        // GF, when present, is the first geometry card.
        let gf = deck.items.iter().find_map(|it| match it {
            Item::Card(c) if c.mnemonic != "CM" && c.mnemonic != "CE" => Some(c),
            _ => None,
        });
        let base_name = gf
            .filter(|c| c.mnemonic == "GF")
            .map(nec_cards::ngf_file_name);
        let base = match &base_name {
            Some(name) => Some(store.get(name)?),
            None => None,
        };
        let mut geom = Geometry::build_on(&deck, base.as_ref().map(|b| &b.geometry))
            .map_err(|e| EngineError::Geometry(e.to_string()))?;
        deck.resolve_percentages(|tag| geom.by_tag(tag).count())
            .map_err(deck_err)?;
        let mut program = read_program_on(&deck, &geom, base.as_ref().map(|b| &b.program))
            .map_err(|e| EngineError::Program(e.to_string()));
        let mut steprad_log = Vec::new();
        let mut steprad_changes = Vec::new();
        if step_radius {
            // Judged at the deck's first frequency, as 4nec2's Convert.exe judges it at FR's.
            if let Ok(p) = &program {
                let mhz = p.frequencies.first().map_or(nec_core::CVEL_MHZ_M, |f| f.mhz());
                let (changes, report) = crate::steprad::corrections(&deck, &geom, p, mhz);
                steprad_log = report.log;
                if !changes.is_empty() {
                    crate::steprad::apply(&mut deck, &changes);
                    geom = Geometry::build_on(&deck, base.as_ref().map(|b| &b.geometry))
                        .map_err(|e| EngineError::Geometry(e.to_string()))?;
                    program = read_program_on(&deck, &geom, base.as_ref().map(|b| &b.program))
                        .map_err(|e| EngineError::Program(e.to_string()));
                    steprad_changes = changes;
                }
            }
        }
        let matching = deck
            .items
            .iter()
            .filter_map(|it| match it {
                Item::Card(c) if c.mnemonic == "NT" => {
                    let json = c.comment.as_deref()?.trim_start().strip_prefix(crate::matching::MARKER)?;
                    let net: crate::matching::Network = serde_json::from_str(json.trim()).ok()?;
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let (tag, seg) = (c.real_or(0, 0.0).ok()? as i32, c.real_or(1, 0.0).ok()? as usize);
                    Some((geom.resolve_segment(tag, seg)?.index - 1, net))
                }
                _ => None,
            })
            .collect();
        Ok(Self {
            geom,
            program,
            variables: vars.to_vec(),
            lowered,
            base,
            base_name,
            run_text: nec_cards::to_engine_input(&deck).map_err(deck_err)?,
            runs,
            matching,
            options: ModelOptions::default(),
            segmented_for: None,
            autoseg_log: Vec::new(),
            steprad_log,
            steprad_changes,
            source: None,
        })
    }

    /// The program, or why the deck cannot be solved.
    pub fn program(&self) -> Result<&Program> {
        self.program.as_ref().map_err(Clone::clone)
    }

    /// Solve at `f` with `options` — on the numerical Green's function when the run is built on
    /// one, which fixes the frequency.
    pub fn solve(&self, f: Frequency, mut options: SolveOptions) -> Result<Solution> {
        let prog = self.program()?;
        // A matching network's NT card holds its admittances at the design frequency; use the
        // network's own at this one.
        for (port1, net) in &self.matching {
            if let Some((y11, y12, y22)) = net.y_params(f.mhz()) {
                for n in options.networks.iter_mut().filter(|n| n.port1 == *port1) {
                    n.kind = nec_solve::NetworkKind::Admittance { y11, y12, y22 };
                }
            }
        }
        match &self.base {
            None => solve(&self.geom, f, &prog.sources, options)
                .map_err(|e| EngineError::Solve(e.to_string())),
            Some(gf) => {
                if (f.mhz() / gf.frequency.mhz() - 1.0).abs() > 1e-9 {
                    return Err(EngineError::Frequency(format!(
                        "this run is built on a numerical Green's function computed at {} MHz, \
                         and can only be solved there",
                        gf.frequency.mhz()
                    )));
                }
                gf.solve(&self.geom, &prog.sources, &options)
                    .map_err(|e| EngineError::Solve(e.to_string()))
            }
        }
    }

    /// Whether segment `index` (0-based) is on a dummy wire — tag 9800 or above, which 4nec2
    /// hides: a current source's or a matching network's, part of the computation rather than of
    /// the antenna.
    #[must_use]
    pub fn is_auxiliary(&self, index: usize) -> bool {
        self.geom.segments.get(index).is_some_and(|s| s.tag >= AUXILIARY_TAG)
    }

    /// The segment (1-based) each source drives, one per `EX` card: a voltage source's own, a
    /// current source's target rather than its dummy wire.
    #[must_use]
    pub fn source_segments(&self) -> Vec<usize> {
        let Ok(prog) = self.program() else {
            return Vec::new();
        };
        prog.sources
            .iter()
            .map(|src| {
                if !self.is_auxiliary(src.segment) {
                    return src.segment + 1;
                }
                prog.networks
                    .iter()
                    .find(|n| n.port1 == src.segment)
                    .map_or(src.segment + 1, |n| n.port2 + 1)
            })
            .collect()
    }

    /// What the model checks need beyond the structure: the frequency, the kernel, the ground,
    /// and the segments with sources and lumped loads.
    #[must_use]
    pub fn lint_context(&self, frequency: Option<Frequency>) -> nec_geom::LintContext {
        use nec_geom::LintGround;
        use nec_kernel::load::LoadKind;
        let Ok(prog) = self.program() else {
            return nec_geom::LintContext { frequency, ..nec_geom::LintContext::default() };
        };
        let ground = match prog.ground {
            nec_solve::GroundSpec::FreeSpace => LintGround::FreeSpace,
            nec_solve::GroundSpec::Perfect => LintGround::Perfect,
            nec_solve::GroundSpec::Reflection { .. } => LintGround::Reflection,
            nec_solve::GroundSpec::Sommerfeld { .. } => LintGround::Sommerfeld,
            nec_solve::GroundSpec::PerfectNearField { .. } => LintGround::MiniNec,
        };
        // Lumped loads only, and not those spread over a whole wire.
        let lumped: Vec<usize> = prog
            .loads
            .iter()
            .filter(|l| matches!(l.kind, LoadKind::SeriesRlc(_) | LoadKind::ParallelRlc(_) | LoadKind::Impedance { .. }))
            .map(|l| l.segment)
            .collect();
        let loads = lumped
            .iter()
            .copied()
            .filter(|&k| {
                let tag = self.geom.segments[k].tag;
                let on_tag = lumped.iter().filter(|&&j| self.geom.segments[j].tag == tag).count();
                on_tag < self.geom.by_tag(tag).count()
            })
            .map(|k| k + 1)
            .collect();
        nec_geom::LintContext {
            frequency,
            extended_kernel: prog.extended_kernel,
            ground,
            sources: prog.sources.iter().map(|s| s.segment + 1).collect(),
            loads,
            ports: prog.networks.iter().flat_map(|n| [n.port1 + 1, n.port2 + 1]).collect(),
        }
    }

    /// The sources as the deck wrote them, one per `EX` card: a voltage source's input
    /// parameters, and for a 4nec2 current source (`EX 6`, lowered to a voltage source on a
    /// dummy wire driving the segment through a network) the parameters at the driven segment
    /// itself — the impedance the structure presents there, which is what such a source sees.
    /// The dummy source's own impedance is only the reciprocal of that, through the network.
    #[must_use]
    pub fn source_inputs(&self, sol: &Solution) -> Vec<InputParameters> {
        let networks = self
            .program()
            .map(|p| p.networks.as_slice())
            .unwrap_or_default();
        sol.inputs
            .iter()
            .map(|input| {
                if input.tag <= nec_cards::lower::CURRENT_SOURCE_TAG_BASE {
                    return *input;
                }
                networks
                    .iter()
                    .find(|n| n.port1 + 1 == input.segment)
                    .and_then(|n| sol.network_ports.iter().find(|p| p.segment == n.port2 + 1))
                    .copied()
                    .unwrap_or(*input)
            })
            .collect()
    }

    /// Compute and keep the numerical Green's function this run asks for with `WG`, returning its
    /// file name.
    pub fn write_greens_function(&self, store: &NgfStore) -> Result<Option<String>> {
        match self.program()?.write_ngf.clone() {
            Some(name) => {
                store.put(&name, &self.run_text)?;
                Ok(Some(name))
            }
            None => Ok(None),
        }
    }
}


/// Auto-segmentation's new counts, deck line → segments, for the wire cards of `text` that
/// need more than they have: `k` per half wavelength at `mhz`, on each card's own segments in
/// `geom` (copies made by `GM`/`GR` follow their card). Pinned cards and dummy wires keep
/// theirs.
fn autoseg_counts(text: &str, geom: &Geometry, k: u32, mhz: f64) -> Result<HashMap<usize, usize>> {
    let half_wave = nec_core::CVEL_MHZ_M / mhz / 2.0;
    let mut own: HashMap<usize, (f64, usize, i32)> = HashMap::new();
    for (i, s) in geom.segments.iter().enumerate() {
        let e = own.entry(geom.origins.get(i).copied().unwrap_or(0)).or_insert((0.0, 0, s.tag));
        e.0 += s.length();
        e.1 += 1;
    }
    let wires = nec_cards::edit::cards(text, &["GW", "GA", "GH"]).map_err(|e| EngineError::Deck(e.to_string()))?;
    Ok(wires
        .iter()
        .filter(|c| !crate::segmentation::pinned(c))
        .filter_map(|c| {
            let &(length, n, tag) = own.get(&c.line)?;
            if tag >= AUXILIARY_TAG {
                return None;
            }
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let want = (f64::from(k) * length / half_wave).ceil() as usize;
            (want > n).then_some((c.line, want))
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `models/HFActiveFeed/2phased.nec`: two elements fed by current sources, which the
    /// reference engine (run on the lowered deck) reports at its network connection points as
    /// 1.67756E+01−3.46065E+01j and 1.22620E+01+3.42789E+00j ohms.
    #[test]
    fn a_current_source_reports_the_impedance_at_its_segment() {
        let deck = "CM 2-Element Phased Array by K6STI\nCE\nSY L1 = 397\nSY L2 = 386\nSY SP = 169\n\
                    GW\t1\t11\t0\t-L1\t0\t0\tL1\t0\t1\nGW\t2\t11\tSP\t-L2\t0\tSP\tL2\t0\t1\n\
                    GS\t0\t0\tin\nGE\t0\nEX\t6\t1\t6\t00\t-.86\t.508\nEX\t6\t2\t6\t00\t1.0\t0.0\n\
                    GN\t-1\nFR\t0\t1\t0\t0\t 7.1\nEN\n";
        let m = Model::new(deck).expect("builds");
        let prog = m.program().expect("program");
        let f = prog.frequencies[0];
        let sol = m.solve(f, prog.options(f)).expect("solves");
        let ins = m.source_inputs(&sol);
        assert_eq!(ins.len(), 2);
        let close = |z: num_complex::Complex64, re: f64, im: f64| {
            let e = num_complex::Complex64::new(re, im);
            (z - e).norm() / e.norm() < 1e-4
        };
        assert_eq!((ins[0].tag, ins[0].segment), (1, 6));
        assert!(
            close(ins[0].impedance, 16.7756, -34.6065),
            "{}",
            ins[0].impedance
        );
        assert!(close(ins[0].current, -0.86, 0.508), "{}", ins[0].current);
        assert_eq!((ins[1].tag, ins[1].segment), (2, 17));
        assert!(
            close(ins[1].impedance, 12.2620, 3.42789),
            "{}",
            ins[1].impedance
        );
    }

    fn autoseg(text: &str, k: u32) -> Model {
        let options = ModelOptions { autoseg: Some(k), ..ModelOptions::default() };
        Model::with_options(text, &SymbolTable::new(), &NgfStore::default(), &options).expect("builds")
    }

    const COARSE: &str = "CE\nGW 1 3 0 0 -.24 0 0 .24 .001\nGW 2 51 1 0 -.24 1 0 .24 .001\nGW 3 +3 2 0 -.24 2 0 .24 .001\nGE 0\nEX 0 1 2 0 1 0\nFR 0 1 0 0 299.8 0\nEN\n";

    #[test]
    fn auto_segmentation_refines_coarse_wires_and_moves_the_source_with_them() {
        // λ/2 = 0.5 m: 21 per half wave makes 0.48 m into ⌈20.16⌉ = 21 segments.
        let m = autoseg(COARSE, 21);
        let count = |tag| m.geom.segments.iter().filter(|s| s.tag == tag).count();
        assert_eq!((count(1), count(2), count(3)), (21, 51, 3), "finer wires and pinned ones keep theirs");
        assert_eq!(m.autoseg_log.len(), 1);
        assert_eq!((m.autoseg_log[0].from, m.autoseg_log[0].to), (3, 21));
        // The centre segment stays the centre: 2 of 3 is 11 of 21.
        assert_eq!(m.source_segments(), vec![11]);
        assert_eq!(m.segmented_for, Some(299.8));
    }

    #[test]
    fn auto_segmentation_follows_the_frequency_solved() {
        let m = autoseg(COARSE, 21);
        let higher = m.for_frequency(599.6).expect("builds").expect("rebuilt");
        assert_eq!(higher.geom.segments.iter().filter(|s| s.tag == 1).count(), 41);
        assert_eq!(higher.source_segments(), vec![21]);
        assert!(m.for_frequency(299.8).expect("builds").is_none(), "already segmented for it");
        assert!(Model::new(COARSE).expect("builds").for_frequency(599.6).expect("builds").is_none());
        // And a sweep through it solves each frequency on its own segmentation.
        let at = |mhz| m.evaluate(mhz, &crate::EvalOptions::default()).expect("solves").z;
        let direct = |mhz| autoseg(COARSE, 21).for_frequency(mhz).expect("builds").expect("rebuilt").evaluate(mhz, &crate::EvalOptions::default()).expect("solves").z;
        assert_eq!(at(599.6), direct(599.6));
    }
}
