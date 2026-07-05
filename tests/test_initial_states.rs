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

//! Tests of `is_ctl_formula_sat`, the initial-states entry point.
//!
//! Its defining property: a formula is satisfied from a set of initial
//! states iff every initial state belongs to the satisfaction set
//! computed by `get_sat_set`. We check that consistency over the zoo
//! models: exhaustively over all subsets of states on small models,
//! and on a deterministic sample of subsets on larger ones (the
//! exhaustive check is exponential in the number of states).
//!
//! Out-of-range initial-state ids are rejected with an error.

use std::collections::HashSet;

use citreelo::solve::{CtlModelChecker, CtlModelCheckingError, get_sat_set, is_ctl_formula_sat};
use map_macro::hash_set;

mod common;

use common::generators::Lcg;
use common::parser::parse;
use common::zoo::{all_total_models, readme_ex1};

const FORMULAS: [&str; 8] = [
    "p",
    "!q",
    "AX q",
    "EX p",
    "AF q",
    "EG p",
    "E[p U q]",
    "AG (p | q)",
];

/// Largest model size on which every non-empty subset of states is
/// tried as the initial-states set; above it, subsets are sampled.
const MAX_STATES_FOR_EXHAUSTIVE_SUBSETS: usize = 6;
const SAMPLED_SUBSETS_PER_FORMULA: usize = 40;

/// The non-empty initial-states subsets to test on a model of `n`
/// states: all of them when feasible, otherwise every singleton, the
/// full set, and a deterministic sample in between.
fn initial_state_subsets(n: usize, rng: &mut Lcg) -> Vec<u32> {
    if n <= MAX_STATES_FOR_EXHAUSTIVE_SUBSETS {
        (1u32..(1 << n)).collect()
    } else {
        let mut masks: Vec<u32> = (0..n).map(|i| 1 << i).collect();
        masks.push((1u32 << n) - 1);
        masks.extend((0..SAMPLED_SUBSETS_PER_FORMULA).map(|_| 1 + rng.below((1 << n) - 1) as u32));
        masks
    }
}

#[test]
fn consistent_with_sat_set_on_subsets_of_initial_states() {
    for (name, kripke) in all_total_models() {
        let checker = CtlModelChecker::new(&kripke);
        let n = kripke.states().len();
        let mut rng = Lcg::new(0x1217 + n as u64);
        for formula_str in FORMULAS {
            let phi = parse(formula_str);
            let sat_set = checker.get_sat_set(&phi);
            for mask in initial_state_subsets(n, &mut rng) {
                let initial: HashSet<usize> = (0..n).filter(|i| mask & (1 << i) != 0).collect();
                let expected = initial.is_subset(&sat_set);
                let got = checker.is_ctl_formula_sat(&initial, &phi).unwrap();
                assert_eq!(
                    got, expected,
                    "is_ctl_formula_sat inconsistent with get_sat_set \
                     on model '{}' for formula '{}' with initial states {:?} \
                     (sat set: {:?})",
                    name, formula_str, initial, sat_set
                );
            }
        }
    }
}

#[test]
fn hand_computed_examples_on_readme_ex1() {
    // the original example: properties of s0 as single initial state
    let kripke = readme_ex1();
    let cases = [
        ("AG p", false),      // s0 -> s1 where p does not hold
        ("AF q", true),       // all paths from s0 eventually reach q
        ("EF (p & q)", true), // s0 -> s2 where p&q holds
        ("AX q", true),       // both successors of s0 satisfy q
        ("EX p", true),       // s2 is a successor of s0 and satisfies p
        ("AG (p | q)", true), // p|q holds on every state
        ("A[q U p]", true),   // p holds immediately on s0
        ("A[p U q]", true),
    ];
    let initial = hash_set! {0};
    for (formula_str, expected) in cases {
        let phi = parse(formula_str);
        assert_eq!(
            is_ctl_formula_sat(&kripke, &initial, &phi).unwrap(),
            expected,
            "unexpected verdict for '{}' from initial state 0",
            formula_str
        );
    }
}

#[test]
fn multiple_initial_states_on_readme_ex1() {
    // q = {1,2} : satisfied from {1}, {2}, {1,2} but not from any
    // set containing 0
    let kripke = readme_ex1();
    let phi = parse("q");
    assert!(is_ctl_formula_sat(&kripke, &hash_set! {1}, &phi).unwrap());
    assert!(is_ctl_formula_sat(&kripke, &hash_set! {2}, &phi).unwrap());
    assert!(is_ctl_formula_sat(&kripke, &hash_set! {1, 2}, &phi).unwrap());
    assert!(!is_ctl_formula_sat(&kripke, &hash_set! {0}, &phi).unwrap());
    assert!(!is_ctl_formula_sat(&kripke, &hash_set! {0, 1, 2}, &phi).unwrap());
}

#[test]
fn rejects_out_of_range_initial_states() {
    // readme_ex1 has 3 states: ids >= 3 must be rejected,
    // reporting the smallest offending id
    let kripke = readme_ex1();
    let phi = parse("p");
    assert_eq!(
        is_ctl_formula_sat(&kripke, &hash_set! {7}, &phi),
        Err(CtlModelCheckingError::OutOfRangeInitialState {
            initial_state_id: 7,
            num_states: 3
        })
    );
    assert_eq!(
        is_ctl_formula_sat(&kripke, &hash_set! {0, 9, 5}, &phi),
        Err(CtlModelCheckingError::OutOfRangeInitialState {
            initial_state_id: 5,
            num_states: 3
        })
    );
    // boundary: the largest valid id is num_states - 1
    assert!(is_ctl_formula_sat(&kripke, &hash_set! {2}, &phi).is_ok());
    assert!(is_ctl_formula_sat(&kripke, &hash_set! {3}, &phi).is_err());

    let msg = is_ctl_formula_sat(&kripke, &hash_set! {7}, &phi)
        .unwrap_err()
        .to_string();
    assert!(
        msg.contains('7') && msg.contains('3'),
        "unhelpful message: {}",
        msg
    );
}

#[test]
fn reusable_checker_matches_one_shot_functions() {
    // a CtlModelChecker answering many queries on one precomputed BDD
    // representation must agree with the one-shot functions that
    // rebuild it per call
    let kripke = readme_ex1();
    let checker = CtlModelChecker::new(&kripke);
    for formula_str in FORMULAS {
        let phi = parse(formula_str);
        assert_eq!(
            checker.get_sat_set(&phi),
            get_sat_set(&kripke, &phi),
            "checker vs one-shot mismatch for '{}'",
            formula_str
        );
        for initial in [hash_set! {0}, hash_set! {1, 2}, hash_set! {0, 1, 2}] {
            assert_eq!(
                checker.is_ctl_formula_sat(&initial, &phi),
                is_ctl_formula_sat(&kripke, &initial, &phi),
                "checker vs one-shot mismatch for '{}' from {:?}",
                formula_str,
                initial
            );
        }
    }
    // the error path behaves identically too
    assert_eq!(
        checker.is_ctl_formula_sat(&hash_set! {9}, &parse("p")),
        is_ctl_formula_sat(&kripke, &hash_set! {9}, &parse("p"))
    );
}

#[test]
fn empty_initial_set_is_vacuously_satisfied() {
    // Pins the documented convention: with no initial state
    // the implication `initial => sat_set` is vacuously true, even for
    // the formula `false`. Revisit if a stricter contract is chosen.
    let kripke = readme_ex1();
    let no_states: HashSet<usize> = HashSet::new();
    assert!(is_ctl_formula_sat(&kripke, &no_states, &parse("false")).unwrap());
    assert!(is_ctl_formula_sat(&kripke, &no_states, &parse("p")).unwrap());
}
