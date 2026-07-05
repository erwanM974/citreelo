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

use nom::{Parser, branch::alt, bytes::complete::tag, combinator::value};

use citreelo::{
    ctl::{CTLFormula, CTLFormulaLeaf},
    parser::{CtlFormulaParser, CtlParseError},
};

use crate::common::model::TestAtomicProp;

/// Concrete parser used by the test suite: the atomic propositions
/// are `p`, `q` and `r` (`true` / `false` and all the operators are
/// handled by the core grammar).
pub struct CtlConcreteParser {}

impl CtlFormulaParser<TestAtomicProp> for CtlConcreteParser {
    fn parse_atomic_proposition<'a, E: nom::error::ParseError<&'a str>>(
        &self,
        input: &'a str,
    ) -> nom::IResult<&'a str, CTLFormula<TestAtomicProp>, E> {
        alt((
            value(
                CTLFormula::Leaf(CTLFormulaLeaf::AtomicProp(TestAtomicProp::P)),
                tag("p"),
            ),
            value(
                CTLFormula::Leaf(CTLFormulaLeaf::AtomicProp(TestAtomicProp::Q)),
                tag("q"),
            ),
            value(
                CTLFormula::Leaf(CTLFormulaLeaf::AtomicProp(TestAtomicProp::R)),
                tag("r"),
            ),
        ))
        .parse(input)
    }
}

/// Parses a whole formula, panicking with context on failure.
pub fn parse(input: &str) -> CTLFormula<TestAtomicProp> {
    match parse_complete(input) {
        Ok(phi) => phi,
        Err(e) => panic!("could not parse CTL formula {:?} : {}", input, e),
    }
}

/// Parses a whole formula (the crate's strict entry point).
pub fn parse_complete(input: &str) -> Result<CTLFormula<TestAtomicProp>, CtlParseError> {
    CtlConcreteParser {}.parse_complete_ctl_formula(input)
}

/// Raw access to the combinator-level parser, exposing the unconsumed
/// remainder. Used by parser tests that pin the remainder behavior.
pub fn parse_partial(input: &str) -> Result<(&str, CTLFormula<TestAtomicProp>), String> {
    let parser = CtlConcreteParser {};
    parser
        .parse_ctl_formula::<nom::error::Error<&str>>(input)
        .map_err(|e| format!("parse error: {:?}", e))
}
