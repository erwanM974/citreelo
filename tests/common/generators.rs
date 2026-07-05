/*
Copyright 2025 Erwan Mahe (github.com/erwanM974)

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

//! Deterministic generators for CTL formulae and Kripke structures,
//! plus a printer producing the test grammar's concrete syntax.
//!
//! No external RNG dependency: a small seeded LCG keeps every
//! "random" test fully reproducible.

use citreelo::ctl::{BinaryCTLOperator, CTLFormula, CTLFormulaLeaf, UnaryCTLOperator};
use citreelo::kripke::{KripkeState, KripkeStructure};

use crate::common::model::{TestAtomicProp, TestDomainOfAp, doap};

pub const UNARY_OPS: [UnaryCTLOperator; 7] = [
    UnaryCTLOperator::Not,
    UnaryCTLOperator::AX,
    UnaryCTLOperator::EX,
    UnaryCTLOperator::AF,
    UnaryCTLOperator::EF,
    UnaryCTLOperator::AG,
    UnaryCTLOperator::EG,
];

pub const BINARY_OPS: [BinaryCTLOperator; 6] = [
    BinaryCTLOperator::And,
    BinaryCTLOperator::Or,
    BinaryCTLOperator::Imply,
    BinaryCTLOperator::Iff,
    BinaryCTLOperator::AU,
    BinaryCTLOperator::EU,
];

/// All leaf formulae: true, false, p, q, r.
pub fn leaves() -> Vec<CTLFormula<TestAtomicProp>> {
    vec![
        CTLFormula::Leaf(CTLFormulaLeaf::True),
        CTLFormula::Leaf(CTLFormulaLeaf::False),
        CTLFormula::Leaf(CTLFormulaLeaf::AtomicProp(TestAtomicProp::P)),
        CTLFormula::Leaf(CTLFormulaLeaf::AtomicProp(TestAtomicProp::Q)),
        CTLFormula::Leaf(CTLFormulaLeaf::AtomicProp(TestAtomicProp::R)),
    ]
}

/// Every CTL formula made of a single operator applied to leaves
/// (plus the leaves themselves):
/// 5 + 7x5 + 6x5x5 = 190 formulae.
pub fn all_single_operator_formulas() -> Vec<CTLFormula<TestAtomicProp>> {
    let leaves_set = leaves();
    let mut all = leaves_set.clone();
    for op in UNARY_OPS {
        for phi in &leaves_set {
            all.push(CTLFormula::Unary(op.clone(), Box::new(phi.clone())));
        }
    }
    for op in BINARY_OPS {
        for phi1 in &leaves_set {
            for phi2 in &leaves_set {
                all.push(CTLFormula::Binary(
                    op.clone(),
                    Box::new(phi1.clone()),
                    Box::new(phi2.clone()),
                ));
            }
        }
    }
    all
}

/// Every composition of two operators: each operator applied to
/// operands drawn from { p, q, op(p) for each unary op, p op q for
/// each binary op } (15 operands), so that every ordered pair
/// (outer operator, inner operator) occurs.
/// 7x15 + 6x15x15 = 1455 formulae.
///
/// A blind cartesian enumeration of all depth-2 formulae would yield
/// ~218k formulae instead, far too many for a unit test, without
/// covering meaningfully more operator interactions.
pub fn all_operator_pair_formulas() -> Vec<CTLFormula<TestAtomicProp>> {
    let p = CTLFormula::Leaf(CTLFormulaLeaf::AtomicProp(TestAtomicProp::P));
    let q = CTLFormula::Leaf(CTLFormulaLeaf::AtomicProp(TestAtomicProp::Q));
    let mut operands = vec![p.clone(), q.clone()];
    for op in UNARY_OPS {
        operands.push(CTLFormula::Unary(op.clone(), Box::new(p.clone())));
    }
    for op in BINARY_OPS {
        operands.push(CTLFormula::Binary(
            op.clone(),
            Box::new(p.clone()),
            Box::new(q.clone()),
        ));
    }
    let mut all = vec![];
    for op in UNARY_OPS {
        for phi in &operands {
            all.push(CTLFormula::Unary(op.clone(), Box::new(phi.clone())));
        }
    }
    for op in BINARY_OPS {
        for phi1 in &operands {
            for phi2 in &operands {
                all.push(CTLFormula::Binary(
                    op.clone(),
                    Box::new(phi1.clone()),
                    Box::new(phi2.clone()),
                ));
            }
        }
    }
    all
}

/// Minimal deterministic pseudo-random generator (64-bit LCG,
/// Knuth MMIX constants). Good enough for test-case diversity.
pub struct Lcg(u64);

impl Lcg {
    pub fn new(seed: u64) -> Self {
        Lcg(seed
            .wrapping_mul(2862933555777941757)
            .wrapping_add(3037000493))
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        // the high bits of an LCG are the most random ones
        self.0 >> 16
    }
    pub fn below(&mut self, n: usize) -> usize {
        assert!(n > 0);
        (self.next_u64() % (n as u64)) as usize
    }
}

fn random_formula_rec(rng: &mut Lcg, max_depth: usize) -> CTLFormula<TestAtomicProp> {
    // at depth 0, or with probability 1/4 before that, emit a leaf
    if max_depth == 0 || rng.below(4) == 0 {
        let leaves = leaves();
        return leaves[rng.below(leaves.len())].clone();
    }
    if rng.below(2) == 0 {
        let op = UNARY_OPS[rng.below(UNARY_OPS.len())].clone();
        CTLFormula::Unary(op, Box::new(random_formula_rec(rng, max_depth - 1)))
    } else {
        let op = BINARY_OPS[rng.below(BINARY_OPS.len())].clone();
        CTLFormula::Binary(
            op,
            Box::new(random_formula_rec(rng, max_depth - 1)),
            Box::new(random_formula_rec(rng, max_depth - 1)),
        )
    }
}

/// `count` pseudo-random formulae of depth <= `max_depth`,
/// fully determined by `seed`.
pub fn random_formulas(
    seed: u64,
    count: usize,
    max_depth: usize,
) -> Vec<CTLFormula<TestAtomicProp>> {
    let mut rng = Lcg::new(seed);
    (0..count)
        .map(|_| random_formula_rec(&mut rng, max_depth))
        .collect()
}

/// A pseudo-random Kripke structure with a total transition relation
/// (every state has between 1 and `max_fanout` successors), fully
/// determined by `seed`. Duplicate targets are allowed on purpose:
/// the implementation must tolerate them.
pub fn random_total_kripke(
    seed: u64,
    n_states: usize,
    max_fanout: usize,
) -> KripkeStructure<TestDomainOfAp> {
    assert!(n_states >= 1 && max_fanout >= 1);
    let mut rng = Lcg::new(seed);
    let states = (0..n_states)
        .map(|_| {
            let mut atoms = vec![];
            if rng.below(2) == 0 {
                atoms.push(TestAtomicProp::P);
            }
            if rng.below(2) == 0 {
                atoms.push(TestAtomicProp::Q);
            }
            if rng.below(2) == 0 {
                atoms.push(TestAtomicProp::R);
            }
            let fanout = 1 + rng.below(max_fanout);
            let targets = (0..fanout).map(|_| rng.below(n_states)).collect();
            KripkeState::new(doap(&atoms), targets)
        })
        .collect();
    KripkeStructure::new(states).expect("generated structures are total by construction")
}

/// Binding strength of a formula's top operator, mirroring the
/// precedence levels of the crate's parser (see src/parser.rs):
/// 1 = `<=>`, 2 = `=>`, 3 = `|`, 4 = `&`, 5 = prefix operators,
/// 6 = self-delimiting (leaves, `A[..U..]`, `E[..U..]`).
fn precedence_level(phi: &CTLFormula<TestAtomicProp>) -> u8 {
    match phi {
        CTLFormula::Leaf(_) => 6,
        CTLFormula::Unary(_, _) => 5,
        CTLFormula::Binary(op, _, _) => match op {
            BinaryCTLOperator::Iff => 1,
            BinaryCTLOperator::Imply => 2,
            BinaryCTLOperator::Or => 3,
            BinaryCTLOperator::And => 4,
            BinaryCTLOperator::AU | BinaryCTLOperator::EU => 6,
        },
    }
}

/// Prints a formula in the concrete syntax accepted by the parser,
/// inserting parentheses only where precedence requires them, so that
/// `parse(&formula_to_string(phi)) == phi` for every formula.
pub fn formula_to_string(phi: &CTLFormula<TestAtomicProp>) -> String {
    fmt_at_level(phi, 0)
}

/// `min_level` is the binding strength the context requires:
/// weaker-binding sub-formulae get parenthesized.
fn fmt_at_level(phi: &CTLFormula<TestAtomicProp>, min_level: u8) -> String {
    let body = match phi {
        CTLFormula::Leaf(leaf) => match leaf {
            CTLFormulaLeaf::True => "true".to_string(),
            CTLFormulaLeaf::False => "false".to_string(),
            CTLFormulaLeaf::AtomicProp(TestAtomicProp::P) => "p".to_string(),
            CTLFormulaLeaf::AtomicProp(TestAtomicProp::Q) => "q".to_string(),
            CTLFormulaLeaf::AtomicProp(TestAtomicProp::R) => "r".to_string(),
        },
        CTLFormula::Unary(op, phi1) => {
            let op_str = match op {
                UnaryCTLOperator::Not => "!",
                UnaryCTLOperator::AX => "AX ",
                UnaryCTLOperator::EX => "EX ",
                UnaryCTLOperator::AF => "AF ",
                UnaryCTLOperator::EF => "EF ",
                UnaryCTLOperator::AG => "AG ",
                UnaryCTLOperator::EG => "EG ",
            };
            format!("{}{}", op_str, fmt_at_level(phi1, 5))
        }
        CTLFormula::Binary(op, phi1, phi2) => {
            match op {
                // left-associative: the right operand must bind strictly
                // tighter to round-trip to the same tree
                BinaryCTLOperator::Iff => {
                    format!("{} <=> {}", fmt_at_level(phi1, 1), fmt_at_level(phi2, 2))
                }
                BinaryCTLOperator::Or => {
                    format!("{} | {}", fmt_at_level(phi1, 3), fmt_at_level(phi2, 4))
                }
                BinaryCTLOperator::And => {
                    format!("{} & {}", fmt_at_level(phi1, 4), fmt_at_level(phi2, 5))
                }
                // right-associative: mirrored
                BinaryCTLOperator::Imply => {
                    format!("{} => {}", fmt_at_level(phi1, 3), fmt_at_level(phi2, 2))
                }
                // brackets delimit the operands: never parenthesize
                BinaryCTLOperator::AU => {
                    format!("A[{} U {}]", fmt_at_level(phi1, 0), fmt_at_level(phi2, 0))
                }
                BinaryCTLOperator::EU => {
                    format!("E[{} U {}]", fmt_at_level(phi1, 0), fmt_at_level(phi2, 0))
                }
            }
        }
    };
    if precedence_level(phi) < min_level {
        format!("({})", body)
    } else {
        body
    }
}
