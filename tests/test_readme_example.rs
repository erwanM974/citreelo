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

//! The worked example from the README, with the formulae written
//! exactly as they appear in the README tables, so that the README
//! stays truthful.

use map_macro::hash_map;

mod common;

use common::asserts::assert_sat_set;
use common::zoo::readme_ex1;

#[test]
fn readme_boolean_table() {
    let kripke = readme_ex1();
    let sat_sets = hash_map! {
        "p"             => vec![0, 2],
        "!p"            => vec![1],
        "q"             => vec![1, 2],
        "!q"            => vec![0],
        "p&q"           => vec![2],
        "p|q"           => vec![0, 1, 2],
        "(!p)&(!q)"     => vec![],
        "(!p)|(!q)"     => vec![0, 1],
    };
    for (phi_as_str, expected) in sat_sets {
        assert_sat_set("readme_ex1", &kripke, phi_as_str, &expected);
    }
}

#[test]
fn readme_ex_table() {
    let kripke = readme_ex1();
    let sat_sets = hash_map! {
        "EX(p)"         => vec![0, 2],
        "EX(q)"         => vec![0, 1],
        "EX(p&q)"       => vec![0],
        "EX(p&(!q))"    => vec![2],
        "EX((!p)&(!q))" => vec![],
    };
    for (phi_as_str, expected) in sat_sets {
        assert_sat_set("readme_ex1", &kripke, phi_as_str, &expected);
    }
}

#[test]
fn readme_ax_table() {
    let kripke = readme_ex1();
    let sat_sets = hash_map! {
        "AX(p)"         => vec![2],
        "AX(q)"         => vec![0, 1],
        "AX(q&(!p))"    => vec![1],
        "AX(p&(!q))"    => vec![2],
        "AX(p&q)"       => vec![],
    };
    for (phi_as_str, expected) in sat_sets {
        assert_sat_set("readme_ex1", &kripke, phi_as_str, &expected);
    }
}

#[test]
fn readme_until_examples() {
    let kripke = readme_ex1();
    let sat_sets = hash_map! {
        "E[p U q]" => vec![0, 1, 2],
        "A[p U q]" => vec![0, 1, 2],
    };
    for (phi_as_str, expected) in sat_sets {
        assert_sat_set("readme_ex1", &kripke, phi_as_str, &expected);
    }
}
