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

//! Temporal operators stacked on temporal results, hand-computed.
//!
//! The sensitive scenario for the transition-relation encoding: a
//! universal preimage applied to a satisfaction set that was itself
//! produced by a preimage, on a model where some state has >= 3
//! successors.

use citreelo::kripke::KripkeStructure;

mod common;

use common::asserts::assert_sat_set;
use common::model::{TestAtomicProp::*, st};
use common::zoo::{chain, diamond, fanout, lasso, readme_ex1};

#[test]
fn ax_over_ex_on_fanout2_control() {
    // control case: fan-out 2 only
    // EX(p) = all states, hence AX(EX(p)) = all states
    let k = fanout(2);
    assert_sat_set("fanout2", &k, "EX p", &[0, 1, 2]);
    assert_sat_set("fanout2", &k, "AX EX p", &[0, 1, 2]);
}

#[test]
fn ax_over_ex_on_fanout3() {
    // s0 has 3 successors.
    // EX(p) = all states, hence AX(EX(p)) = all states, including s0.
    let k = fanout(3);
    assert_sat_set("fanout3", &k, "EX p", &[0, 1, 2, 3]);
    assert_sat_set("fanout3", &k, "AX EX p", &[0, 1, 2, 3]);
}

#[test]
fn ax_over_ex_on_fanout4() {
    let k = fanout(4);
    assert_sat_set("fanout4", &k, "EX p", &[0, 1, 2, 3, 4]);
    assert_sat_set("fanout4", &k, "AX EX p", &[0, 1, 2, 3, 4]);
}

#[test]
fn nested_next_on_mixed_fanout3() {
    // s0{} -> {s1,s2,s3} ; s1{P}, s2{P}, s3{Q}, each a self-loop
    let k = KripkeStructure::new(vec![
        st(&[], &[1, 2, 3]),
        st(&[P], &[1]),
        st(&[P], &[2]),
        st(&[Q], &[3]),
    ])
    .unwrap();
    // EX(q) = {0, 3} ; only s3 has all successors in there
    assert_sat_set("mixed_fanout3", &k, "EX q", &[0, 3]);
    assert_sat_set("mixed_fanout3", &k, "AX EX q", &[3]);
    // EX(p) = {0, 1, 2} ; s3 self-loops outside of it
    assert_sat_set("mixed_fanout3", &k, "EX EX p", &[0, 1, 2]);
}

#[test]
fn ax_over_ax_on_chain4() {
    // s0{P} -> s1{P} -> s2{P} -> s3{Q} (self-loop)
    // AX(p) = {0, 1} ; AX(AX(p)) = {0}
    let k = chain(4);
    assert_sat_set("chain4", &k, "AX p", &[0, 1]);
    assert_sat_set("chain4", &k, "AX AX p", &[0]);
}

#[test]
fn ag_over_ef_reset_property() {
    // "from everywhere it remains possible to reach p" on the diamond:
    // EF(p) = all states, so AG(EF(p)) = all states
    let k = diamond();
    assert_sat_set("diamond", &k, "AG EF p", &[0, 1, 2, 3]);

    // on readme_ex1 the property fails everywhere for (p)&(q):
    // s2 is only reachable before entering the s1 self-loop
    let k = readme_ex1();
    assert_sat_set("readme_ex1", &k, "EF (p & q)", &[0, 2]);
    assert_sat_set("readme_ex1", &k, "AG EF (p & q)", &[]);
}

#[test]
fn ef_over_ag() {
    // s0{Q} -> {s1,s2} ; s1{P} self-loop ; s2{Q} self-loop
    // AG(p) = {s1} ; EF(AG(p)) = states that can reach s1 = {s0, s1}
    let k = KripkeStructure::new(vec![st(&[Q], &[1, 2]), st(&[P], &[1]), st(&[Q], &[2])]).unwrap();
    assert_sat_set("trap", &k, "AG p", &[1]);
    assert_sat_set("trap", &k, "EF AG p", &[0, 1]);
}

#[test]
fn until_over_temporal_operands() {
    // on the lasso: E( EX(p) U AG(q) )
    // EX(p) = {s0, s1} ; AG(q) = {s2} ; the until holds everywhere
    let k = lasso();
    assert_sat_set("lasso", &k, "EX p", &[0, 1]);
    assert_sat_set("lasso", &k, "AG q", &[2]);
    assert_sat_set("lasso", &k, "E[EX p U AG q]", &[0, 1, 2]);
}
