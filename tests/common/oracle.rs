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

//! A naive explicit-state CTL model checker, used as a test oracle.
//!
//! It is deliberately written in the most direct way possible
//! (plain `HashSet<usize>` operations, simple fixpoint loops) so that
//! its correctness can be checked by eye. It shares no code with the
//! BDD-based implementation under test.
//!
//! Like the crate, it assumes a *total* transition relation — which
//! `KripkeStructure::new` now enforces at construction.

use std::collections::HashSet;

use citreelo::ctl::{BinaryCTLOperator, CTLFormula, CTLFormulaLeaf, UnaryCTLOperator};
use citreelo::kripke::{AtomicProposition, KripkeStructure};

use crate::common::model::{TestAtomicProp, TestDomainOfAp};

type States = HashSet<usize>;
type Kripke = KripkeStructure<TestDomainOfAp>;

fn all_states(kripke: &Kripke) -> States {
    (0..kripke.states().len()).collect()
}

fn complement(kripke: &Kripke, s: &States) -> States {
    all_states(kripke).difference(s).copied().collect()
}

/// { s | exists s' in succ(s), s' in target }
fn weak_pre(kripke: &Kripke, target: &States) -> States {
    (0..kripke.states().len())
        .filter(|s| {
            kripke.states()[*s]
                .outgoing_transitions_targets
                .iter()
                .any(|t| target.contains(t))
        })
        .collect()
}

/// { s | forall s' in succ(s), s' in target }  (vacuously true on deadlocks)
fn strong_pre(kripke: &Kripke, target: &States) -> States {
    (0..kripke.states().len())
        .filter(|s| {
            kripke.states()[*s]
                .outgoing_transitions_targets
                .iter()
                .all(|t| target.contains(t))
        })
        .collect()
}

/// Least fixpoint of  Z = seed ∪ (constraint ∩ pre(Z)).
/// With constraint = all states this computes AF/EF; otherwise AU/EU.
fn until_lfp(
    kripke: &Kripke,
    constraint: &States,
    seed: &States,
    pre: impl Fn(&Kripke, &States) -> States,
) -> States {
    let mut current = seed.clone();
    loop {
        let step: States = constraint
            .intersection(&pre(kripke, &current))
            .copied()
            .collect();
        let next: States = current.union(&step).copied().collect();
        if next == current {
            return current;
        }
        current = next;
    }
}

/// Greatest fixpoint of  Z = seed ∩ pre(Z), starting from seed.
fn global_gfp(kripke: &Kripke, seed: &States, pre: impl Fn(&Kripke, &States) -> States) -> States {
    let mut current = seed.clone();
    loop {
        let next: States = current
            .intersection(&pre(kripke, &current))
            .copied()
            .collect();
        if next == current {
            return current;
        }
        current = next;
    }
}

/// Computes the set of states satisfying `phi`, by direct application
/// of CTL semantics on the explicit state space.
pub fn oracle_sat_set(kripke: &Kripke, phi: &CTLFormula<TestAtomicProp>) -> States {
    let all = all_states(kripke);
    match phi {
        CTLFormula::Leaf(leaf) => match leaf {
            CTLFormulaLeaf::True => all,
            CTLFormulaLeaf::False => States::new(),
            CTLFormulaLeaf::AtomicProp(ap) => kripke
                .states()
                .iter()
                .enumerate()
                .filter(|(_, state)| ap.is_satisfied_on_state_domain(&state.value_in_domain))
                .map(|(i, _)| i)
                .collect(),
        },
        CTLFormula::Unary(op, phi1) => {
            let s1 = oracle_sat_set(kripke, phi1);
            match op {
                UnaryCTLOperator::Not => complement(kripke, &s1),
                UnaryCTLOperator::EX => weak_pre(kripke, &s1),
                UnaryCTLOperator::AX => strong_pre(kripke, &s1),
                UnaryCTLOperator::EF => until_lfp(kripke, &all, &s1, weak_pre),
                UnaryCTLOperator::AF => until_lfp(kripke, &all, &s1, strong_pre),
                UnaryCTLOperator::EG => global_gfp(kripke, &s1, weak_pre),
                UnaryCTLOperator::AG => global_gfp(kripke, &s1, strong_pre),
            }
        }
        CTLFormula::Binary(op, phi1, phi2) => {
            let s1 = oracle_sat_set(kripke, phi1);
            let s2 = oracle_sat_set(kripke, phi2);
            match op {
                BinaryCTLOperator::And => s1.intersection(&s2).copied().collect(),
                BinaryCTLOperator::Or => s1.union(&s2).copied().collect(),
                BinaryCTLOperator::Imply => complement(kripke, &s1).union(&s2).copied().collect(),
                BinaryCTLOperator::Iff => {
                    let both: States = s1.intersection(&s2).copied().collect();
                    let neither: States = complement(kripke, &s1)
                        .intersection(&complement(kripke, &s2))
                        .copied()
                        .collect();
                    both.union(&neither).copied().collect()
                }
                BinaryCTLOperator::EU => until_lfp(kripke, &s1, &s2, weak_pre),
                BinaryCTLOperator::AU => until_lfp(kripke, &s1, &s2, strong_pre),
            }
        }
    }
}
