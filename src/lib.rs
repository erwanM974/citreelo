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

//! A simple BDD-based symbolic model checker for
//! [Computation Tree Logic](https://en.wikipedia.org/wiki/Computation_tree_logic) (CTL).
//!
//! - Models are [Kripke structures](kripke::KripkeStructure) whose states
//!   are labelled with values in a user-chosen domain, on which user-defined
//!   [atomic propositions](kripke::AtomicProposition) are evaluated.
//! - Properties are [CTL formulae](ctl::CTLFormula), built directly as an
//!   AST or parsed from a concrete syntax (see [parser]).
//! - Checking is symbolic : sets of states are manipulated as reduced
//!   ordered binary decision diagrams (via
//!   [biodivine-lib-bdd](https://docs.rs/biodivine-lib-bdd)), and all CTL
//!   operators are implemented directly as fixpoint computations rather
//!   than by rewriting to a minimal operator base.
//!
//! The main entry points are [solve::CtlModelChecker] (build once, query
//! many) and the one-shot functions [solve::get_sat_set] /
//! [solve::is_ctl_formula_sat].
//!
//! # Example
//!
//! ```
//! use std::collections::HashSet;
//!
//! use citreelo::kripke::{AtomicProposition, KripkeState, KripkeStructure};
//! use citreelo::parser::CtlFormulaParser;
//! use citreelo::solve::CtlModelChecker;
//!
//! // the domain labelling the states : whether "p" and "q" hold there
//! struct Props {
//!     p: bool,
//!     q: bool,
//! }
//!
//! // the atomic propositions evaluated on that domain
//! #[derive(Debug, Clone, PartialEq, Eq, Hash)]
//! enum Ap {
//!     P,
//!     Q,
//! }
//!
//! impl AtomicProposition<Props> for Ap {
//!     fn is_satisfied_on_state_domain(&self, props: &Props) -> bool {
//!         match self {
//!             Ap::P => props.p,
//!             Ap::Q => props.q,
//!         }
//!     }
//! }
//!
//! // a parser for the atomic propositions : the rest of the CTL grammar
//! // is provided by the CtlFormulaParser trait
//! struct ApParser;
//!
//! impl CtlFormulaParser<Ap> for ApParser {
//!     fn parse_atomic_proposition<'a, E: nom::error::ParseError<&'a str>>(
//!         &self,
//!         input: &'a str,
//!     ) -> nom::IResult<&'a str, citreelo::ctl::CTLFormula<Ap>, E> {
//!         use citreelo::ctl::{CTLFormula, CTLFormulaLeaf};
//!         use nom::Parser;
//!         nom::branch::alt((
//!             nom::character::complete::char('p')
//!                 .map(|_| CTLFormula::Leaf(CTLFormulaLeaf::AtomicProp(Ap::P))),
//!             nom::character::complete::char('q')
//!                 .map(|_| CTLFormula::Leaf(CTLFormulaLeaf::AtomicProp(Ap::Q))),
//!         ))
//!         .parse(input)
//!     }
//! }
//!
//! // a three-state structure :
//! //   s0 (p)   -> s1, s2
//! //   s1 (q)   -> s0
//! //   s2 (p,q) -> s2
//! let kripke = KripkeStructure::new(vec![
//!     KripkeState::new(Props { p: true, q: false }, vec![1, 2]),
//!     KripkeState::new(Props { p: false, q: true }, vec![0]),
//!     KripkeState::new(Props { p: true, q: true }, vec![2]),
//! ])
//! .unwrap();
//!
//! let checker = CtlModelChecker::new(&kripke);
//!
//! // "a state where both p and q hold is reachable"
//! let phi = ApParser.parse_complete_ctl_formula("EF (p & q)").unwrap();
//!
//! // ... holds on every state :
//! assert_eq!(checker.get_sat_set(&phi), HashSet::from([0, 1, 2]));
//! // ... and in particular from the initial state s0 :
//! assert!(checker.is_ctl_formula_sat(&HashSet::from([0]), &phi).unwrap());
//!
//! // "on every path, p keeps holding until q holds"
//! let psi = ApParser.parse_complete_ctl_formula("A[p U q]").unwrap();
//! assert_eq!(checker.get_sat_set(&psi), HashSet::from([0, 1, 2]));
//! ```

/// the BDD encoding of Kripke structures (internal)
pub mod bdd;
/// the CTL formula AST
pub mod ctl;
/// Kripke structures and their validating constructor
pub mod kripke;
/// a concrete syntax for CTL formulae, with operator precedence
pub mod parser;
/// the model-checking algorithms
pub mod solve;

/// visualization helpers (Graphviz rendering of Kripke structures)
pub mod util;
