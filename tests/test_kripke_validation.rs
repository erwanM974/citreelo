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

//! Tests of the validation performed by `KripkeStructure::new`:
//! structures with deadlock states or out-of-range transition targets
//! cannot be constructed, so the model checker only ever operates on
//! total, well-formed structures.

use citreelo::kripke::{KripkeStructure, KripkeStructureBuildError};

mod common;

use common::model::{TestAtomicProp::*, TestDomainOfAp, st};

#[test]
fn accepts_valid_structures() {
    assert!(
        KripkeStructure::new(vec![st(&[P], &[1, 2]), st(&[Q], &[1]), st(&[P, Q], &[0]),]).is_ok()
    );
    // a single state looping on itself is total
    assert!(KripkeStructure::new(vec![st(&[P], &[0]),]).is_ok());
    // duplicate targets are tolerated
    assert!(KripkeStructure::new(vec![st(&[P], &[1, 1, 0]), st(&[Q], &[0]),]).is_ok());
}

#[test]
fn accepts_the_empty_structure() {
    // no state means no deadlock and no transition: vacuously valid
    assert!(KripkeStructure::<TestDomainOfAp>::new(vec![]).is_ok());
}

#[test]
fn rejects_deadlock_states() {
    // a state without outgoing transitions would make universal
    // operators (AX, AF, AU, ...) vacuously true on it
    let result = KripkeStructure::new(vec![st(&[P], &[1]), st(&[Q], &[])]);
    assert_eq!(
        result.err(),
        Some(KripkeStructureBuildError::DeadlockState { state_id: 1 })
    );
}

#[test]
fn rejects_out_of_range_targets() {
    // a transition to a state that does not exist
    let result = KripkeStructure::new(vec![st(&[P], &[0, 5]), st(&[Q], &[0])]);
    assert_eq!(
        result.err(),
        Some(KripkeStructureBuildError::OutOfRangeTransitionTarget {
            origin_state_id: 0,
            target_state_id: 5,
            num_states: 2
        })
    );
}

#[test]
fn accepts_large_structures() {
    // the number of states is not capped: a structure of n states
    // needs only 2*ceil(log2(n)) BDD variables, which always fits the
    // 16-bit variable index of the BDD library
    let self_loops = |n: usize| (0..n).map(|i| st(&[P], &[i])).collect::<Vec<_>>();
    assert!(KripkeStructure::new(self_loops(40_000)).is_ok());
}

#[test]
fn build_errors_have_readable_messages() {
    let deadlock = KripkeStructureBuildError::DeadlockState { state_id: 3 };
    let msg = deadlock.to_string();
    assert!(msg.contains("state 3"), "unhelpful message: {}", msg);
    assert!(msg.contains("deadlock"), "unhelpful message: {}", msg);

    let out_of_range = KripkeStructureBuildError::OutOfRangeTransitionTarget {
        origin_state_id: 0,
        target_state_id: 5,
        num_states: 2,
    };
    let msg = out_of_range.to_string();
    assert!(
        msg.contains('5') && msg.contains('2'),
        "unhelpful message: {}",
        msg
    );
}
