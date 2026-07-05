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

use std::fmt;

/// A Kripke structure is a transition system in which transitions have no label
/// and states are labelled over a Domain Of Atomic Proposition (DOAP).
///
/// An Atomic Propositions (AP) may or may not hold on a given state depending on the value
/// in the DOAP with which it is labelled.
///
/// We use a basic adjacency list representation.
///
/// The states are private and the structure is validated at construction
/// (see [KripkeStructure::new]) so that the model checker only ever
/// operates on well-formed structures: every transition target is in
/// range and the transition relation is total (no deadlock state),
/// as required by CTL semantics.
pub struct KripkeStructure<DOAP> {
    states: Vec<KripkeState<DOAP>>,
}

/// The reasons for which [KripkeStructure::new] may reject its input.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum KripkeStructureBuildError {
    /// A state has no outgoing transition.
    /// CTL semantics is defined over total transition relations:
    /// on a deadlock state, universal operators (AX, AF, AU, ...)
    /// would be vacuously satisfied.
    DeadlockState { state_id: usize },
    /// A transition points to a state that does not exist.
    OutOfRangeTransitionTarget {
        origin_state_id: usize,
        target_state_id: usize,
        num_states: usize,
    },
}

impl fmt::Display for KripkeStructureBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KripkeStructureBuildError::DeadlockState { state_id } => {
                write!(
                    f,
                    "state {} is a deadlock (it has no outgoing transition) : \
                     CTL model checking requires a total transition relation",
                    state_id
                )
            }
            KripkeStructureBuildError::OutOfRangeTransitionTarget {
                origin_state_id,
                target_state_id,
                num_states,
            } => {
                write!(
                    f,
                    "transition from state {} targets state {} \
                     but there are only {} states",
                    origin_state_id, target_state_id, num_states
                )
            }
        }
    }
}

impl std::error::Error for KripkeStructureBuildError {}

impl<DOAP> KripkeStructure<DOAP> {
    /// Validates and builds a Kripke structure.
    ///
    /// Fails if a transition targets a state that does not exist
    /// ([KripkeStructureBuildError::OutOfRangeTransitionTarget])
    /// or if a state has no outgoing transition
    /// ([KripkeStructureBuildError::DeadlockState]) — a system in
    /// which a state may stay forever must model this with an
    /// explicit self-loop.
    pub fn new(states: Vec<KripkeState<DOAP>>) -> Result<Self, KripkeStructureBuildError> {
        let num_states = states.len();
        for (state_id, state) in states.iter().enumerate() {
            if state.outgoing_transitions_targets.is_empty() {
                return Err(KripkeStructureBuildError::DeadlockState { state_id });
            }
            for target in &state.outgoing_transitions_targets {
                if *target >= num_states {
                    return Err(KripkeStructureBuildError::OutOfRangeTransitionTarget {
                        origin_state_id: state_id,
                        target_state_id: *target,
                        num_states,
                    });
                }
            }
        }
        Ok(Self { states })
    }

    /// The states of the structure. A state's id is its position in
    /// this slice; transition targets and initial states are given as
    /// such ids.
    pub fn states(&self) -> &[KripkeState<DOAP>] {
        &self.states
    }
}

/// A state of a [KripkeStructure] is characterized by:
/// - a value in the domain in which Atomic Proposition are evaluated
/// - and its possible next states (following an adjacency list representation of the [KripkeStructure])
pub struct KripkeState<DOAP> {
    pub value_in_domain: DOAP,
    pub outgoing_transitions_targets: Vec<usize>,
}

impl<DOAP> KripkeState<DOAP> {
    /// Builds a state labelled with `value_in_domain` whose successors
    /// are the states with ids `outgoing_transitions_targets`.
    /// Validation happens when assembling the states into a
    /// [KripkeStructure] (see [KripkeStructure::new]).
    pub fn new(value_in_domain: DOAP, outgoing_transitions_targets: Vec<usize>) -> Self {
        Self {
            value_in_domain,
            outgoing_transitions_targets,
        }
    }
}

/// An atomic proposition that can be evaluated on the domain `DOAP`
/// with which the states of a [KripkeStructure] are labelled.
pub trait AtomicProposition<DOAP> {
    /// Whether this atomic proposition holds on a state labelled with
    /// `state_domain`.
    fn is_satisfied_on_state_domain(&self, state_domain: &DOAP) -> bool;
}
