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

//! A catalog of small, named Kripke structures used across the test suite.
//!
//! All structures in this module have a *total* transition relation
//! (every state has at least one successor), matching the assumption of
//! standard CTL semantics. Non-total structures belong in the future
//! edge-case tests, not here.

use citreelo::kripke::{KripkeState, KripkeStructure};

use crate::common::model::{TestAtomicProp::*, TestDomainOfAp, st};

/// All zoo models are valid by construction.
fn build(states: Vec<KripkeState<TestDomainOfAp>>) -> KripkeStructure<TestDomainOfAp> {
    KripkeStructure::new(states).expect("zoo models must be valid")
}

/// The 3-state example from the README:
///
/// ```text
///   s0{P} ---> s1{Q} (self-loop)
///     | ^
///     v |
///   s2{P,Q}
/// ```
/// Transitions: s0 -> {s1, s2}, s1 -> {s1}, s2 -> {s0}.
pub fn readme_ex1() -> KripkeStructure<TestDomainOfAp> {
    build(vec![st(&[P], &[1, 2]), st(&[Q], &[1]), st(&[P, Q], &[0])])
}

/// A line of `len` states ending in a self-loop:
///
/// ```text
///   s0{P} -> s1{P} -> ... -> s_last{Q} (self-loop)
/// ```
/// All states are labelled {P} except the last, labelled {Q}.
pub fn chain(len: usize) -> KripkeStructure<TestDomainOfAp> {
    assert!(len >= 1);
    let states = (0..len)
        .map(|i| {
            if i == len - 1 {
                st(&[Q], &[i])
            } else {
                st(&[P], &[i + 1])
            }
        })
        .collect();
    build(states)
}

/// A ring of `len` states:
///
/// ```text
///   s0{Q} -> s1{P} -> ... -> s_last{P} -> s0
/// ```
/// s0 is labelled {Q}, every other state {P}.
pub fn cycle(len: usize) -> KripkeStructure<TestDomainOfAp> {
    assert!(len >= 1);
    let states = (0..len)
        .map(|i| {
            let atoms = if i == 0 { vec![Q] } else { vec![P] };
            st(&atoms, &[(i + 1) % len])
        })
        .collect();
    build(states)
}

/// A p-cycle with an escape to a q-trap:
///
/// ```text
///   s0{P} <--> s1{P}        (2-cycle on which P holds globally)
///     |
///     v
///   s2{Q} (self-loop)       (escape from s0 into a Q trap)
/// ```
/// Transitions: s0 -> {s1, s2}, s1 -> {s0}, s2 -> {s2}.
///
/// Distinguishes E from A on fixpoints: EG p = {s0, s1} but AG p = {};
/// E(p U q) = {s0, s1, s2} but A(p U q) = {s2}.
pub fn lasso() -> KripkeStructure<TestDomainOfAp> {
    build(vec![st(&[P], &[1, 2]), st(&[P], &[0]), st(&[Q], &[2])])
}

/// A diamond converging on a {P,Q} sink:
///
/// ```text
///        s0{}
///       /    \
///   s1{P}    s2{Q}
///       \    /
///       s3{P,Q} (self-loop)
/// ```
pub fn diamond() -> KripkeStructure<TestDomainOfAp> {
    build(vec![
        st(&[], &[1, 2]),
        st(&[P], &[3]),
        st(&[Q], &[3]),
        st(&[P, Q], &[3]),
    ])
}

/// One root state with `k` successors, each a {P} self-loop:
///
/// ```text
///   s0{} -> s1{P} (loop), s2{P} (loop), ..., sk{P} (loop)
/// ```
/// Fan-out >= 3 stresses the encoding of states with several
/// successors in the transition relation.
pub fn fanout(k: usize) -> KripkeStructure<TestDomainOfAp> {
    assert!(k >= 1);
    let mut states = vec![st(&[], &(1..=k).collect::<Vec<_>>())];
    for i in 1..=k {
        states.push(st(&[P], &[i]));
    }
    build(states)
}

/// A single {P} state looping on itself.
pub fn single_self_loop() -> KripkeStructure<TestDomainOfAp> {
    build(vec![st(&[P], &[0])])
}

/// The complete graph on `n` states (including self-loops),
/// with a diverse deterministic labelling:
/// state i is labelled P iff i is even, Q iff i % 3 == 0, R iff i == n-1.
pub fn total_dense(n: usize) -> KripkeStructure<TestDomainOfAp> {
    assert!(n >= 1);
    let all_targets: Vec<usize> = (0..n).collect();
    let states = (0..n)
        .map(|i| {
            let mut atoms = vec![];
            if i % 2 == 0 {
                atoms.push(P);
            }
            if i % 3 == 0 {
                atoms.push(Q);
            }
            if i == n - 1 {
                atoms.push(R);
            }
            st(&atoms, &all_targets)
        })
        .collect();
    build(states)
}

/// The whole catalog, with names for failure messages.
/// Used by tests that sweep an invariant over every model.
pub fn all_total_models() -> Vec<(&'static str, KripkeStructure<TestDomainOfAp>)> {
    vec![
        ("readme_ex1", readme_ex1()),
        ("chain4", chain(4)),
        ("cycle3", cycle(3)),
        ("cycle4", cycle(4)),
        ("lasso", lasso()),
        ("diamond", diamond()),
        ("fanout2", fanout(2)),
        ("fanout3", fanout(3)),
        ("fanout4", fanout(4)),
        ("single_self_loop", single_self_loop()),
        ("dense4", total_dense(4)),
        ("dense5", total_dense(5)),
        // sizes 7..9 cross the 8-state boundary
        // (3 bits in a binary state encoding, including the exact power of two).
        ("chain8", chain(8)),
        ("cycle9", cycle(9)),
        ("dense8", total_dense(8)),
        ("dense9", total_dense(9)),
        // sizes 15..17 cross the 16-state boundary (4 bits in the binary
        // state encoding) : 15 leaves one unused bit pattern, 16 fills the
        // 4 bits exactly, 17 needs a 5th bit and leaves 15 unused patterns.
        ("chain15", chain(15)),
        ("cycle16", cycle(16)),
        ("chain17", chain(17)),
        ("dense15", total_dense(15)),
        ("dense16", total_dense(16)),
        ("dense17", total_dense(17)),
    ]
}
