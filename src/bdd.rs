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


use std::collections::HashSet;

use biodivine_lib_bdd::{Bdd, BddVariable, BddVariableSet};

use crate::kripke::KripkeStructure;





/// We represent a Kripke structure using two sets of variables :
///   - a set S = {s1 , ... , sn } of variables representing the current state
///   - a set S'= {s1', ... , sn'} representing the next state 
/// 
/// These variables are stored in a vector [s1,...,sn,s1',...,sn']
/// 
/// # Current State Set 
/// 
/// We model a current set of states of the system as a formula/BDD that is a disjunction of variables from S.
/// i.e., given S = {s1 , ... , sn }, if one is either in si or in sj, this corresponds to the formula
/// 
/// ϕ = si ∨ sj 
/// 
/// Here we have a loose specification i.e. it does not say anything about any sk that is neither si nor sj 
/// 
/// # Transition relation
/// 
/// Given the notation si -> sj' to represent the fact that there exist a transition 
/// from state si to state sj in the Kripke structure,
/// the transition relation takes the form of the formula/BDD:
/// 
/// ∨_{si -> sj'} ( (¬s1 ∧ ... si ∧ ... ¬sn) ∧ (¬s1' ∧ ... sj' ∧ ... ¬sn') )
/// 
/// Note that here, unlike above, we have a strict specification i.e., 
/// we specifically declare ¬vk for all the vk that are not involved in the transition
/// 
/// To facilitate symbolic model checking, we precompute and store the transition relation
/// at the creation of the KripkeStructureBddRepresentation object.
/// 
/// # Relational product to substitute current states set with next states set 
/// 
/// Likewise, to faciliate symbolic model checking, we precompute and store a formula to
/// apply the relational product.
/// This formula is:
/// 
/// ∧_{i=1..n} (si <=> si')
pub(crate) struct KripkeStructureBddRepresentation {
    num_states : usize,
    pub var_set : BddVariableSet,
    /// vector [s1,...,sn,s1',...,sn']
    pub raw_vars : Vec<BddVariable>,
    /// formula corresponding to the transition relation
    transition_relation : Bdd,
    /// we also memoize the negated version
    negated_transition_relation : Bdd,
    /// formula that is used for the relational product
    next_iff_current : Bdd
}



/// tool function to build the BDD corresponding to the transition
/// relation of a Kripke Structure 
fn get_strict_state_formula_for_transition_relation(
        is_next : bool, 
        num_states : usize, 
        var_set : &BddVariableSet, 
        raw_vars : &[BddVariable], 
        selected_state_id : usize
    ) -> Bdd {
        let mut formula = var_set.mk_true();
        for st_id in 0..num_states {
            let var = if is_next {
                raw_vars.get(num_states + st_id).unwrap()
            } else {
                raw_vars.get(st_id).unwrap()
            };
            let state_bdd = if st_id == selected_state_id {
                var_set.mk_var(*var)
            } else {
                var_set.mk_var(*var).not()
            };
            formula = formula.and(&state_bdd);
        }
        formula
    }

impl KripkeStructureBddRepresentation {

    pub(crate) fn get_state_formula(&self, selected_state_id : usize) -> Bdd {
        let mut formula = self.var_set.mk_true();
        for st_id in 0..self.num_states {
            let var = self.raw_vars.get(st_id).unwrap();
            let state_bdd = if st_id == selected_state_id {
                self.var_set.mk_var(*var)
            } else {
                self.var_set.mk_var(*var).not()
            };
            formula = formula.and(&state_bdd);
        }
        formula
    }

    /// tool function to build the BDD corresponding to the initial state
    pub(crate) fn get_states_set_formula(
        &self,
        selected_states_ids : &HashSet<usize> 
    ) -> Bdd {
        let mut formula = self.var_set.mk_false();
        for st_id in selected_states_ids {
            let state_bdd = self.get_state_formula(*st_id);
            formula = formula.or(  
                &state_bdd
            );
        }
        formula
    }

    pub fn from_kripke_structure<DOAP>(kripke : &KripkeStructure<DOAP>) -> Self {
        let num_states = kripke.states.len();
        let var_set = BddVariableSet::new_anonymous((num_states*2) as u16);
        let raw_vars = var_set.variables();
        // ***
        let mut transition_relation = var_set.mk_false();
        for (origin_st_id, k_state) in kripke.states.iter().enumerate() {
            let origin_st_current_formula = get_strict_state_formula_for_transition_relation(
                false,
                num_states,
                &var_set,
                &raw_vars,
                origin_st_id
            );
            for target_st_id in &k_state.outgoing_transitions_targets {
                let target_st_next_formula = get_strict_state_formula_for_transition_relation(
                    true,
                    num_states,
                    &var_set,
                    &raw_vars,
                    *target_st_id
                );
                let transition_bdd = origin_st_current_formula.and(&target_st_next_formula);
                transition_relation = transition_relation.or(&transition_bdd);
            }
        }
        // ***
        let negated_transition_relation = transition_relation.not();
        // ***
        let mut next_iff_current = var_set.mk_true();
        for st_id in 0..num_states {
            let st_current_var = raw_vars.get(st_id).unwrap();
            let st_next_var = raw_vars.get(st_id + num_states).unwrap();
            next_iff_current = next_iff_current.and(
                &var_set.mk_var(*st_current_var).iff(&var_set.mk_var(*st_next_var))
            );
        }
        // ***
        Self {
            num_states,var_set,raw_vars,transition_relation,negated_transition_relation,next_iff_current
        }
    }

    /// Given a BDD representing a set of states `current_states`,
    /// returns a BDD representing the preimage of that state according to the transition relation
    /// in `self.transition_relation`
    /// 
    /// See [PreImageKind] for a definition of preimage.
    pub fn get_pre_image_by_transition_relation(&self, kind : PreImageKind, current_states : &Bdd) -> Bdd {
        match kind {
            // EX(S) = ∀s′⋅ T(s,s′) ∧ S(s′)
            PreImageKind::Weak => {
                current_states
                    .and(&self.next_iff_current)
                    .exists(&self.raw_vars[0..self.num_states])
                    .and(&self.transition_relation)
                    .exists(&self.raw_vars[self.num_states..])
            },
            // AX(S) = ∀s′⋅ ¬T(s,s′) ∨ S(s′)
            PreImageKind::Strong => {
                current_states
                    .and(&self.next_iff_current)
                    .exists(&self.raw_vars[0..self.num_states])
                    .or(&self.negated_transition_relation)
                    .for_all(&self.raw_vars[self.num_states..])
            }
        }
    }
}


/// In a transition system defined by a set of states `S` and a transition relation `⇾` we define the
/// notions of weak and strong preimage as follows.
/// 
/// # Weak preimage
/// 
/// The Weak preimage of a subset of states `X ⊂ S` as the set of states :
/// 
/// `Wpi(X) = {s ∈ S | ∃ s' ∈ X, s ⇾ s'}`
/// 
/// # Strong preimage
/// 
/// The Strong preimage of a subset of states `X ⊂ S` as the set of states :
/// 
/// `Spi(X) = {s ∈ S | ∀ s' ∈ S, (s ⇾ s') ⇒ (s' ∈ X)}`
/// 
/// # References
/// 
/// The weak/strong preimage terminology can be found in <https://doi.org/10.1016/S0004-3702(02)00374-0>
pub(crate) enum PreImageKind {
    Weak,
    Strong
}


