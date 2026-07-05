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

use biodivine_lib_bdd::{Bdd, BddPartialValuation, BddVariable, BddVariableSet};

use crate::kripke::KripkeStructure;

/// We represent a Kripke structure with n states by encoding state ids
/// in binary over k = ⌈log₂(n)⌉ bits, using two sets of BDD variables :
///   - a set C = {c1 , ... , ck} of variables encoding the current state
///   - a set N = {n1 , ... , nk} encoding the next state
///
/// State i is encoded by the valuation of C (resp. N) that reads i in
/// binary, cj (resp. nj) being the j-th least significant bit of i.
/// A state is thus a *strict* conjunction of k literals fixing every bit.
///
/// # Variable ordering
///
/// The variables are interleaved : [c1, n1, c2, n2, ... , ck, nk].
/// This matters for the relational product formula `next_iff_current`
/// (see below) : ∧_{j=1..k} (cj <=> nj) has a BDD of linear size (3k+2
/// nodes) under the interleaved ordering, but ~2^k nodes under the block
/// ordering [c1..ck, n1..nk] — the textbook equality-relation blowup.
///
/// # Current State Set
///
/// We model a set of states of the system as a formula/BDD over C that
/// is a disjunction of the binary encodings of the member states.
///
/// # Unused valuations
///
/// When n is not a power of two, the valuations of C (resp. N) encoding
/// a number in [n, 2^k) do not correspond to any state. The BDDs
/// manipulated by the solver are only meaningful on the valuations that
/// encode actual states and are "don't care" on the unused ones (e.g.,
/// complementing a set of states may add unused valuations). This is
/// sound because :
///   - the transition relation (see below) fixes every bit of both of
///     its endpoints, so it is false whenever either endpoint valuation
///     is unused ;
///   - consequently, neither preimage (see [PreImageKind]) depends on
///     the value of its argument on unused valuations, the weak preimage
///     never produces one, and the strong preimage includes all of them
///     (vacuously : an unused valuation has no outgoing transition),
///     which the next point makes harmless ;
///   - the only places where BDDs are read back as sets of states
///     (extracting a satisfaction set state by state, or checking that a
///     set of initial states entails a satisfaction set) query
///     exclusively the valuations that encode actual states.
///
/// # Transition relation
///
/// Given the notation si -> sj to represent the fact that there exists a
/// transition from state si to state sj in the Kripke structure, the
/// transition relation takes the form of the formula/BDD :
///
/// ∨_{si -> sj} ( enc_C(i) ∧ enc_N(j) )
///
/// where enc_C(i) (resp. enc_N(j)) is the conjunction of the k literals
/// over C (resp. N) encoding i (resp. j) in binary.
///
/// To facilitate symbolic model checking, we precompute and store the
/// transition relation at the creation of the
/// KripkeStructureBddRepresentation object.
///
/// # Relational product to substitute current states set with next states set
///
/// Likewise, to facilitate symbolic model checking, we precompute and
/// store a formula to apply the relational product.
/// This formula is :
///
/// ∧_{j=1..k} (cj <=> nj)
pub(crate) struct KripkeStructureBddRepresentation {
    pub(crate) var_set: BddVariableSet,
    /// current-state bit variables [c1,...,ck], least significant bit first
    current_state_vars: Vec<BddVariable>,
    /// next-state bit variables [n1,...,nk], least significant bit first
    next_state_vars: Vec<BddVariable>,
    /// formula corresponding to the transition relation
    transition_relation: Bdd,
    /// we also memoize the negated version
    negated_transition_relation: Bdd,
    /// formula that is used for the relational product
    next_iff_current: Bdd,
}

/// The number of bits over which the state ids 0..num_states are encoded,
/// i.e., ⌈log₂(num_states)⌉ (0 for the empty and the single-state
/// structures, whose only encoding is the empty conjunction).
fn num_bits_for_state_ids(num_states: usize) -> usize {
    if num_states <= 1 {
        0
    } else {
        (usize::BITS - (num_states - 1).leading_zeros()) as usize
    }
}

/// The partial valuation fixing `bit_vars` to the binary encoding of
/// `state_id` (least significant bit first).
fn state_encoding(bit_vars: &[BddVariable], state_id: usize) -> BddPartialValuation {
    BddPartialValuation::from_values_iter(
        bit_vars
            .iter()
            .enumerate()
            .map(|(bit, var)| (*var, (state_id >> bit) & 1 == 1)),
    )
}

impl KripkeStructureBddRepresentation {
    /// The BDD (a strict conjunction over the current-state bits)
    /// encoding the single state `selected_state_id`.
    pub(crate) fn get_state_formula(&self, selected_state_id: usize) -> Bdd {
        self.var_set
            .mk_conjunctive_clause(&state_encoding(&self.current_state_vars, selected_state_id))
    }

    /// tool function to build the BDD corresponding to an arbitrary set of states
    /// (disjunction of the binary encodings of the selected states)
    pub(crate) fn get_states_set_formula(&self, selected_states_ids: &HashSet<usize>) -> Bdd {
        let mut formula = self.var_set.mk_false();
        for st_id in selected_states_ids {
            let state_bdd = self.get_state_formula(*st_id);
            formula = formula.or(&state_bdd);
        }
        formula
    }

    pub(crate) fn from_kripke_structure<DOAP>(kripke: &KripkeStructure<DOAP>) -> Self {
        let num_states = kripke.states().len();
        let num_bits = num_bits_for_state_ids(num_states);
        // num_bits <= usize::BITS, so 2 * num_bits always fits the
        // 16-bit variable index of the BDD library
        let var_set = BddVariableSet::new_anonymous((num_bits * 2) as u16);
        let all_vars = var_set.variables();
        // interleaved ordering : [c1, n1, c2, n2, ...]
        let current_state_vars: Vec<BddVariable> = all_vars.iter().step_by(2).copied().collect();
        let next_state_vars: Vec<BddVariable> =
            all_vars.iter().skip(1).step_by(2).copied().collect();
        // ***
        // one clause per transition, fixing every bit of both endpoints
        let mut transition_clauses = Vec::new();
        for (origin_st_id, k_state) in kripke.states().iter().enumerate() {
            for target_st_id in &k_state.outgoing_transitions_targets {
                transition_clauses.push(BddPartialValuation::from_values_iter(
                    current_state_vars
                        .iter()
                        .enumerate()
                        .map(|(bit, var)| (*var, (origin_st_id >> bit) & 1 == 1))
                        .chain(
                            next_state_vars
                                .iter()
                                .enumerate()
                                .map(|(bit, var)| (*var, (*target_st_id >> bit) & 1 == 1)),
                        ),
                ));
            }
        }
        let transition_relation = var_set.mk_dnf(&transition_clauses);
        // ***
        let negated_transition_relation = transition_relation.not();
        // ***
        let mut next_iff_current = var_set.mk_true();
        for (current_var, next_var) in current_state_vars.iter().zip(next_state_vars.iter()) {
            next_iff_current =
                next_iff_current.and(&var_set.mk_var(*current_var).iff(&var_set.mk_var(*next_var)));
        }
        // ***
        Self {
            var_set,
            current_state_vars,
            next_state_vars,
            transition_relation,
            negated_transition_relation,
            next_iff_current,
        }
    }

    /// Given a BDD representing a set of states `current_states`,
    /// returns a BDD representing the preimage of that state according to the transition relation
    /// in `self.transition_relation`
    ///
    /// See [PreImageKind] for a definition of preimage.
    pub(crate) fn get_pre_image_by_transition_relation(
        &self,
        kind: PreImageKind,
        current_states: &Bdd,
    ) -> Bdd {
        match kind {
            // EX(S) = ∃s′⋅ T(s,s′) ∧ S(s′)
            PreImageKind::Weak => current_states
                .and(&self.next_iff_current)
                .exists(&self.current_state_vars)
                .and(&self.transition_relation)
                .exists(&self.next_state_vars),
            // AX(S) = ∀s′⋅ ¬T(s,s′) ∨ S(s′)
            PreImageKind::Strong => current_states
                .and(&self.next_iff_current)
                .exists(&self.current_state_vars)
                .or(&self.negated_transition_relation)
                .for_all(&self.next_state_vars),
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
    Strong,
}
