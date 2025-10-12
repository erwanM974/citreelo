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



use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::rc::Rc;

use biodivine_lib_bdd::*;

use crate::bdd::KripkeStructureBddRepresentation;
use crate::bdd::PreImageKind;
use crate::ctl::*;
use crate::kripke::*;



pub fn get_sat_set<
    DOAP,
    AP : AtomicProposition<DOAP> + PartialEq + Eq + Clone + Hash, 
    >(
    kripke : &KripkeStructure<DOAP>,
    formula : &CTLFormula<AP>
) -> HashSet<usize> {
    
    let mc = KripkeStructureBddRepresentation::from_kripke_structure(kripke);
    let (mut sub_formulae_memoizer,true_bdd) = {
        let (atoms,true_formula,false_formula) = formula.collect_leaves();
        initialize_memoizer_at_leaves(
            kripke,
            &mc,
            atoms,
            true_formula,
            false_formula
        )
    };
    let sat_set_bdd = get_ctl_subformula_sat_set_rec(
        &mc,
        &true_bdd,
        &mut sub_formulae_memoizer,
        formula
    );
    
    let mut states = HashSet::new();
    let num_states = mc.raw_vars.len()/2;
    for st_id in 0..num_states {
        let bdd_with_only_that_state = mc.get_state_formula(st_id);
        if !sat_set_bdd.and(&bdd_with_only_that_state).is_false() {
            states.insert(st_id);
        }
    }
    states
}

pub fn is_ctl_formula_sat<
    DOAP,
    AP : AtomicProposition<DOAP> + PartialEq + Eq + Clone + Hash, 
    >(
    kripke : &KripkeStructure<DOAP>,
    initial_states : &HashSet<usize>,
    formula : &CTLFormula<AP>
) -> bool {
    
    let mc = KripkeStructureBddRepresentation::from_kripke_structure(kripke);

    let sat_set_bdd = {
        let (mut sub_formulae_memoizer,true_bdd) = {
            let (atoms,true_formula,false_formula) = formula.collect_leaves();
            initialize_memoizer_at_leaves(
                kripke,
                &mc,
                atoms,
                true_formula,
                false_formula
            )
        };
        get_ctl_subformula_sat_set_rec(
            &mc,
            &true_bdd,
            &mut sub_formulae_memoizer,
            formula
        )
    };
    
    let initial_states_bdd = mc.get_states_set_formula(initial_states);

    let implication = initial_states_bdd.imp(&sat_set_bdd);

    implication.is_true()
}


fn initialize_memoizer_at_leaves<
    'a,
    DOAP,
    AP : AtomicProposition<DOAP> + PartialEq + Eq + Clone + Hash
    >(
    kripke : &KripkeStructure<DOAP>,
    mc : &KripkeStructureBddRepresentation,
    atoms : HashSet<&'a CTLFormula<AP>>,
    true_formula : Option<&'a CTLFormula<AP>>,
    false_formula : Option<&'a CTLFormula<AP>>,
) -> (HashMap<&'a CTLFormula<AP>,Rc<Bdd>>, Rc<Bdd>) {
    // ***
    let mut atoms_memoizer = HashMap::new();
    for atom in atoms {
        atoms_memoizer.insert(atom, mc.var_set.mk_false());
    }
    for (stid,state) in kripke.states.iter().enumerate() {
        for (atom,bdd) in atoms_memoizer.iter_mut() {
            if let CTLFormula::Leaf(CTLFormulaLeaf::AtomicProp(ap)) = atom
                && ap.is_satisfied_on_state_domain(&state.value_in_domain) {
                    let var = mc.raw_vars.get(stid).unwrap();
                    *bdd = bdd.or(&mc.var_set.mk_var(*var));
                }
        }
    }
    // ***
    let mut sub_formulae_memoizer : HashMap<&'a CTLFormula<AP>,Rc<Bdd>> = atoms_memoizer.into_iter().map(|(k,v)| (k,Rc::new(v))).collect(); 
    let true_bdd = Rc::new(mc.var_set.mk_true());
    if let Some(x) = true_formula {
        sub_formulae_memoizer.insert(x, true_bdd.clone());
    }
    if let Some(x) = false_formula {
        sub_formulae_memoizer.insert(x, Rc::new(mc.var_set.mk_false()));
    }
    // ***
    (sub_formulae_memoizer,true_bdd)
}




fn get_ctl_subformula_sat_set_rec<
    'a,
    DOAP,
    AP : AtomicProposition<DOAP> + PartialEq + Eq + Clone + Hash
    >(
    mc : &KripkeStructureBddRepresentation,
    true_bdd : &Rc<Bdd>,
    sub_formulae_memoizer : &mut HashMap<&'a CTLFormula<AP>,Rc<Bdd>>,
    phi : &'a CTLFormula<AP>
) -> Rc<Bdd> {
    if let Some(got_bdd) = sub_formulae_memoizer.get(phi) {
        return got_bdd.clone();
    }
    let phi_bdd = match phi {
        CTLFormula::Unary(un_op, phi1) => {
            let bdd1= get_ctl_subformula_sat_set_rec(mc,true_bdd,sub_formulae_memoizer,phi1);
            match un_op {
                UnaryCTLOperator::Not => {
                    bdd1.not()
                },
                UnaryCTLOperator::AX => mc.get_pre_image_by_transition_relation(PreImageKind::Strong,&bdd1),
                UnaryCTLOperator::EX => mc.get_pre_image_by_transition_relation(PreImageKind::Weak,&bdd1),
                UnaryCTLOperator::AF => until_fixpoint(
                    true_bdd, bdd1, |x| mc.get_pre_image_by_transition_relation(PreImageKind::Strong, x)
                ),
                UnaryCTLOperator::EF => until_fixpoint(
                    true_bdd, bdd1, |x| mc.get_pre_image_by_transition_relation(PreImageKind::Weak, x)
                ),
                UnaryCTLOperator::AG => global_fixpoint(
                    bdd1, |x| mc.get_pre_image_by_transition_relation(PreImageKind::Strong, x)
                ),
                UnaryCTLOperator::EG => global_fixpoint(
                    bdd1, |x| mc.get_pre_image_by_transition_relation(PreImageKind::Weak, x)
                ),
            }
        },
        CTLFormula::Binary(bi_op, phi1, phi2) => {
            let bdd1 = get_ctl_subformula_sat_set_rec(mc,true_bdd,sub_formulae_memoizer,phi1);
            let bdd2 = get_ctl_subformula_sat_set_rec(mc,true_bdd,sub_formulae_memoizer,phi2);
            match bi_op {
                BinaryCTLOperator::And => bdd1.and(&bdd2),
                BinaryCTLOperator::Or => bdd1.or(&bdd2),
                BinaryCTLOperator::Imply => bdd1.imp(&bdd2),
                BinaryCTLOperator::Iff => bdd1.iff(&bdd2),
                BinaryCTLOperator::AU => until_fixpoint(
                    &bdd1, bdd2, |x| mc.get_pre_image_by_transition_relation(PreImageKind::Strong, x)
                ),
                BinaryCTLOperator::EU => until_fixpoint(
                    &bdd1, bdd2, |x| mc.get_pre_image_by_transition_relation(PreImageKind::Weak, x)
                ),
            }
        },
        CTLFormula::Leaf(_) => {
            panic!("leaf should have been preprocessed")
        },
    };
    sub_formulae_memoizer.entry(phi).insert_entry(Rc::new(phi_bdd)).get().clone()
}



fn global_fixpoint(
    bdd : Rc<Bdd>,
    step_fn : impl Fn(&Bdd) -> Bdd 
) -> Bdd {
    let mut current = (*bdd).clone();
    loop {
        let next = current.and(&step_fn(&current));
        if next == current {
            break;
        } else {
            current = next;
        }
    }
    current
}



fn until_fixpoint(
    before : &Rc<Bdd>,
    after : Rc<Bdd>,
    step_fn : impl Fn(&Bdd) -> Bdd 
) -> Bdd {
    let mut current = (*after).clone();
    loop {
        let next = current.or(&before.and(&step_fn(&current)));
        if next == current {
            break;
        } else {
            current = next;
        }
    }
    current
}