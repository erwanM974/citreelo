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

//! CTL algebraic identities, checked on every model of the zoo.
//!
//! These tests need no hand-computed expectations: both sides of each
//! identity must give the same satisfaction set. Since the crate
//! implements every operator directly (no rewriting into a minimal
//! operator basis — see the README), the two sides follow genuinely
//! different code paths, which makes these strong invariants.
//!
//! Substituted base formulae are always wrapped in parentheses in the
//! templates, so the identities hold whatever the precedence of the
//! base formula.

mod common;

use common::asserts::assert_same_sat_set;
use common::zoo::all_total_models;

/// base sub-formulae the identities are instantiated with
const BASES: [&str; 7] = ["p", "q", "r", "p & q", "p | !q", "EX p", "AF q"];

fn check_identity_on_all_models(
    lhs_template: impl Fn(&str) -> String,
    rhs_template: impl Fn(&str) -> String,
) {
    for (name, kripke) in all_total_models() {
        for base in BASES {
            assert_same_sat_set(name, &kripke, &lhs_template(base), &rhs_template(base));
        }
    }
}

#[test]
fn duality_ax_is_not_ex_not() {
    check_identity_on_all_models(|b| format!("AX ({})", b), |b| format!("!EX !({})", b));
}

#[test]
fn duality_ex_is_not_ax_not() {
    check_identity_on_all_models(|b| format!("EX ({})", b), |b| format!("!AX !({})", b));
}

#[test]
fn duality_ag_is_not_ef_not() {
    check_identity_on_all_models(|b| format!("AG ({})", b), |b| format!("!EF !({})", b));
}

#[test]
fn duality_eg_is_not_af_not() {
    check_identity_on_all_models(|b| format!("EG ({})", b), |b| format!("!AF !({})", b));
}

#[test]
fn duality_af_is_not_eg_not() {
    check_identity_on_all_models(|b| format!("AF ({})", b), |b| format!("!EG !({})", b));
}

#[test]
fn af_is_until_with_true() {
    check_identity_on_all_models(|b| format!("AF ({})", b), |b| format!("A[true U ({})]", b));
}

#[test]
fn ef_is_until_with_true() {
    check_identity_on_all_models(|b| format!("EF ({})", b), |b| format!("E[true U ({})]", b));
}

#[test]
fn expansion_law_ef() {
    // EF b  ==  b | EX EF b
    check_identity_on_all_models(
        |b| format!("EF ({})", b),
        |b| format!("({}) | EX EF ({})", b, b),
    );
}

#[test]
fn expansion_law_af() {
    // AF b  ==  b | AX AF b
    check_identity_on_all_models(
        |b| format!("AF ({})", b),
        |b| format!("({}) | AX AF ({})", b, b),
    );
}

#[test]
fn expansion_law_eg() {
    // EG b  ==  b & EX EG b
    check_identity_on_all_models(
        |b| format!("EG ({})", b),
        |b| format!("({}) & EX EG ({})", b, b),
    );
}

#[test]
fn expansion_law_ag() {
    // AG b  ==  b & AX AG b
    check_identity_on_all_models(
        |b| format!("AG ({})", b),
        |b| format!("({}) & AX AG ({})", b, b),
    );
}

#[test]
fn expansion_law_eu() {
    // E[a U b]  ==  b | (a & EX E[a U b])
    for (name, kripke) in all_total_models() {
        for a in ["p", "p | q", "EX p"] {
            for b in ["q", "r", "p & q"] {
                let lhs = format!("E[({}) U ({})]", a, b);
                let rhs = format!("({}) | (({}) & EX E[({}) U ({})])", b, a, a, b);
                assert_same_sat_set(name, &kripke, &lhs, &rhs);
            }
        }
    }
}

#[test]
fn expansion_law_au() {
    // A[a U b]  ==  b | (a & AX A[a U b])
    for (name, kripke) in all_total_models() {
        for a in ["p", "p | q", "EX p"] {
            for b in ["q", "r", "p & q"] {
                let lhs = format!("A[({}) U ({})]", a, b);
                let rhs = format!("({}) | (({}) & AX A[({}) U ({})])", b, a, a, b);
                assert_same_sat_set(name, &kripke, &lhs, &rhs);
            }
        }
    }
}

#[test]
fn duality_au() {
    // A[a U b]  ==  !( E[!b U (!a & !b)] | EG !b )
    for (name, kripke) in all_total_models() {
        for a in ["p", "p | q"] {
            for b in ["q", "p & q"] {
                let lhs = format!("A[({}) U ({})]", a, b);
                let rhs = format!("!(E[!({}) U (!({}) & !({}))] | EG !({}))", b, a, b, b);
                assert_same_sat_set(name, &kripke, &lhs, &rhs);
            }
        }
    }
}

#[test]
fn boolean_tautologies() {
    // sanity: material implication and iff expressed with & | !
    check_identity_on_all_models(|b| format!("p => ({})", b), |b| format!("!p | ({})", b));
    check_identity_on_all_models(
        |b| format!("p <=> ({})", b),
        |b| format!("(p & ({})) | (!p & !({}))", b, b),
    );
}
