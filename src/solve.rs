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

use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::rc::Rc;

use biodivine_lib_bdd::*;

use crate::bdd::KripkeStructureBddRepresentation;
use crate::bdd::PreImageKind;
use crate::ctl::*;
use crate::kripke::*;

/// The reasons for which [CtlModelChecker::is_ctl_formula_sat]
/// (and the [is_ctl_formula_sat] convenience function)
/// may reject their input.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CtlModelCheckingError {
    /// An initial state id does not correspond to a state
    /// of the Kripke structure.
    OutOfRangeInitialState {
        initial_state_id: usize,
        num_states: usize,
    },
}

impl std::fmt::Display for CtlModelCheckingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CtlModelCheckingError::OutOfRangeInitialState {
                initial_state_id,
                num_states,
            } => {
                write!(
                    f,
                    "initial state {} is out of range : \
                     the Kripke structure has only {} states",
                    initial_state_id, num_states
                )
            }
        }
    }
}

impl std::error::Error for CtlModelCheckingError {}

/// A CTL model checker for a given Kripke structure.
///
/// Building the checker precomputes the BDD representation of the
/// structure (in particular its transition relation), which is the
/// expensive part of symbolic model checking; the checker then answers
/// any number of queries without rebuilding it.
///
/// The convenience functions [get_sat_set] and [is_ctl_formula_sat]
/// build a fresh checker on every call: prefer constructing a
/// [CtlModelChecker] when checking several formulae against the same
/// structure.
pub struct CtlModelChecker<'a, DOAP> {
    kripke: &'a KripkeStructure<DOAP>,
    bdd_repr: KripkeStructureBddRepresentation,
}

impl<'a, DOAP> CtlModelChecker<'a, DOAP> {
    pub fn new(kripke: &'a KripkeStructure<DOAP>) -> Self {
        let bdd_repr = KripkeStructureBddRepresentation::from_kripke_structure(kripke);
        Self { kripke, bdd_repr }
    }

    /// The Kripke structure this checker operates on.
    pub fn kripke(&self) -> &'a KripkeStructure<DOAP> {
        self.kripke
    }

    /// BDD over the current-state variables representing the set of
    /// states satisfying `formula`.
    fn get_sat_set_as_bdd<AP: AtomicProposition<DOAP> + PartialEq + Eq + Clone + Hash>(
        &self,
        formula: &CTLFormula<AP>,
    ) -> Rc<Bdd> {
        let (mut sub_formulae_memoizer, true_bdd) =
            initialize_memoizer_at_leaves(self.kripke, &self.bdd_repr, formula.collect_leaves());
        get_ctl_subformula_sat_set_rec(
            &self.bdd_repr,
            &true_bdd,
            &mut sub_formulae_memoizer,
            formula,
        )
    }

    /// Computes the set of ids of the states satisfying `formula`.
    pub fn get_sat_set<AP: AtomicProposition<DOAP> + PartialEq + Eq + Clone + Hash>(
        &self,
        formula: &CTLFormula<AP>,
    ) -> HashSet<usize> {
        let sat_set_bdd = self.get_sat_set_as_bdd(formula);
        let mut states = HashSet::new();
        for st_id in 0..self.kripke.states().len() {
            let bdd_with_only_that_state = self.bdd_repr.get_state_formula(st_id);
            if !sat_set_bdd.and(&bdd_with_only_that_state).is_false() {
                states.insert(st_id);
            }
        }
        states
    }

    /// Checks whether `formula` is satisfied from the given set of
    /// initial states, i.e., whether every initial state belongs to
    /// the satisfaction set of `formula`.
    ///
    /// Fails with [CtlModelCheckingError::OutOfRangeInitialState] if
    /// `initial_states` contains an id that does not correspond to a
    /// state of the Kripke structure.
    ///
    /// Note that with an empty `initial_states` set the result is
    /// vacuously `true`, whatever the formula (including `false`).
    pub fn is_ctl_formula_sat<AP: AtomicProposition<DOAP> + PartialEq + Eq + Clone + Hash>(
        &self,
        initial_states: &HashSet<usize>,
        formula: &CTLFormula<AP>,
    ) -> Result<bool, CtlModelCheckingError> {
        let num_states = self.kripke.states().len();
        // report the smallest offending id so that the error is
        // deterministic whatever the iteration order of the set
        if let Some(invalid_id) = initial_states.iter().filter(|id| **id >= num_states).min() {
            return Err(CtlModelCheckingError::OutOfRangeInitialState {
                initial_state_id: *invalid_id,
                num_states,
            });
        }
        let sat_set_bdd = self.get_sat_set_as_bdd(formula);
        let initial_states_bdd = self.bdd_repr.get_states_set_formula(initial_states);
        let implication = initial_states_bdd.imp(&sat_set_bdd);
        Ok(implication.is_true())
    }
}

/// One-shot convenience for [CtlModelChecker::get_sat_set]:
/// builds the BDD representation of `kripke`, answers, and discards it.
pub fn get_sat_set<DOAP, AP: AtomicProposition<DOAP> + PartialEq + Eq + Clone + Hash>(
    kripke: &KripkeStructure<DOAP>,
    formula: &CTLFormula<AP>,
) -> HashSet<usize> {
    CtlModelChecker::new(kripke).get_sat_set(formula)
}

/// One-shot convenience for [CtlModelChecker::is_ctl_formula_sat]:
/// builds the BDD representation of `kripke`, answers, and discards it.
pub fn is_ctl_formula_sat<DOAP, AP: AtomicProposition<DOAP> + PartialEq + Eq + Clone + Hash>(
    kripke: &KripkeStructure<DOAP>,
    initial_states: &HashSet<usize>,
    formula: &CTLFormula<AP>,
) -> Result<bool, CtlModelCheckingError> {
    CtlModelChecker::new(kripke).is_ctl_formula_sat(initial_states, formula)
}

fn initialize_memoizer_at_leaves<
    'a,
    DOAP,
    AP: AtomicProposition<DOAP> + PartialEq + Eq + Clone + Hash,
>(
    kripke: &KripkeStructure<DOAP>,
    mc: &KripkeStructureBddRepresentation,
    leaves: CollectedLeaves<'a, AP>,
) -> (HashMap<&'a CTLFormula<AP>, Rc<Bdd>>, Rc<Bdd>) {
    // ***
    let mut atoms_memoizer = HashMap::new();
    for atom in leaves.atoms {
        atoms_memoizer.insert(atom, mc.var_set.mk_false());
    }
    for (stid, state) in kripke.states().iter().enumerate() {
        let state_bdd = mc.get_state_formula(stid);
        for (atom, bdd) in atoms_memoizer.iter_mut() {
            if let CTLFormula::Leaf(CTLFormulaLeaf::AtomicProp(ap)) = atom
                && ap.is_satisfied_on_state_domain(&state.value_in_domain)
            {
                *bdd = bdd.or(&state_bdd);
            }
        }
    }
    // ***
    let mut sub_formulae_memoizer: HashMap<&'a CTLFormula<AP>, Rc<Bdd>> = atoms_memoizer
        .into_iter()
        .map(|(k, v)| (k, Rc::new(v)))
        .collect();
    let true_bdd = Rc::new(mc.var_set.mk_true());
    if let Some(x) = leaves.true_formula {
        sub_formulae_memoizer.insert(x, true_bdd.clone());
    }
    if let Some(x) = leaves.false_formula {
        sub_formulae_memoizer.insert(x, Rc::new(mc.var_set.mk_false()));
    }
    // ***
    (sub_formulae_memoizer, true_bdd)
}

fn get_ctl_subformula_sat_set_rec<
    'a,
    DOAP,
    AP: AtomicProposition<DOAP> + PartialEq + Eq + Clone + Hash,
>(
    mc: &KripkeStructureBddRepresentation,
    true_bdd: &Rc<Bdd>,
    sub_formulae_memoizer: &mut HashMap<&'a CTLFormula<AP>, Rc<Bdd>>,
    phi: &'a CTLFormula<AP>,
) -> Rc<Bdd> {
    if let Some(got_bdd) = sub_formulae_memoizer.get(phi) {
        return got_bdd.clone();
    }
    let phi_bdd = match phi {
        CTLFormula::Unary(un_op, phi1) => {
            let bdd1 = get_ctl_subformula_sat_set_rec(mc, true_bdd, sub_formulae_memoizer, phi1);
            match un_op {
                UnaryCTLOperator::Not => bdd1.not(),
                UnaryCTLOperator::AX => {
                    mc.get_pre_image_by_transition_relation(PreImageKind::Strong, &bdd1)
                }
                UnaryCTLOperator::EX => {
                    mc.get_pre_image_by_transition_relation(PreImageKind::Weak, &bdd1)
                }
                UnaryCTLOperator::AF => until_fixpoint(true_bdd, bdd1, |x| {
                    mc.get_pre_image_by_transition_relation(PreImageKind::Strong, x)
                }),
                UnaryCTLOperator::EF => until_fixpoint(true_bdd, bdd1, |x| {
                    mc.get_pre_image_by_transition_relation(PreImageKind::Weak, x)
                }),
                UnaryCTLOperator::AG => global_fixpoint(bdd1, |x| {
                    mc.get_pre_image_by_transition_relation(PreImageKind::Strong, x)
                }),
                UnaryCTLOperator::EG => global_fixpoint(bdd1, |x| {
                    mc.get_pre_image_by_transition_relation(PreImageKind::Weak, x)
                }),
            }
        }
        CTLFormula::Binary(bi_op, phi1, phi2) => {
            let bdd1 = get_ctl_subformula_sat_set_rec(mc, true_bdd, sub_formulae_memoizer, phi1);
            let bdd2 = get_ctl_subformula_sat_set_rec(mc, true_bdd, sub_formulae_memoizer, phi2);
            match bi_op {
                BinaryCTLOperator::And => bdd1.and(&bdd2),
                BinaryCTLOperator::Or => bdd1.or(&bdd2),
                BinaryCTLOperator::Imply => bdd1.imp(&bdd2),
                BinaryCTLOperator::Iff => bdd1.iff(&bdd2),
                BinaryCTLOperator::AU => until_fixpoint(&bdd1, bdd2, |x| {
                    mc.get_pre_image_by_transition_relation(PreImageKind::Strong, x)
                }),
                BinaryCTLOperator::EU => until_fixpoint(&bdd1, bdd2, |x| {
                    mc.get_pre_image_by_transition_relation(PreImageKind::Weak, x)
                }),
            }
        }
        CTLFormula::Leaf(_) => {
            panic!("leaf should have been preprocessed")
        }
    };
    sub_formulae_memoizer
        .entry(phi)
        .insert_entry(Rc::new(phi_bdd))
        .get()
        .clone()
}

fn global_fixpoint(bdd: Rc<Bdd>, step_fn: impl Fn(&Bdd) -> Bdd) -> Bdd {
    let mut current = (*bdd).clone();
    loop {
        let next = current.and(&step_fn(&current));
        if next == current {
            break;
        } else {
            current = next;
        }
    }
    current
}

fn until_fixpoint(before: &Rc<Bdd>, after: Rc<Bdd>, step_fn: impl Fn(&Bdd) -> Bdd) -> Bdd {
    let mut current = (*after).clone();
    loop {
        let next = current.or(&before.and(&step_fn(&current)));
        if next == current {
            break;
        } else {
            current = next;
        }
    }
    current
}
