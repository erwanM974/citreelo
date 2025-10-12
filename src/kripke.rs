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





/// A Kripke structure is a transition system in which transitions have no label
/// and states are labelled over a Domain Of Atomic Proposition (DOAP).
/// 
/// An Atomic Propositions (AP) may or may not hold on a given state depending on the value 
/// in the DOAP with which it is labelled.
/// 
/// We use a basic adjacency list representation.
pub struct KripkeStructure<DOAP> {
    pub states : Vec<KripkeState<DOAP>>
}

impl<DOAP> KripkeStructure<DOAP> {
    pub fn new(states: Vec<KripkeState<DOAP>>) -> Self {
        Self { states }
    }
}

/// A state of a [KripkeStructure] is characterize by:
/// - a value in the domain in which Atomic Proposition are evaluated
/// - and its possible next states (following an adjacency list representation of the [KripkeStructure])
pub struct KripkeState<DOAP> {
    pub value_in_domain : DOAP,
    pub outgoing_transitions_targets : Vec<usize>
}

impl<DOAP> KripkeState<DOAP> {
    pub fn new(value_in_domain: DOAP, outgoing_transitions_targets: Vec<usize>) -> Self {
        Self { value_in_domain, outgoing_transitions_targets }
    }
}



pub trait AtomicProposition<DOAP> {
    fn is_satisfied_on_state_domain(&self, state_domain : &DOAP) -> bool;
}

