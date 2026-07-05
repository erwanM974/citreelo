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

//! Benchmarks of [CtlModelChecker::get_sat_set] on a pre-built checker
//! (the BDD representation is constructed outside the timed loop; see
//! bench_build.rs for the construction cost).
//!
//! There is one group per cost mechanism of the solver rather than one
//! per operator :
//! - least fixpoints whose iteration count is the model diameter
//!   (`EF` / `AF` / `A[_ U _]` on chains),
//! - greatest fixpoints with full-diameter unwinding
//!   (`AG` / `EG` on cycles, where the one !p state empties the set
//!   one state per iteration),
//! - nested fixpoints (`AG EF`),
//! - dense models, where the transition-relation BDD is the stress,
//! - a seeded random batch approximating an average caller workload.
//!
//! Each scaling group sweeps the model size so that criterion reports
//! a curve per mechanism.

use std::hint::black_box;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use citreelo::kripke::KripkeStructure;
use citreelo::solve::CtlModelChecker;

#[path = "../tests/common/mod.rs"]
mod common;

use common::generators::{random_formulas, random_total_kripke};
use common::model::TestDomainOfAp;
use common::parser::parse;
use common::zoo::{chain, cycle, total_dense};

/// Model sizes for the scaling groups (shared with bench_build.rs).
///
/// When changing them, save a fresh reference baseline (see bench.sh),
/// as comparisons only cover the sizes present in the baseline.
const SIZES: [usize; 6] = [4, 8, 12, 16, 32, 64];

/// One scaling group : solves `formula_str` on `model_of(n)` for every
/// size in [SIZES], with the checker built outside the timed loop.
fn bench_formula_scaling(
    c: &mut Criterion,
    group_name: &str,
    formula_str: &str,
    model_of: impl Fn(usize) -> KripkeStructure<TestDomainOfAp>,
) {
    let phi = parse(formula_str);
    let mut group = c.benchmark_group(group_name);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));
    group.sample_size(20);
    for n in SIZES {
        let kripke = model_of(n);
        let checker = CtlModelChecker::new(&kripke);
        group.bench_with_input(BenchmarkId::from_parameter(n), &checker, |b, checker| {
            b.iter(|| checker.get_sat_set(black_box(&phi)));
        });
    }
    group.finish();
}

fn solve_least_fixpoints(c: &mut Criterion) {
    // on chain(n), q holds only on the last state : the fixpoint has
    // to walk the whole diameter backwards
    bench_formula_scaling(c, "solve/EF_on_chain", "EF q", chain);
    bench_formula_scaling(c, "solve/AF_on_chain", "AF q", chain);
    bench_formula_scaling(c, "solve/AU_on_chain", "A[p U q]", chain);
}

fn solve_greatest_fixpoints(c: &mut Criterion) {
    // on cycle(n), p fails only on s0 : the greatest fixpoint removes
    // one state per iteration all the way around the ring
    bench_formula_scaling(c, "solve/AG_on_cycle", "AG p", cycle);
    bench_formula_scaling(c, "solve/EG_on_cycle", "EG p", cycle);
}

fn solve_nested_fixpoints(c: &mut Criterion) {
    bench_formula_scaling(c, "solve/AG_EF_on_chain", "AG EF p", chain);
}

fn solve_dense(c: &mut Criterion) {
    // a fixed battery of formulae per iteration : on complete graphs
    // the fixpoints converge in very few iterations, so the cost is
    // dominated by BDD operations against the big transition relation
    let formulas = ["AG (p | q)", "A[p U q]", "EG p", "AG EF p"].map(parse);
    let mut group = c.benchmark_group("solve/dense_battery");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));
    group.sample_size(20);
    for n in [4usize, 8, 12, 16, 32] {
        let kripke = total_dense(n);
        let checker = CtlModelChecker::new(&kripke);
        group.bench_with_input(BenchmarkId::from_parameter(n), &checker, |b, checker| {
            b.iter(|| {
                for phi in &formulas {
                    black_box(checker.get_sat_set(black_box(phi)));
                }
            });
        });
    }
    group.finish();
}

fn solve_random_workload(c: &mut Criterion) {
    // 20 seeded random formulae on a seeded sparse random model, all
    // solved per iteration : an "average workload" smoothing out the
    // per-formula variance
    let kripke = random_total_kripke(0xC0FFEE, 12, 3);
    let checker = CtlModelChecker::new(&kripke);
    let formulas = random_formulas(0xFEED5EED, 20, 3);
    let mut group = c.benchmark_group("solve/random_workload");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(3));
    group.sample_size(20);
    group.bench_function("20_formulas_depth3_on_12_states", |b| {
        b.iter(|| {
            for phi in &formulas {
                black_box(checker.get_sat_set(black_box(phi)));
            }
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    solve_least_fixpoints,
    solve_greatest_fixpoints,
    solve_nested_fixpoints,
    solve_dense,
    solve_random_workload
);
criterion_main!(benches);
