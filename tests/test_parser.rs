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

//! Tests of the concrete CTL grammar: produced ASTs, operator
//! precedence and associativity, keyword boundaries, rejection of
//! malformed input, and printer/parser round-trips.
//! No model checking happens here.

use citreelo::ctl::{BinaryCTLOperator, CTLFormula, CTLFormulaLeaf, UnaryCTLOperator};
use citreelo::parser::CtlParseError;

use BinaryCTLOperator::*;
use UnaryCTLOperator::*;

mod common;

use common::generators::{
    all_operator_pair_formulas, all_single_operator_formulas, formula_to_string, random_formulas,
};
use common::model::TestAtomicProp;
use common::parser::{parse, parse_complete, parse_partial};

// small AST builders to keep expectations readable
fn p() -> CTLFormula<TestAtomicProp> {
    CTLFormula::Leaf(CTLFormulaLeaf::AtomicProp(TestAtomicProp::P))
}
fn q() -> CTLFormula<TestAtomicProp> {
    CTLFormula::Leaf(CTLFormulaLeaf::AtomicProp(TestAtomicProp::Q))
}
fn r() -> CTLFormula<TestAtomicProp> {
    CTLFormula::Leaf(CTLFormulaLeaf::AtomicProp(TestAtomicProp::R))
}
fn un(op: UnaryCTLOperator, phi: CTLFormula<TestAtomicProp>) -> CTLFormula<TestAtomicProp> {
    CTLFormula::Unary(op, Box::new(phi))
}
fn bin(
    op: BinaryCTLOperator,
    phi1: CTLFormula<TestAtomicProp>,
    phi2: CTLFormula<TestAtomicProp>,
) -> CTLFormula<TestAtomicProp> {
    CTLFormula::Binary(op, Box::new(phi1), Box::new(phi2))
}

#[test]
fn parses_leaves() {
    assert_eq!(parse("p"), p());
    assert_eq!(parse("q"), q());
    assert_eq!(parse("r"), r());
    assert_eq!(parse("true"), CTLFormula::Leaf(CTLFormulaLeaf::True));
    assert_eq!(parse("false"), CTLFormula::Leaf(CTLFormulaLeaf::False));
}

#[test]
fn parses_prefix_operators() {
    assert_eq!(parse("!p"), un(Not, p()));
    assert_eq!(parse("AX p"), un(AX, p()));
    assert_eq!(parse("EX p"), un(EX, p()));
    assert_eq!(parse("AF p"), un(AF, p()));
    assert_eq!(parse("EF p"), un(EF, p()));
    assert_eq!(parse("AG p"), un(AG, p()));
    assert_eq!(parse("EG p"), un(EG, p()));
    // parenthesized operand needs no space
    assert_eq!(parse("AX(p)"), un(AX, p()));
    assert_eq!(parse("!(p)"), un(Not, p()));
}

#[test]
fn parses_binary_operators() {
    assert_eq!(parse("p & q"), bin(And, p(), q()));
    assert_eq!(parse("p | q"), bin(Or, p(), q()));
    assert_eq!(parse("p => q"), bin(Imply, p(), q()));
    assert_eq!(parse("p <=> q"), bin(Iff, p(), q()));
    // spaces are optional around symbolic operators
    assert_eq!(parse("p&q"), bin(And, p(), q()));
    assert_eq!(parse("p|q"), bin(Or, p(), q()));
}

#[test]
fn parses_until() {
    assert_eq!(parse("A[p U q]"), bin(AU, p(), q()));
    assert_eq!(parse("E[p U q]"), bin(EU, p(), q()));
    // operands of U are full formulae
    assert_eq!(
        parse("E[p & q U r => p]"),
        bin(EU, bin(And, p(), q()), bin(Imply, r(), p()))
    );
    // temporal operands
    assert_eq!(parse("A[EX p U AG q]"), bin(AU, un(EX, p()), un(AG, q())));
    // nested untils
    assert_eq!(parse("A[E[p U q] U r]"), bin(AU, bin(EU, p(), q()), r()));
}

#[test]
fn precedence_of_boolean_connectives() {
    // ! > & > | > => > <=>
    assert_eq!(parse("p & q | r"), bin(Or, bin(And, p(), q()), r()));
    assert_eq!(parse("p | q & r"), bin(Or, p(), bin(And, q(), r())));
    assert_eq!(parse("!p & q"), bin(And, un(Not, p()), q()));
    assert_eq!(parse("!p | !q"), bin(Or, un(Not, p()), un(Not, q())));
    assert_eq!(
        parse("p & q => r | p"),
        bin(Imply, bin(And, p(), q()), bin(Or, r(), p()))
    );
    assert_eq!(parse("p => q <=> r"), bin(Iff, bin(Imply, p(), q()), r()));
}

#[test]
fn prefix_operators_bind_tighter_than_binary_ones() {
    assert_eq!(parse("AX p & q"), bin(And, un(AX, p()), q()));
    assert_eq!(parse("EF p | q"), bin(Or, un(EF, p()), q()));
    assert_eq!(parse("AG p => q"), bin(Imply, un(AG, p()), q()));
}

#[test]
fn associativity() {
    // left-associative: &, |, <=>
    assert_eq!(parse("p & q & r"), bin(And, bin(And, p(), q()), r()));
    assert_eq!(parse("p | q | r"), bin(Or, bin(Or, p(), q()), r()));
    assert_eq!(parse("p <=> q <=> r"), bin(Iff, bin(Iff, p(), q()), r()));
    // right-associative: =>
    assert_eq!(parse("p => q => r"), bin(Imply, p(), bin(Imply, q(), r())));
}

#[test]
fn chained_prefix_operators() {
    assert_eq!(parse("AG EF p"), un(AG, un(EF, p())));
    assert_eq!(parse("AX AX p"), un(AX, un(AX, p())));
    assert_eq!(parse("!AX !p"), un(Not, un(AX, un(Not, p()))));
    assert_eq!(parse("!!p"), un(Not, un(Not, p())));
}

#[test]
fn parentheses_override_precedence() {
    assert_eq!(parse("AX (p & q)"), un(AX, bin(And, p(), q())));
    assert_eq!(parse("(p | q) & r"), bin(And, bin(Or, p(), q()), r()));
    assert_eq!(parse("p & (q | r)"), bin(And, p(), bin(Or, q(), r())));
    assert_eq!(parse("!(p & q)"), un(Not, bin(And, p(), q())));
}

#[test]
fn tolerates_whitespace() {
    assert_eq!(parse("  AX   p  "), un(AX, p()));
    assert_eq!(parse(" p "), p());
    assert_eq!(parse("A [ p U q ]"), bin(AU, p(), q()));
    assert_eq!(
        parse("AG ( p => EF q )"),
        un(AG, bin(Imply, p(), un(EF, q())))
    );
}

#[test]
fn keyword_boundaries() {
    // "AXp" is neither the keyword AX (no word boundary) nor an atom
    assert!(parse_complete("AXp").is_err());
    assert!(parse_complete("AXE").is_err());
    assert!(parse_complete("truep").is_err());
    // bracket until requires word boundaries around U
    assert!(parse_complete("A[pUq]").is_err());
    assert!(parse_complete("A[p U q]").is_ok());
}

#[test]
fn parses_deeply_nested_formula() {
    let depth = 50;
    let input = format!("{}{}", "!".repeat(depth), "p");
    let mut expected = p();
    for _ in 0..depth {
        expected = un(Not, expected);
    }
    assert_eq!(parse(&input), expected);
}

#[test]
fn rejects_malformed_input() {
    assert!(parse_complete("").is_err());
    assert!(parse_complete("&").is_err());
    assert!(parse_complete("p &").is_err());
    assert!(parse_complete("& p").is_err());
    assert!(parse_complete("(p").is_err());
    assert!(parse_complete("p)").is_err());
    assert!(parse_complete("AX").is_err());
    assert!(parse_complete("A[p U]").is_err());
    assert!(parse_complete("A[p q]").is_err());
    assert!(parse_complete("A[p U q").is_err());
    assert!(parse_complete("p q").is_err());
    assert!(parse_complete("z").is_err());
}

#[test]
fn error_variants_and_offsets() {
    // trailing input after a valid formula
    match parse_complete("p q") {
        Err(CtlParseError::TrailingInput { offset, near }) => {
            assert_eq!(offset, 2);
            assert_eq!(near, "q");
        }
        other => panic!("expected TrailingInput, got {:?}", other),
    }
    // no formula at all
    assert!(matches!(
        parse_complete("&"),
        Err(CtlParseError::SyntaxError { .. })
    ));
    // errors render with position information
    let msg = parse_complete("p q").unwrap_err().to_string();
    assert!(msg.contains("offset 2"), "unhelpful message: {}", msg);
}

#[test]
fn combinator_level_parser_leaves_a_remainder() {
    // the combinator-style trait method parses a prefix and returns
    // the rest (for embedding in larger grammars); the complete entry
    // point used everywhere else rejects such input
    let (rem, phi) = parse_partial("p extra").unwrap();
    assert_eq!(phi, p());
    assert_eq!(rem, " extra");
    assert!(parse_complete("p extra").is_err());
}

#[test]
fn printer_and_parser_roundtrip() {
    // printing with minimal parentheses and reparsing must yield the
    // same AST, for every single-operator and operator-pair formula
    // (~1.6k) plus a batch of random deep formulae
    let mut formulas = all_single_operator_formulas();
    formulas.extend(all_operator_pair_formulas());
    formulas.extend(random_formulas(0xC17EE10, 300, 4));
    for phi in formulas {
        let printed = formula_to_string(&phi);
        match parse_complete(&printed) {
            Ok(reparsed) => assert_eq!(reparsed, phi, "roundtrip mismatch for '{}'", printed),
            Err(e) => panic!("could not reparse printed formula '{}': {}", printed, e),
        }
    }
}
