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

//! The fixpoint operators EF / AF / EG / AG / EU / AU, hand-computed
//! on models chosen so that each fixpoint needs several iterations
//! to converge (chains, cycles, lassos) and so that the E and A
//! variants give different answers.

use citreelo::kripke::KripkeStructure;

mod common;

use common::asserts::assert_sat_set;
use common::model::{TestAtomicProp::*, st};
use common::zoo::{chain, cycle, diamond, lasso};

#[test]
fn fixpoints_on_chain4() {
    // s0{P} -> s1{P} -> s2{P} -> s3{Q} (self-loop)
    // AF/EF need len-1 iterations to propagate back from s3.
    let k = chain(4);
    assert_sat_set("chain4", &k, "AF q", &[0, 1, 2, 3]);
    assert_sat_set("chain4", &k, "EF q", &[0, 1, 2, 3]);
    assert_sat_set("chain4", &k, "AF p", &[0, 1, 2]);
    assert_sat_set("chain4", &k, "EF p", &[0, 1, 2]);
    assert_sat_set("chain4", &k, "AG p", &[]);
    assert_sat_set("chain4", &k, "EG p", &[]);
    assert_sat_set("chain4", &k, "AG q", &[3]);
    assert_sat_set("chain4", &k, "EG q", &[3]);
    assert_sat_set("chain4", &k, "A[p U q]", &[0, 1, 2, 3]);
    assert_sat_set("chain4", &k, "E[p U q]", &[0, 1, 2, 3]);
    assert_sat_set("chain4", &k, "AG (p | q)", &[0, 1, 2, 3]);
    assert_sat_set("chain4", &k, "EF (p & q)", &[]);
}

#[test]
fn fixpoints_on_cycle3() {
    // s0{Q} -> s1{P} -> s2{P} -> s0 : a single cyclic path
    let k = cycle(3);
    assert_sat_set("cycle3", &k, "AF q", &[0, 1, 2]);
    assert_sat_set("cycle3", &k, "EF q", &[0, 1, 2]);
    assert_sat_set("cycle3", &k, "AF p", &[0, 1, 2]);
    // the unique path keeps coming back to s0 where p does not hold
    assert_sat_set("cycle3", &k, "EG p", &[]);
    assert_sat_set("cycle3", &k, "AG p", &[]);
    assert_sat_set("cycle3", &k, "AG (p | q)", &[0, 1, 2]);
    assert_sat_set("cycle3", &k, "E[p U q]", &[0, 1, 2]);
    assert_sat_set("cycle3", &k, "A[p U q]", &[0, 1, 2]);
    assert_sat_set("cycle3", &k, "EF (p & q)", &[]);
}

#[test]
fn fixpoints_on_lasso() {
    // s0{P} <-> s1{P} (P-cycle), s0 -> s2{Q} (Q-trap)
    // E and A differ everywhere here: staying on the cycle
    // forever avoids q, while escaping reaches it.
    let k = lasso();
    assert_sat_set("lasso", &k, "EG p", &[0, 1]);
    assert_sat_set("lasso", &k, "AG p", &[]);
    assert_sat_set("lasso", &k, "EF q", &[0, 1, 2]);
    assert_sat_set("lasso", &k, "AF q", &[2]);
    assert_sat_set("lasso", &k, "E[p U q]", &[0, 1, 2]);
    assert_sat_set("lasso", &k, "A[p U q]", &[2]);
    assert_sat_set("lasso", &k, "EG q", &[2]);
    assert_sat_set("lasso", &k, "AG (p | q)", &[0, 1, 2]);
}

#[test]
fn fixpoints_on_diamond() {
    // s0{} -> {s1{P}, s2{Q}} -> s3{P,Q} (self-loop)
    let k = diamond();
    assert_sat_set("diamond", &k, "EF (p & q)", &[0, 1, 2, 3]);
    assert_sat_set("diamond", &k, "AF (p & q)", &[0, 1, 2, 3]);
    assert_sat_set("diamond", &k, "AG (p | q)", &[1, 2, 3]);
    assert_sat_set("diamond", &k, "EG p", &[1, 3]);
    assert_sat_set("diamond", &k, "EG q", &[2, 3]);
    assert_sat_set("diamond", &k, "A[q U (p & q)]", &[2, 3]);
    assert_sat_set("diamond", &k, "E[q U (p & q)]", &[2, 3]);
}

#[test]
fn until_distinguishes_universal_from_existential() {
    // s0{P} -> {s1, s2} ; s1{P} self-loop (p forever, never q) ;
    // s2{Q} self-loop.
    // From s0, *some* path reaches q (via s2) but the s1 branch
    // never does: E(p U q) holds at s0, A(p U q) does not.
    let k = KripkeStructure::new(vec![st(&[P], &[1, 2]), st(&[P], &[1]), st(&[Q], &[2])]).unwrap();
    assert_sat_set("escape", &k, "E[p U q]", &[0, 2]);
    assert_sat_set("escape", &k, "A[p U q]", &[2]);
    assert_sat_set("escape", &k, "EF q", &[0, 2]);
    assert_sat_set("escape", &k, "AF q", &[2]);
    assert_sat_set("escape", &k, "EG p", &[0, 1]);
    assert_sat_set("escape", &k, "AG p", &[1]);
}
