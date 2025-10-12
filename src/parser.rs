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


use nom::{branch::alt, bytes::complete::tag, character::complete::multispace0, combinator::{map, value}, error::ParseError, IResult, Parser};

use crate::ctl::*;



#[derive(Debug, PartialEq, Eq, Clone, Hash)]
enum ModalQuantifier {
    Always,
    Exists
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
enum PathQuantifier<AP> {
    Next(CTLFormula<AP>),
    Global(CTLFormula<AP>),
    Finally(CTLFormula<AP>),
    Until(CTLFormula<AP>,CTLFormula<AP>)
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
enum UnaryPathQuantifierKind {
    Next,
    Global,
    Finally
}


pub trait CtlFormulaParser<AP> : Sized {

    fn parse_atomic_proposition<'a, E: ParseError<&'a str>>(
        &self,
        input : &'a str
    ) -> IResult<&'a str, AP, E>;

    fn parse_ctl_formula<'a, E: ParseError<&'a str>>(
        &self,
        input : &'a str
    ) -> IResult<&'a str, CTLFormula<AP>,E> {
        alt((
            |x| parse_under_modal_quantifier(self, x),
            |x| parse_under_unary_logic_operator(self, x),
            |x| parse_under_binary_logic_operator(self, x),
            map(
            |x| self.parse_atomic_proposition(x),
            |x| CTLFormula::Leaf(CTLFormulaLeaf::AtomicProp(x))
            )
        )).parse(input)  
    }

}

fn parse_under_unary_logic_operator<'a, AP, E: ParseError<&'a str>>(
        formula_parser : &impl CtlFormulaParser<AP>,
        input : &'a str
    ) -> IResult<&'a str, CTLFormula<AP>,E> {
    match (
            multispace0,
            value((), tag("!")),
            multispace0,
            nom::character::complete::char('('),
            multispace0,
            |x| formula_parser.parse_ctl_formula(x),
            multispace0,
            nom::character::complete::char(')')
        ).parse(input) {
        IResult::Ok((rem, (_, _, _, _, _, sub_phi,_,_))) => {
            IResult::Ok((rem, CTLFormula::Unary(UnaryCTLOperator::Not, Box::new(sub_phi))))
        },
        IResult::Err(e) => {
            IResult::Err(e)
        }
    }
}

fn parse_under_binary_logic_operator<'a, AP, E: ParseError<&'a str>>(
    formula_parser : &impl CtlFormulaParser<AP>,
    input : &'a str
) -> IResult<&'a str, CTLFormula<AP>,E> {
    match (
        multispace0,
        nom::character::complete::char('('),
        multispace0,
        |x| formula_parser.parse_ctl_formula(x),
        multispace0,
        nom::character::complete::char(')'),
        multispace0,
        alt((
            value(BinaryCTLOperator::And, tag("&")),
            value(BinaryCTLOperator::Or, tag("|")),
            value(BinaryCTLOperator::Iff, tag("<=>")),
            value(BinaryCTLOperator::Imply, tag("=>")),
        )),
        multispace0,
        nom::character::complete::char('('),
        multispace0,
        |x| formula_parser.parse_ctl_formula(x),
        multispace0,
        nom::character::complete::char(')')
    ).parse(input) {
    IResult::Ok((rem, (_, _,_,left_phi, _, _, _, bi_op,_,_,_,right_phi,_,_))) => {
        IResult::Ok((rem, CTLFormula::Binary(bi_op,Box::new(left_phi),Box::new(right_phi))))
    },
    IResult::Err(e) => {
        IResult::Err(e)
    }
}
}



fn parse_under_modal_quantifier<'a, AP,E: ParseError<&'a str>>(
        formula_parser : &impl CtlFormulaParser<AP>,
        input : &'a str
    ) -> IResult<&'a str, CTLFormula<AP>,E> {
        match (
                multispace0,
                alt((
                        value(ModalQuantifier::Always, tag("A")),
                        value(ModalQuantifier::Exists, tag("E")),
                )),
                multispace0,
                nom::character::complete::char('('),
                multispace0,
                |x| parse_under_path_quantifier(formula_parser,x),
                multispace0,
                nom::character::complete::char(')')
            ).parse(input) {
            IResult::Ok((rem, (_, modal_q, _, _, _, path_q,_,_))) => {
                match path_q {
                    PathQuantifier::Next(sub_phi) => {
                        match modal_q {
                            ModalQuantifier::Always => {
                                IResult::Ok((rem, CTLFormula::Unary(UnaryCTLOperator::AX, Box::new(sub_phi))))
                            },
                            ModalQuantifier::Exists => {
                                IResult::Ok((rem, CTLFormula::Unary(UnaryCTLOperator::EX, Box::new(sub_phi))))
                            },
                        }
                    },
                    PathQuantifier::Global(sub_phi) => {
                        match modal_q {
                            ModalQuantifier::Always => {
                                IResult::Ok((rem, CTLFormula::Unary(UnaryCTLOperator::AG, Box::new(sub_phi))))
                            },
                            ModalQuantifier::Exists => {
                                IResult::Ok((rem, CTLFormula::Unary(UnaryCTLOperator::EG, Box::new(sub_phi))))
                            },
                        }
                    },
                    PathQuantifier::Finally(sub_phi) => {
                        match modal_q {
                            ModalQuantifier::Always => {
                                IResult::Ok((rem, CTLFormula::Unary(UnaryCTLOperator::AF, Box::new(sub_phi))))
                            },
                            ModalQuantifier::Exists => {
                                IResult::Ok((rem, CTLFormula::Unary(UnaryCTLOperator::EF, Box::new(sub_phi))))
                            },
                        }
                    },
                    PathQuantifier::Until(sub_phi1,sub_phi2) => {
                        match modal_q {
                            ModalQuantifier::Always => {
                                IResult::Ok((rem, CTLFormula::Binary(BinaryCTLOperator::AU, Box::new(sub_phi1),Box::new(sub_phi2))))
                            },
                            ModalQuantifier::Exists => {
                                IResult::Ok((rem, CTLFormula::Binary(BinaryCTLOperator::EU, Box::new(sub_phi1),Box::new(sub_phi2))))
                            },
                        }
                    },
                }
            },
            IResult::Err(e) => {
                IResult::Err(e)
            }
        }
    }

    fn parse_under_path_quantifier<'a, AP, E: ParseError<&'a str>>(
        formula_parser : &impl CtlFormulaParser<AP>,
        input : &'a str
    ) -> IResult<&'a str, PathQuantifier<AP>,E> {
        alt((
            |x| parse_under_unary_path_quantifier(formula_parser, x),
            |x| parse_under_binary_path_quantifier(formula_parser, x),
        )).parse(input)
    }

    fn parse_under_binary_path_quantifier<'a, AP, E: ParseError<&'a str>>(
        formula_parser : &impl CtlFormulaParser<AP>,
        input : &'a str
    ) -> IResult<&'a str, PathQuantifier<AP>,E> {
        match (
                multispace0,
                nom::character::complete::char('('),
                multispace0,
                |x| formula_parser.parse_ctl_formula(x),
                multispace0,
                nom::character::complete::char(')'),
                multispace0,
                value((), tag("U")),
                multispace0,
                nom::character::complete::char('('),
                multispace0,
                |x| formula_parser.parse_ctl_formula(x),
                multispace0,
                nom::character::complete::char(')')
            ).parse(input) {
            IResult::Ok((rem, (_, _,_,left_phi, _, _, _, _,_,_,_,right_phi,_,_))) => {
                IResult::Ok((rem, PathQuantifier::Until(left_phi,right_phi)))
            },
            IResult::Err(e) => {
                IResult::Err(e)
            }
        }
    }

    fn parse_under_unary_path_quantifier<'a, AP, E: ParseError<&'a str>>(
        formula_parser : &impl CtlFormulaParser<AP>,
        input : &'a str
    ) -> IResult<&'a str, PathQuantifier<AP>,E> {
        match (
                multispace0,
                alt((
                        value(UnaryPathQuantifierKind::Next, tag("X")),
                        value(UnaryPathQuantifierKind::Global, tag("G")),
                        value(UnaryPathQuantifierKind::Finally, tag("F")),
                )),
                multispace0,
                nom::character::complete::char('('),
                multispace0,
                |x| formula_parser.parse_ctl_formula(x),
                multispace0,
                nom::character::complete::char(')')
            ).parse(input) {
            IResult::Ok((rem, (_, upath_qk, _, _, _, sub_phi,_,_))) => {
                match upath_qk {
                    UnaryPathQuantifierKind::Next => {
                        IResult::Ok((rem, PathQuantifier::Next(sub_phi)))
                    },
                    UnaryPathQuantifierKind::Global => {
                        IResult::Ok((rem, PathQuantifier::Global(sub_phi)))
                    },
                    UnaryPathQuantifierKind::Finally => {
                        IResult::Ok((rem, PathQuantifier::Finally(sub_phi)))
                    },
                }
            },
            IResult::Err(e) => {
                IResult::Err(e)
            }
        }
    }


