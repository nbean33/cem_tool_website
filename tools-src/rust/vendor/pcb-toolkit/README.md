# pcb-toolkit

PCB design calculations as a Rust library: transmission-line impedance, differential pairs,
current capacity, fusing, vias, padstacks, signal-integrity rules of thumb, and more.

Formulas come from the published literature and are cited in each module. Where this crate
diverges from Saturn PCB Toolkit — which it started as a reimplementation of — the divergence
is deliberate, tested, and documented with its numerical cause. See the workspace
[`CLAUDE.md`](../../CLAUDE.md) for the goal and [`VALIDATION.md`](../../VALIDATION.md) for
what is verified against what.

```rust
use pcb_toolkit::impedance::microstrip::{self, MicrostripInput};

let result = microstrip::calculate(&MicrostripInput {
    width: 10.0,
    height: 5.0,
    thickness: 1.4,
    er: 4.6,
    frequency: 0.0,
})?;

println!("Zo = {:.2} Ohms", result.zo);
println!("Er_eff = {:.4}", result.er_eff);
println!("Tpd = {:.2} ps/in", result.tpd_ps_per_in);
```

## Error handling

Every public calculation function returns `Result<T, CalcError>`. Negative dimensions,
out-of-range dielectric constants, unknown materials, impossible geometries and **non-finite
inputs** are all rejected — never silently clamped, and never allowed to reach the output as
`NaN`. The last of those is worth stating explicitly: every range check is written `x <= 0.0`,
which is *false* for NaN, so each entry point validates finiteness first.

Where a model has a published accuracy band, leaving it is not an error — the closed form is
still the best number available. Those cases return their result with
`ImpedanceResult::accuracy_warnings` populated, so the caveat travels with the number.

## Calculators

| Module | Model | Notes |
|---|---|---|
| `impedance::microstrip` | Hammerstad-Jensen 1980 | Kirschning-Jansen dispersion |
| `impedance::stripline` | Cohn 1955, both branches | Reference vector; reports its accuracy band |
| `impedance::offset_stripline` | Cohn, composed | Two centred striplines in parallel |
| `impedance::embedded` | Brooks | No reference vector |
| `impedance::coplanar` | Wadell | Complete elliptic integral form |
| `wires` | HSDD ROUND / COAX / TWIST | Round wire over a plane, coax, twisted pair |
| `differential` | Coupled-line theory | 5 topologies |
| `current` | IPC-2221A and IPC-2152 | DC resistance, skin depth, voltage drop |
| `fusing` | Onderdonk | Trace and area forms |
| `via` | Coaxial model | C, L, Z, resonance |
| `inductor` | Mohan/Wheeler; HSDD loops | Planar spirals and single-turn loops |
| `padstack` | Geometry | Thru-hole, corner-to-corner |
| `spacing` | IPC-2221C | 8 device classes, linear above 500 V |
| `crosstalk` | NEXT estimation | Microstrip only |
| `signal` | HSDD rules of thumb | Knee frequency, critical length, Γ/VSWR/return loss, Er from delay |
| `reactance` | Xc, Xl, f_res | Plus the forms an edge sees: `Tr/(πC)`, `πL/Tr` |
| `resistance` | HSDD RESIST | Copper temperature factor, plane spreading resistance |
| `pdn` | Target impedance | Plane capacitance, Xc |
| `thermal` | T_j = R·P + T_ambient | |
| `ohms_law` | V=IR, attenuators | LED bias, Pi/T pads, series/parallel R/C/L |
| `ppm` | PPM ↔ Hz | Crystal load capacitance |
| `wavelength` | λ = c/(f√Er) | λ and its common fractions |
| `wire_gauge` | AWG table | Both directions: gauge → properties, diameter → gauge |

## Supporting modules

- **`materials`** — 45 substrate materials with Er and Tg, extracted from the Saturn binary.
  Case-insensitive lookup. The authoritative Er mapping is
  `docs/notes/materials-er-mapping.md`; it is non-sequential, so a reader should not infer it
  from list order.
- **`copper`** — copper weight to thickness (0.25 oz through 5 oz), plating thickness, etch
  factor cross-section correction. Every enum has a `FromStr` with documented aliases, so the
  CLI and the desktop app accept the same spellings.
- **`units`** — `Length`, `Freq`, `Capacitance`, `Inductance`, `Temperature`, each parsing a
  number plus a unit suffix into the canonical internal unit (mils, Hz, F, H, °C).
- **`constants`** — 9 constants, mostly derived from one another so they cannot drift apart.
  Nothing is kept here without a caller.

## Design

- **f64 everywhere.** No arbitrary-precision or unit-of-measure crates.
- **Canonical internal units.** Mils for length, Hz for frequency; conversion at the API
  boundary.
- **No I/O.** The library computes; front ends read files and print.
- **Minimal dependencies.** `thiserror` for errors, `serde` for serialization. Nothing else,
  and no dependency without a caller.

## License

Licensed under either of [Apache License, Version 2.0](../../LICENSE-APACHE) or
[MIT License](../../LICENSE-MIT) at your option.
