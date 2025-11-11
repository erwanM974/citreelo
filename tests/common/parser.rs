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



use nom::{Parser, branch::alt, bytes::complete::tag, combinator::{map, value}};

use citreelo::{ctl::{CTLFormula, CTLFormulaLeaf}, parser::CtlFormulaParser};

use crate::common::model::TestAtomicProp;








pub struct CtlConcreteParser {}

impl CtlFormulaParser<TestAtomicProp> for CtlConcreteParser {
    fn parse_atomic_proposition<'a, E: nom::error::ParseError<&'a str>>(
        &self,
        input : &'a str
    ) -> nom::IResult<&'a str, CTLFormula<TestAtomicProp>, E> {
        map(
            alt((
                value(TestAtomicProp::P, tag("p")),
                value(TestAtomicProp::Q, tag("q")),
            )),
            |x| CTLFormula::Leaf(CTLFormulaLeaf::AtomicProp(x))
        )
        .parse(input)
    }
}