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

//! Differential testing: the symbolic BDD-based checker must agree
//! with the naive explicit-state oracle (tests/common/oracle.rs)
//! on every model / formula pair.
//!
//! This is the primary safety net for refactoring the solver: it
//! covers operator combinations no hand-written test thinks of.
//! Everything is deterministic (seeded LCG), so failures reproduce.
//!
//! The sweeps are parallelized across models with scoped threads and
//! reuse one [CtlModelChecker] per model, so the BDD representation of
//! each model is built once per sweep; together with the dependency
//! opt-level override in Cargo.toml this keeps the whole file in the
//! seconds range despite thousands of solver calls.

use citreelo::kripke::KripkeStructure;
use citreelo::solve::CtlModelChecker;

mod common;

use common::asserts::assert_matches_oracle;
use common::generators::{
    all_operator_pair_formulas, all_single_operator_formulas, random_formulas, random_total_kripke,
};
use common::model::TestDomainOfAp;
use common::zoo::all_total_models;

/// Runs `job` once per (named) model, in parallel, and propagates the
/// first panic (i.e. the first assertion failure) to the test harness.
fn for_each_model_in_parallel(
    models: Vec<(String, KripkeStructure<TestDomainOfAp>)>,
    job: impl Fn(&str, &KripkeStructure<TestDomainOfAp>) + Sync,
) {
    std::thread::scope(|scope| {
        let handles: Vec<_> = models
            .iter()
            .map(|(name, kripke)| {
                let job = &job;
                scope.spawn(move || job(name, kripke))
            })
            .collect();
        for handle in handles {
            if let Err(panic_payload) = handle.join() {
                std::panic::resume_unwind(panic_payload);
            }
        }
    });
}

fn zoo() -> Vec<(String, KripkeStructure<TestDomainOfAp>)> {
    all_total_models()
        .into_iter()
        .map(|(name, kripke)| (name.to_string(), kripke))
        .collect()
}

#[test]
fn every_operator_over_leaves_on_zoo_models() {
    // every operator applied to every leaf combination
    // (190 formulae) on every zoo model
    let formulas = all_single_operator_formulas();
    for_each_model_in_parallel(zoo(), |name, kripke| {
        let checker = CtlModelChecker::new(kripke);
        for phi in &formulas {
            assert_matches_oracle(name, &checker, phi);
        }
    });
}

#[test]
fn every_operator_pair_on_zoo_models() {
    // every (outer operator, inner operator) composition
    // (1455 formulae) on every zoo model -- this is the sweep that
    // catches transition-relation encoding bugs
    let formulas = all_operator_pair_formulas();
    for_each_model_in_parallel(zoo(), |name, kripke| {
        let checker = CtlModelChecker::new(kripke);
        for phi in &formulas {
            assert_matches_oracle(name, &checker, phi);
        }
    });
}

// Budgets for the randomized sweeps. Kept deliberately modest: deep
// formulae on dense models hit near-worst-case BDD sizes, so volume
// is what drives the runtime of this file.
// Bump these locally for a more thorough hunt (e.g. before a release).
const RANDOM_FORMULAS_PER_MODEL: usize = 60;
const RANDOM_FORMULA_MAX_DEPTH: usize = 3;
const RANDOM_MODEL_COUNT: u64 = 12;

#[test]
fn random_deep_formulas_on_zoo_models() {
    // deeper formulae than the exhaustive sweep, seeded per model
    // (from its name, so the cases stay stable when the zoo grows)
    for_each_model_in_parallel(zoo(), |name, kripke| {
        let checker = CtlModelChecker::new(kripke);
        let seed = 0xC17EE10 + name.bytes().map(u64::from).sum::<u64>();
        for phi in random_formulas(seed, RANDOM_FORMULAS_PER_MODEL, RANDOM_FORMULA_MAX_DEPTH) {
            assert_matches_oracle(name, &checker, &phi);
        }
    });
}

#[test]
fn random_formulas_on_random_models() {
    // pseudo-random total structures of 1..=6 states with fan-out
    // up to 3 (duplicate targets included on purpose), each checked
    // against random formulae
    let models: Vec<(String, KripkeStructure<TestDomainOfAp>)> = (0..RANDOM_MODEL_COUNT)
        .map(|seed| {
            let n_states = 1 + (seed as usize % 6);
            (
                format!("random_model_seed{}", seed),
                random_total_kripke(seed, n_states, 3),
            )
        })
        .collect();
    for_each_model_in_parallel(models, |name, kripke| {
        let checker = CtlModelChecker::new(kripke);
        // recover the seed from the model name
        let seed: u64 = name
            .trim_start_matches("random_model_seed")
            .parse()
            .unwrap();
        for phi in random_formulas(
            0xFEED + seed,
            RANDOM_FORMULAS_PER_MODEL,
            RANDOM_FORMULA_MAX_DEPTH,
        ) {
            assert_matches_oracle(name, &checker, &phi);
        }
    });
}
