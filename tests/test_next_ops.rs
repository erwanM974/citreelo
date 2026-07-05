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

//! EX / AX in isolation, hand-computed, systematically across
//! fan-outs 1 to 4, self-loops, and duplicated transitions.
//! Fan-out >= 3 exercises the encoding of states with several
//! successors in the transition relation, at depth 1.

use citreelo::kripke::KripkeStructure;

mod common;

use common::asserts::assert_sat_set;
use common::model::{TestAtomicProp::*, st};
use common::zoo::{diamond, fanout, single_self_loop};

#[test]
fn next_on_single_self_loop() {
    let k = single_self_loop();
    assert_sat_set("single_self_loop", &k, "EX p", &[0]);
    assert_sat_set("single_self_loop", &k, "AX p", &[0]);
    assert_sat_set("single_self_loop", &k, "EX q", &[]);
    assert_sat_set("single_self_loop", &k, "AX q", &[]);
}

#[test]
fn next_on_diamond() {
    // s0{} -> {s1,s2} ; s1{P} -> s3 ; s2{Q} -> s3 ; s3{P,Q} -> s3
    let k = diamond();
    assert_sat_set("diamond", &k, "EX p", &[0, 1, 2, 3]);
    assert_sat_set("diamond", &k, "AX p", &[1, 2, 3]);
    assert_sat_set("diamond", &k, "EX q", &[0, 1, 2, 3]);
    assert_sat_set("diamond", &k, "AX q", &[1, 2, 3]);
    assert_sat_set("diamond", &k, "EX (p & q)", &[1, 2, 3]);
    assert_sat_set("diamond", &k, "AX (p | q)", &[0, 1, 2, 3]);
    assert_sat_set("diamond", &k, "EX r", &[]);
}

#[test]
fn next_on_uniform_fanouts() {
    // fanout(k): s0{} -> {s1..sk}, each si{P} a self-loop.
    // The expected sets are the same for every k; with k >= 3 this
    // exercises the multi-target transition encoding.
    for k_size in 1..=4 {
        let k = fanout(k_size);
        let name = format!("fanout{}", k_size);
        let all: Vec<usize> = (0..=k_size).collect();
        assert_sat_set(&name, &k, "EX p", &all);
        assert_sat_set(&name, &k, "AX p", &all);
        assert_sat_set(&name, &k, "EX q", &[]);
        assert_sat_set(&name, &k, "AX q", &[]);
    }
}

#[test]
fn next_on_mixed_fanout3() {
    // s0{} -> {s1,s2,s3} ; s1{P}, s2{P}, s3{Q}, each a self-loop.
    // Unlike the uniform fanout models, AX and EX differ at s0 here.
    let k = KripkeStructure::new(vec![
        st(&[], &[1, 2, 3]),
        st(&[P], &[1]),
        st(&[P], &[2]),
        st(&[Q], &[3]),
    ])
    .unwrap();
    assert_sat_set("mixed_fanout3", &k, "EX p", &[0, 1, 2]);
    assert_sat_set("mixed_fanout3", &k, "AX p", &[1, 2]);
    assert_sat_set("mixed_fanout3", &k, "EX q", &[0, 3]);
    assert_sat_set("mixed_fanout3", &k, "AX q", &[3]);
    assert_sat_set("mixed_fanout3", &k, "AX (p | q)", &[0, 1, 2, 3]);
}

#[test]
fn next_with_duplicate_transitions() {
    // The transition s0 -> s1 is declared twice: the semantics must be
    // identical to declaring it once.
    let k = KripkeStructure::new(vec![st(&[P], &[1, 1]), st(&[Q], &[1])]).unwrap();
    assert_sat_set("dup_transition", &k, "EX q", &[0, 1]);
    assert_sat_set("dup_transition", &k, "AX q", &[0, 1]);
    assert_sat_set("dup_transition", &k, "EX p", &[]);
    assert_sat_set("dup_transition", &k, "AX p", &[]);
}

#[test]
fn next_with_duplicate_transitions_among_several() {
    // duplicates mixed with other targets: s0 -> {s1, s1, s2}
    let k =
        KripkeStructure::new(vec![st(&[], &[1, 1, 2]), st(&[P], &[1]), st(&[Q], &[2])]).unwrap();
    assert_sat_set("dup_among_several", &k, "EX p", &[0, 1]);
    assert_sat_set("dup_among_several", &k, "EX q", &[0, 2]);
    assert_sat_set("dup_among_several", &k, "AX (p | q)", &[0, 1, 2]);
    assert_sat_set("dup_among_several", &k, "AX p", &[1]);
}
