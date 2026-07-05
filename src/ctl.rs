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

use std::{collections::HashSet, hash::Hash};

/// The unary connectives of CTL : boolean negation and the six
/// path-quantified temporal operators on a single sub-formula.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum UnaryCTLOperator {
    /// boolean negation `!φ`
    Not,
    /// `AX φ` : φ holds at the next state of every path
    AX,
    /// `EX φ` : φ holds at the next state of some path
    EX,
    /// `AF φ` : on every path, φ eventually holds
    AF,
    /// `EF φ` : on some path, φ eventually holds
    EF,
    /// `AG φ` : on every path, φ holds at every state
    AG,
    /// `EG φ` : on some path, φ holds at every state
    EG,
}

/// The binary connectives of CTL : the boolean connectives and the
/// path-quantified until operators.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum BinaryCTLOperator {
    /// conjunction `φ & ψ`
    And,
    /// disjunction `φ | ψ`
    Or,
    /// implication `φ => ψ`
    Imply,
    /// equivalence `φ <=> ψ`
    Iff,
    /// `A[φ U ψ]` : on every path, φ holds until ψ eventually holds
    AU,
    /// `E[φ U ψ]` : on some path, φ holds until ψ eventually holds
    EU,
}

/// The leaves of a [CTLFormula] : the boolean constants and the
/// user-defined atomic propositions.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum CTLFormulaLeaf<AP> {
    True,
    False,
    AtomicProp(AP),
}

/// The abstract syntax tree of a CTL formula over atomic propositions
/// of type `AP`.
///
/// Formulae can be built directly, or parsed from a concrete syntax
/// with operator precedence (see [crate::parser]).
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum CTLFormula<AP> {
    Leaf(CTLFormulaLeaf<AP>),
    // ***
    Unary(UnaryCTLOperator, Box<CTLFormula<AP>>),
    Binary(BinaryCTLOperator, Box<CTLFormula<AP>>, Box<CTLFormula<AP>>),
}

/// The leaves occurring in a [CTLFormula], as returned by
/// [CTLFormula::collect_leaves] : the set of distinct atomic
/// propositions, and, if present, one representative `true` leaf
/// and one representative `false` leaf.
pub struct CollectedLeaves<'a, AP> {
    pub atoms: HashSet<&'a CTLFormula<AP>>,
    pub true_formula: Option<&'a CTLFormula<AP>>,
    pub false_formula: Option<&'a CTLFormula<AP>>,
}

impl<AP: Clone + PartialEq + Eq + Hash> CTLFormula<AP> {
    pub fn collect_leaves(&self) -> CollectedLeaves<'_, AP> {
        let mut atoms = HashSet::new();
        let mut true_formula = None;
        let mut false_formula = None;
        self.collect_leaves_rec(&mut atoms, &mut true_formula, &mut false_formula);
        CollectedLeaves {
            atoms,
            true_formula,
            false_formula,
        }
    }

    fn collect_leaves_rec<'a>(
        &'a self,
        atoms: &mut HashSet<&'a CTLFormula<AP>>,
        true_formula: &mut Option<&'a CTLFormula<AP>>,
        false_formula: &mut Option<&'a CTLFormula<AP>>,
    ) {
        match self {
            x @ CTLFormula::Leaf(leaf) => match leaf {
                CTLFormulaLeaf::True => {
                    if true_formula.is_none() {
                        *true_formula = Some(x);
                    }
                }
                CTLFormulaLeaf::False => {
                    if false_formula.is_none() {
                        *false_formula = Some(x);
                    }
                }
                CTLFormulaLeaf::AtomicProp(_) => {
                    atoms.insert(x);
                }
            },
            CTLFormula::Unary(_, phi1) => {
                phi1.collect_leaves_rec(atoms, true_formula, false_formula);
            }
            CTLFormula::Binary(_, phi1, phi2) => {
                phi1.collect_leaves_rec(atoms, true_formula, false_formula);
                phi2.collect_leaves_rec(atoms, true_formula, false_formula);
            }
        };
    }
}
