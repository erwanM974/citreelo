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

//! A recursive-descent parser for CTL formulae with the usual operator
//! precedences, so that e.g. `AG (p => EF q)` can be written without
//! fully parenthesizing every sub-formula.
//!
//! # Grammar
//!
//! From weakest to strongest binding:
//!
//! | level | operators                              | associativity |
//! |-------|----------------------------------------|---------------|
//! | 1     | `<=>`                                  | left          |
//! | 2     | `=>`                                   | right         |
//! | 3     | `\|`                                   | left          |
//! | 4     | `&`                                    | left          |
//! | 5     | `!`, `AX`, `EX`, `AF`, `EF`, `AG`, `EG`| prefix        |
//! | 6     | atoms, `true`, `false`, `(φ)`, `A[φ U ψ]`, `E[φ U ψ]` | |
//!
//! Prefix operators chain (`AG EF p`, `!AX !p`) and bind tighter than
//! the binary connectives: `AX p & q` reads as `(AX p) & q`.
//! The until operators use the classic bracket notation `A[φ U ψ]` /
//! `E[φ U ψ]`, where φ and ψ are full formulae.
//!
//! Keywords (`AX`, ..., `A`, `E`, `U`, `true`, `false`) are matched up
//! to a word boundary, so user-defined atomic propositions whose names
//! merely start with a keyword (e.g. `AXE`) are not shadowed. Atoms
//! named exactly like a keyword are shadowed wherever the grammar
//! expects that keyword, and should be avoided.
//!
//! Atomic propositions themselves are parsed by the user-provided
//! [CtlFormulaParser::parse_atomic_proposition](crate::parser::CtlFormulaParser::parse_atomic_proposition).
//!
//! Use [CtlFormulaParser::parse_complete_ctl_formula](crate::parser::CtlFormulaParser::parse_complete_ctl_formula)
//! to parse a whole input string: unlike the combinator-style
//! [CtlFormulaParser::parse_ctl_formula](crate::parser::CtlFormulaParser::parse_ctl_formula),
//! it fails on trailing input instead of silently accepting a prefix of the formula.

use std::fmt;

use nom::{
    IResult, Parser,
    bytes::complete::tag,
    character::complete::{char as nom_char, multispace0},
    combinator::cut,
    error::{ErrorKind, ParseError},
};

use crate::ctl::*;

/// The reasons for which [CtlFormulaParser::parse_complete_ctl_formula]
/// may reject its input. Offsets are byte offsets into the input string.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CtlParseError {
    /// no formula could be recognized at this position
    SyntaxError { offset: usize, near: String },
    /// a formula was recognized but it does not span the whole input
    TrailingInput { offset: usize, near: String },
}

impl fmt::Display for CtlParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CtlParseError::SyntaxError { offset, near } => {
                write!(f, "CTL syntax error at offset {} near \"{}\"", offset, near)
            }
            CtlParseError::TrailingInput { offset, near } => {
                write!(
                    f,
                    "the CTL formula ends at offset {} but trailing input remains : \"{}\"",
                    offset, near
                )
            }
        }
    }
}

impl std::error::Error for CtlParseError {}

fn error_snippet(rest: &str) -> String {
    rest.chars().take(24).collect()
}

/// A parser for CTL formulae over atomic propositions of type `AP`.
///
/// Implementors only provide [Self::parse_atomic_proposition]; the
/// rest of the grammar (see the [module documentation](self)) comes
/// for free through the provided methods.
pub trait CtlFormulaParser<AP>: Sized {
    /// Parses a single atomic proposition (without leading whitespace).
    fn parse_atomic_proposition<'a, E: ParseError<&'a str>>(
        &self,
        input: &'a str,
    ) -> IResult<&'a str, CTLFormula<AP>, E>;

    /// Combinator-style parser: parses the longest formula at the start
    /// of `input` (leading whitespace allowed) and returns the rest.
    ///
    /// Most callers want [Self::parse_complete_ctl_formula] instead,
    /// which rejects trailing input.
    fn parse_ctl_formula<'a, E: ParseError<&'a str>>(
        &self,
        input: &'a str,
    ) -> IResult<&'a str, CTLFormula<AP>, E> {
        parse_iff_level(self, input)
    }

    /// Parses `input` as one CTL formula spanning the whole string
    /// (modulo surrounding whitespace).
    fn parse_complete_ctl_formula(&self, input: &str) -> Result<CTLFormula<AP>, CtlParseError> {
        match self.parse_ctl_formula::<nom::error::Error<&str>>(input) {
            Ok((rem, phi)) => {
                let trailing = rem.trim_start();
                if trailing.is_empty() {
                    Ok(phi)
                } else {
                    Err(CtlParseError::TrailingInput {
                        offset: input.len() - trailing.len(),
                        near: error_snippet(trailing),
                    })
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(CtlParseError::SyntaxError {
                    offset: input.len() - e.input.len(),
                    near: error_snippet(e.input),
                })
            }
            Err(nom::Err::Incomplete(_)) => Err(CtlParseError::SyntaxError {
                offset: input.len(),
                near: String::new(),
            }),
        }
    }
}

/// matches `keyword` up to a word boundary, so that e.g. an atomic
/// proposition named "AXE" is not shadowed by the keyword "AX"
fn parse_keyword<'a, E: ParseError<&'a str>>(
    keyword: &str,
    input: &'a str,
) -> IResult<&'a str, (), E> {
    let (rem, _) = tag(keyword).parse(input)?;
    if rem
        .chars()
        .next()
        .is_some_and(|c| c.is_alphanumeric() || c == '_')
    {
        Err(nom::Err::Error(E::from_error_kind(input, ErrorKind::Tag)))
    } else {
        Ok((rem, ()))
    }
}

/// level 1 : `<=>`, left-associative
fn parse_iff_level<'a, AP, P: CtlFormulaParser<AP>, E: ParseError<&'a str>>(
    formula_parser: &P,
    input: &'a str,
) -> IResult<&'a str, CTLFormula<AP>, E> {
    let (mut rem, mut formula) = parse_imply_level(formula_parser, input)?;
    while let Ok((after_op, _)) = (multispace0::<&'a str, E>, tag("<=>")).parse(rem) {
        let (after_rhs, rhs) = parse_imply_level(formula_parser, after_op)?;
        formula = CTLFormula::Binary(BinaryCTLOperator::Iff, Box::new(formula), Box::new(rhs));
        rem = after_rhs;
    }
    Ok((rem, formula))
}

/// level 2 : `=>`, right-associative
fn parse_imply_level<'a, AP, P: CtlFormulaParser<AP>, E: ParseError<&'a str>>(
    formula_parser: &P,
    input: &'a str,
) -> IResult<&'a str, CTLFormula<AP>, E> {
    let (rem, lhs) = parse_or_level(formula_parser, input)?;
    // NB: on "<=>" the tag below fails (it starts with '<'),
    // so iff/imply do not steal each other's operator
    if let Ok((after_op, _)) = (multispace0::<&'a str, E>, tag("=>")).parse(rem) {
        let (after_rhs, rhs) = parse_imply_level(formula_parser, after_op)?;
        Ok((
            after_rhs,
            CTLFormula::Binary(BinaryCTLOperator::Imply, Box::new(lhs), Box::new(rhs)),
        ))
    } else {
        Ok((rem, lhs))
    }
}

/// level 3 : `|`, left-associative
fn parse_or_level<'a, AP, P: CtlFormulaParser<AP>, E: ParseError<&'a str>>(
    formula_parser: &P,
    input: &'a str,
) -> IResult<&'a str, CTLFormula<AP>, E> {
    let (mut rem, mut formula) = parse_and_level(formula_parser, input)?;
    while let Ok((after_op, _)) = (multispace0::<&'a str, E>, nom_char('|')).parse(rem) {
        let (after_rhs, rhs) = parse_and_level(formula_parser, after_op)?;
        formula = CTLFormula::Binary(BinaryCTLOperator::Or, Box::new(formula), Box::new(rhs));
        rem = after_rhs;
    }
    Ok((rem, formula))
}

/// level 4 : `&`, left-associative
fn parse_and_level<'a, AP, P: CtlFormulaParser<AP>, E: ParseError<&'a str>>(
    formula_parser: &P,
    input: &'a str,
) -> IResult<&'a str, CTLFormula<AP>, E> {
    let (mut rem, mut formula) = parse_unary_level(formula_parser, input)?;
    while let Ok((after_op, _)) = (multispace0::<&'a str, E>, nom_char('&')).parse(rem) {
        let (after_rhs, rhs) = parse_unary_level(formula_parser, after_op)?;
        formula = CTLFormula::Binary(BinaryCTLOperator::And, Box::new(formula), Box::new(rhs));
        rem = after_rhs;
    }
    Ok((rem, formula))
}

/// level 5 : the prefix operators `!`, `AX`, `EX`, `AF`, `EF`, `AG`,
/// `EG`, plus the bracketed `A[φ U ψ]` / `E[φ U ψ]`
fn parse_unary_level<'a, AP, P: CtlFormulaParser<AP>, E: ParseError<&'a str>>(
    formula_parser: &P,
    input: &'a str,
) -> IResult<&'a str, CTLFormula<AP>, E> {
    let (input, _) = multispace0(input)?;
    // ***
    if let Ok((rem, _)) = nom_char::<&'a str, E>('!').parse(input) {
        let (rem, sub_phi) = cut(|i| parse_unary_level(formula_parser, i)).parse(rem)?;
        return Ok((
            rem,
            CTLFormula::Unary(UnaryCTLOperator::Not, Box::new(sub_phi)),
        ));
    }
    // ***
    let unary_temporal_keywords = [
        ("AX", UnaryCTLOperator::AX),
        ("EX", UnaryCTLOperator::EX),
        ("AF", UnaryCTLOperator::AF),
        ("EF", UnaryCTLOperator::EF),
        ("AG", UnaryCTLOperator::AG),
        ("EG", UnaryCTLOperator::EG),
    ];
    for (keyword, operator) in unary_temporal_keywords {
        if let Ok((rem, _)) = parse_keyword::<E>(keyword, input) {
            let (rem, sub_phi) = cut(|i| parse_unary_level(formula_parser, i)).parse(rem)?;
            return Ok((rem, CTLFormula::Unary(operator, Box::new(sub_phi))));
        }
    }
    // ***
    let until_keywords = [("A", BinaryCTLOperator::AU), ("E", BinaryCTLOperator::EU)];
    for (keyword, operator) in until_keywords {
        if let Ok((rem, _)) = parse_keyword::<E>(keyword, input)
            && let Ok((rem, _)) = (multispace0::<&'a str, E>, nom_char('[')).parse(rem)
        {
            // beyond "A[" / "E[" this can only be an until : commit
            let (rem, (phi1, phi2)) = cut(|i| parse_until_body(formula_parser, i)).parse(rem)?;
            return Ok((
                rem,
                CTLFormula::Binary(operator, Box::new(phi1), Box::new(phi2)),
            ));
        }
        // a bare "A" / "E" without '[' may still be an atomic
        // proposition : fall through to the primary level
    }
    // ***
    parse_primary(formula_parser, input)
}

/// the part after `A[` / `E[` : `φ U ψ ]`
fn parse_until_body<'a, AP, P: CtlFormulaParser<AP>, E: ParseError<&'a str>>(
    formula_parser: &P,
    input: &'a str,
) -> IResult<&'a str, (CTLFormula<AP>, CTLFormula<AP>), E> {
    let (rem, phi1) = parse_iff_level(formula_parser, input)?;
    let (rem, _) = multispace0(rem)?;
    let (rem, _) = parse_keyword::<E>("U", rem)?;
    let (rem, phi2) = parse_iff_level(formula_parser, rem)?;
    let (rem, _) = multispace0(rem)?;
    let (rem, _) = nom_char(']').parse(rem)?;
    Ok((rem, (phi1, phi2)))
}

/// level 6 : `true`, `false`, parenthesized formulae and the
/// user-provided atomic propositions
fn parse_primary<'a, AP, P: CtlFormulaParser<AP>, E: ParseError<&'a str>>(
    formula_parser: &P,
    input: &'a str,
) -> IResult<&'a str, CTLFormula<AP>, E> {
    let (input, _) = multispace0(input)?;
    // ***
    if let Ok((rem, _)) = nom_char::<&'a str, E>('(').parse(input) {
        let (rem, phi) = cut(|i| parse_iff_level(formula_parser, i)).parse(rem)?;
        let (rem, _) = cut((multispace0, nom_char(')'))).parse(rem)?;
        return Ok((rem, phi));
    }
    // ***
    if let Ok((rem, _)) = parse_keyword::<E>("true", input) {
        return Ok((rem, CTLFormula::Leaf(CTLFormulaLeaf::True)));
    }
    if let Ok((rem, _)) = parse_keyword::<E>("false", input) {
        return Ok((rem, CTLFormula::Leaf(CTLFormulaLeaf::False)));
    }
    // ***
    formula_parser.parse_atomic_proposition(input)
}
