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

//! Tests of the Graphviz drawer at the DOT-string level.
//!
//! Working on the DOT string directly means neither invoking the
//! external `dot` binary nor writing image artifacts.

use citreelo::util::viz_kripke::KripkeStructureGraphvizDrawer;
use graphviz_dot_builder::traits::DotTranslatable;

mod common;

use common::drawer::TestKripkeDrawer;
use common::zoo::readme_ex1;

#[test]
fn dot_output_contains_all_states_and_transitions() {
    let kripke = readme_ex1();
    let dot = TestKripkeDrawer {}.get_kripke_repr(&kripke).to_dot_string();

    // one node per state
    for node in ["st0", "st1", "st2"] {
        assert!(
            dot.contains(node),
            "missing node '{}' in DOT output:\n{}",
            node,
            dot
        );
    }
    // one edge per transition (the builder renders edges as "a->b;")
    for edge in ["st0->st1", "st0->st2", "st1->st1", "st2->st0"] {
        assert!(
            dot.contains(edge),
            "missing edge '{}' in DOT output:\n{}",
            edge,
            dot
        );
    }
    // no other edge got invented
    assert_eq!(
        dot.matches("->").count(),
        4,
        "unexpected number of edges:\n{}",
        dot
    );
}

#[test]
fn dot_output_contains_state_labels() {
    let kripke = readme_ex1();
    let dot = TestKripkeDrawer {}.get_kripke_repr(&kripke).to_dot_string();

    // the drawer labels each state with its id and its atomic propositions
    for label in ["s0", "s1", "s2", "{P}", "{Q}", "{P,Q}"] {
        assert!(
            dot.contains(label),
            "missing label '{}' in DOT output:\n{}",
            label,
            dot
        );
    }
}
