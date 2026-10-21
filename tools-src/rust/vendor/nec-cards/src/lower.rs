//! Lowering 4nec2's extension cards to standard NEC-2, as 4nec2 does before running the engine.
//!
//! 4nec2 accepts card types the NEC-2 engine does not handle the way 4nec2 means them — the
//! engine rejects `LD 6` ("IMPROPER LOAD TYPE") and ignores `EX 6` — and rewrites them into
//! standard cards when it writes the engine's input. The rules here were read from the 4nec2 GUI executable
//! (decompiled, `C:\4nec2-decompiled\4nec2-gui`) and agree with its help file:
//!
//! * **`LD 6` — LC trap** `LD 6 tag from to Q L C [f]`: a coil of unloaded `Q` (0 means 100)
//!   with its loss resistance worked out at the design frequency `f` (MHz; the first `FR`
//!   frequency when omitted), in parallel with `C`. The lossy coil `r + jX`, `X = 2πfL`,
//!   `r = X/Q`, is converted exactly to its parallel equivalent `R' = X²/r + r`,
//!   `X' = r²/X + X`, and written as `LD 1 tag from to R' X'/(2πf) C`.
//! * **`LD 7` — insulated wire** `LD 7 tag from to εr R`: a coating of relative permittivity
//!   `εr` (clamped to 1.02–20) and outer radius `R` around a wire of radius `r`, modelled as
//!   distributed inductance after L. B. Cebik:
//!   `L' = 2e-7 · (εr·R/r)^(1/12) · (1 − 1/εr) · ln(R/r)` H/m, with `R/r ≥ 1`, written as
//!   `LD 2 tag from to 0 L' 0`. `r` is the radius on the first `GW` card carrying the tag (the
//!   last `GW` card when none does, which is what 4nec2's search leaves for tag 0).
//! * **`GN 3` — "MiniNec" ground** `GN 3 . . . εr σ`: perfect ground for the currents and
//!   impedance, the given ground for far fields. Written as `GN 1` followed by
//!   `GD 0 0 0 0 εr σ`, a second ground medium starting at distance zero and height zero (from the
//!   4nec2 help; the engine itself, given a raw `GN 3`, also fills the matrix as for perfect
//!   ground, but reflects an incident plane wave off the finite ground).
//! * **`EX 6` — current source** `EX 6 tag seg . I_re I_im`: the `n`-th `EX` card becomes a
//!   voltage source on a one-segment dummy wire tagged `9900 + n`, 9.9 km up, joined to the
//!   target segment by a network whose only term is `Y12 = j` S:
//!
//!   ```text
//!   GW 9900+n 1  -c 0 9900+n  c 0 9900+n  c/20      c = 0.00051 λ at the design frequency
//!   EX 0 9900+n 1 0  Re(jI) Im(jI)
//!   NT 9900+n 1 tag seg  0 0 0 1 0 0
//!   ```
//!
//!   With `Y22 = 0` the network drives the segment with current `−Y21·V = −j·jI = I`
//!   regardless of what the structure does, which is a current source.

use crate::deck::{Card, Deck, Field, FieldKind, Item};
use crate::error::{CardError, Result};

/// Base tag for the dummy wires carrying current sources.
pub const CURRENT_SOURCE_TAG_BASE: i32 = 9900;

/// Whether a deck uses any of 4nec2's extension cards.
#[must_use]
pub fn uses_4nec2_extensions(deck: &Deck) -> bool {
    deck.items.iter().any(|it| match it {
        Item::Card(c) => is_extension(c),
        _ => false,
    })
}

fn is_extension(c: &Card) -> bool {
    let kind = c.int(0).ok();
    (c.mnemonic == "LD" && matches!(kind, Some(6 | 7)))
        || (c.mnemonic == "EX" && kind == Some(6))
        || (c.mnemonic == "GN" && kind == Some(3))
}

/// Rewrite `LD 6`, `LD 7` and `EX 6` into standard NEC-2 cards. Requires a resolved deck.
pub fn lower_4nec2(deck: &Deck) -> Result<Deck> {
    if !uses_4nec2_extensions(deck) {
        return Ok(deck.clone());
    }
    let design_mhz = design_frequency(deck);
    let wires: Vec<(i32, f64)> = deck
        .cards("GW")
        .filter_map(|c| Some((c.int(0).ok()?, c.real(8).ok()?)))
        .collect();

    let mut items: Vec<Item> = Vec::with_capacity(deck.items.len() + 8);
    let mut dummies: Vec<Card> = Vec::new();
    let mut networks: Vec<Card> = Vec::new();
    let mut ex_index = 0;
    for item in &deck.items {
        let Item::Card(card) = item else {
            items.push(item.clone());
            continue;
        };
        if card.mnemonic == "EX" {
            ex_index += 1;
        }
        if !is_extension(card) {
            items.push(item.clone());
            continue;
        }
        let fail = |reason: &str| CardError::Lowering {
            line: card.line,
            mnemonic: card.mnemonic.clone(),
            reason: reason.to_string(),
        };
        let v = |i: usize| card.real_or(i, 0.0);
        if card.mnemonic == "GN" {
            // GN 3: perfect ground under the antenna, the real ground only for far fields.
            items.push(Item::Card(with_fields(card, &[1.0])));
            items.push(Item::Card(with_fields_named(card, "GD", &[0.0, 0.0, 0.0, 0.0, v(4)?, v(5)?])));
        } else if card.mnemonic == "LD" && card.int(0)? == 6 {
            let mut q = v(4)?;
            if q <= 0.0 {
                q = 100.0;
            }
            let (l, c) = (v(5)?, v(6)?);
            let mut f = v(7)?;
            if f <= 0.0 {
                f = design_mhz.ok_or_else(|| fail("no FR card to take the design frequency from"))?;
            }
            let w = 2.0 * std::f64::consts::PI * f * 1e6;
            let x = w * l;
            let r = x / q;
            let r_par = x * x / r.max(1e-10) + r;
            let x_par = r * r / x.max(1e-10) + x;
            items.push(Item::Card(with_fields(card, &[1.0, v(1)?, v(2)?, v(3)?, r_par, x_par / w, c])));
        } else if card.mnemonic == "LD" {
            let tag = card.int(1)?;
            let wire_r = wires
                .iter()
                .find(|(t, _)| *t == tag)
                .or(wires.last())
                .map(|&(_, r)| r)
                .filter(|&r| r > 0.0)
                .ok_or_else(|| fail("no GW radius for the coated wire"))?;
            let eps = v(4)?.clamp(1.02, 20.0);
            let ratio = (v(5)? / wire_r).max(1.0);
            let l = 2e-7 * (ratio * eps).powf(1.0 / 12.0) * (1.0 - 1.0 / eps) * ratio.ln();
            items.push(Item::Card(with_fields(card, &[2.0, f64::from(tag), v(2)?, v(3)?, 0.0, l, 0.0])));
        } else {
            // EX 6: a voltage source on a dummy wire, coupled through NT.
            let f = design_mhz.ok_or_else(|| fail("no FR card to size the dummy wire"))?;
            let tag = f64::from(CURRENT_SOURCE_TAG_BASE + ex_index);
            let c = 0.00051 * 299.8 / f;
            let (ire, iim) = (v(4)?, v(5)?);
            dummies.push(with_fields_named(card, "GW", &[tag, 1.0, -c, 0.0, tag, c, 0.0, tag, c / 20.0]));
            items.push(Item::Card(with_fields_named(card, "EX", &[0.0, tag, 1.0, 0.0, -iim, ire])));
            networks.push(with_fields_named(
                card,
                "NT",
                &[tag, 1.0, v(1)?, v(2)?, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            ));
        }
    }

    // Dummy wires go last in the geometry, just before GE, so no GS, GM or GX acts on them.
    if !dummies.is_empty() {
        let ge = items
            .iter()
            .position(|it| matches!(it, Item::Card(c) if c.mnemonic == "GE"))
            .ok_or_else(|| CardError::Lowering {
                line: 0,
                mnemonic: "EX".to_string(),
                reason: "a current source needs a GE card to place its dummy wire before".to_string(),
            })?;
        items.splice(ge..ge, dummies.into_iter().map(Item::Card));
    }
    // The network cards must sit together with any TL/NT cards already present: NEC starts a
    // new, separate set of networks at any NT or TL that follows some other card.
    if !networks.is_empty() {
        let last_network = items
            .iter()
            .rposition(|it| matches!(it, Item::Card(c) if c.mnemonic == "NT" || c.mnemonic == "TL"));
        let at = match last_network {
            Some(i) => i + 1,
            None => {
                items
                    .iter()
                    .rposition(|it| matches!(it, Item::Card(c) if c.mnemonic == "EX"))
                    .unwrap_or(items.len())
                    + 1
            }
        };
        items.splice(at..at, networks.into_iter().map(Item::Card));
    }

    Ok(Deck {
        items,
        symbols: deck.symbols.clone(),
    })
}

/// The frequency of the first `FR` card, in MHz.
fn design_frequency(deck: &Deck) -> Option<f64> {
    deck.cards("FR").next().and_then(|c| c.real(4).ok()).filter(|f| *f > 0.0)
}

fn with_fields(card: &Card, values: &[f64]) -> Card {
    with_fields_named(card, &card.mnemonic, values)
}

fn with_fields_named(card: &Card, mnemonic: &str, values: &[f64]) -> Card {
    Card {
        mnemonic: mnemonic.to_string(),
        fields: values
            .iter()
            .map(|&v| Field {
                text: format!("{v}"),
                value: Some(v),
                kind: FieldKind::Numeric,
            })
            .collect(),
        comment: None,
        line: card.line,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_deck;

    fn lowered(src: &str) -> Deck {
        let mut d = parse_deck(src).expect("parses");
        d.resolve().expect("resolves");
        lower_4nec2(&d).expect("lowers")
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() <= 1e-12 * b.abs().max(1e-300)
    }

    fn card<'a>(d: &'a Deck, m: &str) -> Vec<&'a Card> {
        d.cards(m).collect()
    }

    #[test]
    fn an_lc_trap_becomes_its_exact_parallel_equivalent() {
        // Q = 100, L = 5.5 uH, C = 45 pF at 7 MHz: X = 241.9 ohms, r = 2.419 ohms.
        let d = lowered("GW 1 9 0 0 0 1 0 0 1e-3\nGE 0\nLD 6 1 5 5 100 5.5e-6 45e-12\nFR 0 1 0 0 7 0\nEN\n");
        let ld = card(&d, "LD")[0];
        assert_eq!(ld.int(0).unwrap(), 1);
        let x = 2.0 * std::f64::consts::PI * 7e6 * 5.5e-6;
        let r = x / 100.0;
        assert!((ld.real(4).unwrap() - r * (1.0 + 100.0 * 100.0)).abs() < 1e-9 * x * 100.0);
        assert!((ld.real(5).unwrap() - 5.5e-6 * (1.0 + 1e-4)).abs() < 1e-18);
        assert!(close(ld.real(6).unwrap(), 45e-12));
    }

    #[test]
    fn a_trap_with_no_q_gets_100_and_its_own_frequency_if_given() {
        let d = lowered("GW 1 9 0 0 0 1 0 0 1e-3\nGE 0\nLD 6 1 5 5 0 1e-6 1e-12 10\nFR 0 1 0 0 7 0\nEN\n");
        let ld = card(&d, "LD")[0];
        let x = 2.0 * std::f64::consts::PI * 10e6 * 1e-6;
        assert!((ld.real(4).unwrap() - (x / 100.0) * (1.0 + 1e4)).abs() < 1e-6);
    }

    #[test]
    fn a_coated_wire_becomes_distributed_inductance() {
        // HFsimple\36inv_coat.nec: 0.5 mm wire, 1 mm PVC coat of permittivity 4.5.
        let d = lowered("GW 1 9 0 0 0 1 0 0 .5mm\nGE 0\nLD 7 0 0 0 4.5 1mm\nFR 0 1 0 0 7 0\nEN\n");
        let ld = card(&d, "LD")[0];
        assert_eq!(ld.int(0).unwrap(), 2);
        let expect = 2e-7 * (2.0_f64 * 4.5).powf(1.0 / 12.0) * (1.0 - 1.0 / 4.5) * 2.0_f64.ln();
        assert!((ld.real(5).unwrap() - expect).abs() < 1e-20, "{}", ld.real(5).unwrap());
        assert!(ld.real(4).unwrap().abs() < 1e-300 && ld.real(6).unwrap().abs() < 1e-300);
    }

    #[test]
    fn a_current_source_becomes_a_dummy_wire_a_voltage_source_and_a_network() {
        let d = lowered(
            "GW 1 9 0 0 0 1 0 0 1e-3\nGE 0\nEX 0 1 2 0 1 0\nEX 6 1 5 0 2 0.5\nFR 0 1 0 0 14 0\nEN\n",
        );
        // The second EX card: tag 9902.
        let gw = card(&d, "GW");
        assert_eq!(gw.len(), 2);
        let c = 0.00051 * 299.8 / 14.0;
        assert_eq!(gw[1].int(0).unwrap(), 9902);
        assert!((gw[1].real(5).unwrap() - c).abs() < 1e-15);
        assert!((gw[1].real(8).unwrap() - c / 20.0).abs() < 1e-15);
        // Placed before GE.
        let ge_pos = d.items.iter().position(|i| matches!(i, Item::Card(c) if c.mnemonic == "GE")).unwrap();
        let gw_pos = d.items.iter().rposition(|i| matches!(i, Item::Card(c) if c.mnemonic == "GW")).unwrap();
        assert!(gw_pos < ge_pos);
        // V = j·I.
        let ex = card(&d, "EX");
        assert_eq!((ex[1].int(0).unwrap(), ex[1].int(1).unwrap()), (0, 9902));
        assert!(close(ex[1].real(4).unwrap(), -0.5) && close(ex[1].real(5).unwrap(), 2.0));
        let nt = card(&d, "NT");
        assert_eq!(nt.len(), 1);
        let f: Vec<i32> = (0..10).map(|i| nt[0].int(i).unwrap()).collect();
        assert_eq!(f, vec![9902, 1, 1, 5, 0, 0, 0, 1, 0, 0]);
    }

    #[test]
    fn mininec_ground_becomes_perfect_ground_plus_a_far_field_medium() {
        let d = lowered("GW 1 9 0 0 0 0 0 1 1e-3
GE 1
GN 3 0 0 0 14 0.006
FR 0 1 0 0 7 0
EN
");
        assert_eq!(card(&d, "GN")[0].int(0).unwrap(), 1);
        let gd = card(&d, "GD")[0];
        assert!(close(gd.real(4).unwrap(), 14.0) && close(gd.real(5).unwrap(), 0.006));
        assert_eq!(gd.fields.len(), 6, "GD 0 0 0 0 eps sig: no cliff distance or height");
    }

    #[test]
    fn generated_networks_join_existing_ones() {
        let d = lowered(
            "GW 1 9 0 0 0 1 0 0 1e-3\nGW 2 1 0 1 0 0 1.1 0 1e-3\nGE 0\nEX 6 1 5 0 1 0\n\
             TL 2 1 1 3 50 0\nFR 0 1 0 0 14 0\nEN\n",
        );
        let mnemonics: Vec<&str> = d
            .items
            .iter()
            .filter_map(|i| match i {
                Item::Card(c) => Some(c.mnemonic.as_str()),
                _ => None,
            })
            .collect();
        let tl = mnemonics.iter().position(|m| *m == "TL").unwrap();
        assert_eq!(mnemonics[tl + 1], "NT", "{mnemonics:?}");
    }
}
