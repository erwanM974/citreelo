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

//! Satisfaction sets for leaves (true / false / atoms) and the pure
//! boolean connectives, with hand-computed expectations.
//! No temporal operators here.

use citreelo::ctl::{CTLFormula, CTLFormulaLeaf};

mod common;

use common::asserts::assert_sat_set;
use common::model::TestAtomicProp;
use common::parser::parse;
use common::zoo::{diamond, readme_ex1};

#[test]
fn leaves_on_readme_ex1() {
    // s0{P}, s1{Q}, s2{P,Q}
    let k = readme_ex1();
    assert_sat_set("readme_ex1", &k, "true", &[0, 1, 2]);
    assert_sat_set("readme_ex1", &k, "false", &[]);
    assert_sat_set("readme_ex1", &k, "p", &[0, 2]);
    assert_sat_set("readme_ex1", &k, "q", &[1, 2]);
    assert_sat_set("readme_ex1", &k, "r", &[]);
}

#[test]
fn negation_on_readme_ex1() {
    let k = readme_ex1();
    assert_sat_set("readme_ex1", &k, "!p", &[1]);
    assert_sat_set("readme_ex1", &k, "!q", &[0]);
    assert_sat_set("readme_ex1", &k, "!r", &[0, 1, 2]);
    assert_sat_set("readme_ex1", &k, "!true", &[]);
    assert_sat_set("readme_ex1", &k, "!false", &[0, 1, 2]);
    assert_sat_set("readme_ex1", &k, "!!p", &[0, 2]);
}

#[test]
fn conjunction_disjunction_on_readme_ex1() {
    let k = readme_ex1();
    assert_sat_set("readme_ex1", &k, "p & q", &[2]);
    assert_sat_set("readme_ex1", &k, "p | q", &[0, 1, 2]);
    assert_sat_set("readme_ex1", &k, "!p & !q", &[]);
    assert_sat_set("readme_ex1", &k, "!p | !q", &[0, 1]);
    assert_sat_set("readme_ex1", &k, "p & true", &[0, 2]);
    assert_sat_set("readme_ex1", &k, "p & false", &[]);
    assert_sat_set("readme_ex1", &k, "p | false", &[0, 2]);
    assert_sat_set("readme_ex1", &k, "p | true", &[0, 1, 2]);
}

#[test]
fn implication_on_readme_ex1() {
    // p = {0,2}, q = {1,2}
    let k = readme_ex1();
    assert_sat_set("readme_ex1", &k, "p => q", &[1, 2]);
    assert_sat_set("readme_ex1", &k, "q => p", &[0, 2]);
    assert_sat_set("readme_ex1", &k, "false => p", &[0, 1, 2]);
    assert_sat_set("readme_ex1", &k, "p => true", &[0, 1, 2]);
    assert_sat_set("readme_ex1", &k, "true => p", &[0, 2]);
}

#[test]
fn iff_on_readme_ex1() {
    let k = readme_ex1();
    assert_sat_set("readme_ex1", &k, "p <=> q", &[2]);
    assert_sat_set("readme_ex1", &k, "p <=> p", &[0, 1, 2]);
    assert_sat_set("readme_ex1", &k, "p <=> false", &[1]);
    assert_sat_set("readme_ex1", &k, "p <=> true", &[0, 2]);
}

#[test]
fn boolean_ops_on_diamond() {
    // s0{}, s1{P}, s2{Q}, s3{P,Q} : p = {1,3}, q = {2,3}
    let k = diamond();
    assert_sat_set("diamond", &k, "p | q", &[1, 2, 3]);
    assert_sat_set("diamond", &k, "!(p | q)", &[0]);
    assert_sat_set("diamond", &k, "p & q", &[3]);
    assert_sat_set("diamond", &k, "p <=> q", &[0, 3]);
    assert_sat_set("diamond", &k, "p => q", &[0, 2, 3]);
}

#[test]
fn collect_leaves_gathers_atoms_and_constants() {
    // ((p)&(q)) | ((p)&(true)) : atoms {p, q}, a True leaf, no False leaf
    let phi = parse("(p & q) | (p & true)");
    let leaves = phi.collect_leaves();

    let p = CTLFormula::Leaf(CTLFormulaLeaf::AtomicProp(TestAtomicProp::P));
    let q = CTLFormula::Leaf(CTLFormulaLeaf::AtomicProp(TestAtomicProp::Q));
    assert_eq!(leaves.atoms.len(), 2);
    assert!(leaves.atoms.contains(&p));
    assert!(leaves.atoms.contains(&q));
    assert!(leaves.true_formula.is_some());
    assert!(leaves.false_formula.is_none());
}

#[test]
fn collect_leaves_on_single_atom() {
    let phi = parse("p");
    let leaves = phi.collect_leaves();
    assert_eq!(leaves.atoms.len(), 1);
    assert!(leaves.true_formula.is_none());
    assert!(leaves.false_formula.is_none());
}
