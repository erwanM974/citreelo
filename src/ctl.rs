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


use std::{collections::HashSet, hash::Hash};




#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum UnaryCTLOperator {
    Not,
    AX,
    EX,
    AF,
    EF,
    AG,
    EG
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum BinaryCTLOperator {
    And,
    Or,
    Imply,
    Iff,
    AU,
    EU
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum CTLFormulaLeaf<AP> {
    True,
    False,
    AtomicProp(AP),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum CTLFormula<AP> {
    Leaf(CTLFormulaLeaf<AP>),
    // ***
    Unary(UnaryCTLOperator,Box<CTLFormula<AP>>),
    Binary(BinaryCTLOperator,Box<CTLFormula<AP>>, Box<CTLFormula<AP>>)
}




impl<AP : Clone + PartialEq + Eq + Hash> CTLFormula<AP> {

    pub fn collect_leaves(&self) -> (
        HashSet<&CTLFormula<AP>>,
        Option<&CTLFormula<AP>>,
        Option<&CTLFormula<AP>>
    ) {
        let mut atoms = HashSet::new();
        let mut true_formula = None;
        let mut false_formula = None;
        self.collect_leaves_rec(&mut atoms, &mut true_formula, &mut false_formula);
        (atoms,true_formula,false_formula) 
    }

    fn collect_leaves_rec<'a>(
        &'a self, 
        atoms : &mut HashSet<&'a CTLFormula<AP>>,
        true_formula : &mut Option<&'a CTLFormula<AP>>,
        false_formula : &mut Option<&'a CTLFormula<AP>>,
    ) {
        match self {
            x @ CTLFormula::Leaf(leaf) => {
                match leaf {
                    CTLFormulaLeaf::True => {
                        if true_formula.is_none() {
                            *true_formula = Some(x);
                        }
                    },
                    CTLFormulaLeaf::False => {
                        if false_formula.is_none() {
                            *false_formula = Some(x);
                        }
                    },
                    CTLFormulaLeaf::AtomicProp(_) => {
                        atoms.insert(x);
                    },
                }
            }
            CTLFormula::Unary(_, phi1) => {
                phi1.collect_leaves_rec(atoms,true_formula,false_formula);
            },
            CTLFormula::Binary(_, phi1, phi2) => {
                phi1.collect_leaves_rec(atoms,true_formula,false_formula);
                phi2.collect_leaves_rec(atoms,true_formula,false_formula);
            }
        };
    }
}



