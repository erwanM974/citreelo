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



use map_macro::{hash_map, hash_set};

use crate::{kripke::{KripkeState, KripkeStructure}, parser::CtlFormulaParser, solve::{get_sat_set, is_ctl_formula_sat}, test::{model::{TestAtomicProp, TestDomainOfAp}, parser::CtlConcreteParser}};







pub fn get_example_1() -> KripkeStructure<TestDomainOfAp> {
    let states = vec![
        KripkeState::new(
            TestDomainOfAp::new(hash_set! {TestAtomicProp::P}), 
            vec![1,2]
        ),
        KripkeState::new(
            TestDomainOfAp::new(hash_set! {TestAtomicProp::Q}), 
            vec![1]
        ),
        KripkeState::new(
            TestDomainOfAp::new(hash_set! {TestAtomicProp::P,TestAtomicProp::Q}), 
            vec![0]
        ),
    ];
    KripkeStructure::new(states)
}



#[test]
pub fn test_ex1_sat_set1() {
    let kripke = get_example_1();

    let sat_sets = hash_map! {
        // ***
        "p"                   => hash_set! {0,2},
        "!(p)"                => hash_set! {1},
        "q"                   => hash_set! {1,2},
        "!(q)"                => hash_set! {0},
        "(p)&(q)"             => hash_set! {2},
        "(p)|(q)"             => hash_set! {0,1,2},
        "(!(p))&(!(q))"       => hash_set! {},
        "(!(p))|(!(q))"       => hash_set! {0,1},
        // ***
        "E(X(p))"             => hash_set! {0,2},
        "E(X(q))"             => hash_set! {0,1},
        "E(X((p)&(q)))"       => hash_set! {0},
        "E(X((p)&(!(q))))"    => hash_set! {2},
        "E(X((!(p))&(!(q))))" => hash_set! {},
        // ***
        "A(X(p))"             => hash_set! {2},
        "A(X(q))"             => hash_set! {0,1},
        "A(X((q)&(!(p))))"    => hash_set! {1},
        "A(X((p)&(!(q))))"    => hash_set! {2},
        "A(X((p)&(q)))"       => hash_set! {},
    };

    let parser = CtlConcreteParser{};
    for (phi_as_str,expect_sat_set) in sat_sets {
        let (_,phi) = parser.parse_ctl_formula::<nom::error::Error<&str>>(phi_as_str).unwrap();
        let got_sat_set = get_sat_set(&kripke,&phi);
        assert_eq!(got_sat_set,expect_sat_set,"{} : \n{:?}\n", phi_as_str, phi);
    }
}




#[test]
pub fn test_ex1_sat_set2() {
    let kripke = get_example_1();

    let sat_sets = hash_map! {
        // ***
        "E((p)U(q))" => hash_set! {0,1,2},
        "A((p)U(q))" => hash_set! {0,1,2},
    };

    let parser = CtlConcreteParser{};
    for (phi_as_str,expect_sat_set) in sat_sets {
        let (_,phi) = parser.parse_ctl_formula::<nom::error::Error<&str>>(phi_as_str).unwrap();
        let got_sat_set = get_sat_set(&kripke,&phi);
        assert_eq!(got_sat_set,expect_sat_set,"{} : \n{:?}\n", phi_as_str, phi);
    }
}




#[test]
pub fn test_ex1_with_init_states1() {
    let kripke = get_example_1();
    

    let phis = hash_map! {
        "A(G(p))"       => false,  // Not all paths from s0​ always satisfy p (e.g., s0 → s1​)
        "A(F(q))"       => true,   // From s0 all paths eventually reach a state where q holds
        "E(F((p)&(q)))" => true,   // there exists a path from s0 to s2 where p&q holds
        "A(X(q))"       => true,   // q holds in both s1 and s2, which are the successors of s0
        "E(X(p))"       => true,   // there exists a successor of s0 (i.e. s2) where p holds
        "A(G((p)|(q)))" => true,   // in all states either p or q holds
        "A((q)U(p))"    => true,   
        "A((p)U(q))"    => true,   
    };

    let initial_states = hash_set! {0};
    let parser = CtlConcreteParser{};
    for (phi_as_str,is_sat) in phis {
        let (_,phi) = parser.parse_ctl_formula::<nom::error::Error<&str>>(phi_as_str).unwrap();
        let result = is_ctl_formula_sat(&kripke,&initial_states,&phi);
        assert_eq!(result,is_sat,"{} : \n{:?}\n", phi_as_str, phi);
    }
}

