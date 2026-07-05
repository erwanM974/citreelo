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

//! Assertion helpers shared by the test files. All failure messages
//! carry the model name and the formula in concrete syntax, which is
//! essential when a sweep over hundreds of cases fails.

use std::collections::HashSet;

use citreelo::ctl::CTLFormula;
use citreelo::kripke::KripkeStructure;
use citreelo::solve::{CtlModelChecker, get_sat_set};

use crate::common::generators::formula_to_string;
use crate::common::model::{TestAtomicProp, TestDomainOfAp};
use crate::common::oracle::oracle_sat_set;
use crate::common::parser::parse;

type Kripke = KripkeStructure<TestDomainOfAp>;

/// Parses and solves a formula given in concrete syntax.
pub fn solve_str(kripke: &Kripke, formula: &str) -> HashSet<usize> {
    get_sat_set(kripke, &parse(formula))
}

/// Asserts that the satisfaction set of `formula` on `kripke`
/// is exactly `expected`.
pub fn assert_sat_set(model_name: &str, kripke: &Kripke, formula: &str, expected: &[usize]) {
    let got = solve_str(kripke, formula);
    let expected: HashSet<usize> = expected.iter().copied().collect();
    assert_eq!(
        got, expected,
        "sat set mismatch on model '{}' for formula '{}'",
        model_name, formula
    );
}

/// Asserts that two formulae have the same satisfaction set on `kripke`
/// (used for CTL algebraic identities).
pub fn assert_same_sat_set(model_name: &str, kripke: &Kripke, lhs: &str, rhs: &str) {
    let got_lhs = solve_str(kripke, lhs);
    let got_rhs = solve_str(kripke, rhs);
    assert_eq!(
        got_lhs, got_rhs,
        "identity violated on model '{}' : '{}' gave {:?} but '{}' gave {:?}",
        model_name, lhs, got_lhs, rhs, got_rhs
    );
}

/// Asserts that the symbolic checker agrees with the naive
/// explicit-state oracle on `phi`. Takes a [CtlModelChecker] so that
/// sweeping many formulae over one model reuses its BDD representation.
pub fn assert_matches_oracle(
    model_name: &str,
    checker: &CtlModelChecker<TestDomainOfAp>,
    phi: &CTLFormula<TestAtomicProp>,
) {
    let got = checker.get_sat_set(phi);
    let expected = oracle_sat_set(checker.kripke(), phi);
    assert_eq!(
        got,
        expected,
        "symbolic checker disagrees with explicit-state oracle \
         on model '{}' for formula '{}'",
        model_name,
        formula_to_string(phi)
    );
}
