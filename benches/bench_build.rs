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

//! Benchmarks of [CtlModelChecker::new], i.e. the construction of the
//! BDD representation of a Kripke structure (dominated by the encoding
//! of the transition relation).
//!
//! Sparse families (edge count ~ n) and dense ones
//! (edge count n²) are kept in separate groups because they stress
//! the encoding differently.
//!
//! Timed iterations include dropping the built representation.

use std::hint::black_box;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use citreelo::solve::CtlModelChecker;

#[path = "../tests/common/mod.rs"]
mod common;

use common::generators::random_total_kripke;
use common::zoo::{chain, cycle, fanout, total_dense};

fn build_sparse(c: &mut Criterion) {
    let mut group = c.benchmark_group("build/sparse");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));
    group.sample_size(20);
    // sweep sizes are shared with bench_solve.rs; when changing them,
    // save a fresh reference baseline (see bench.sh), as comparisons
    // only cover the sizes present in the baseline
    for n in [4usize, 8, 12, 16, 32, 64] {
        // fanout(k) has k + 1 states
        let models = [
            ("chain", chain(n)),
            ("cycle", cycle(n)),
            ("fanout", fanout(n - 1)),
        ];
        for (family, kripke) in &models {
            group.bench_with_input(BenchmarkId::new(*family, n), kripke, |b, kripke| {
                b.iter(|| CtlModelChecker::new(black_box(kripke)));
            });
        }
    }
    group.finish();
}

fn build_dense(c: &mut Criterion) {
    let mut group = c.benchmark_group("build/dense");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));
    group.sample_size(20);
    for n in [4usize, 8, 12, 16, 32, 64] {
        let kripke = total_dense(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &kripke, |b, kripke| {
            b.iter(|| CtlModelChecker::new(black_box(kripke)));
        });
    }
    group.finish();
}

fn build_random(c: &mut Criterion) {
    // sparse pseudo-random structures : the "realistic caller" shape
    let mut group = c.benchmark_group("build/random_sparse");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));
    group.sample_size(20);
    for n in [8usize, 12, 16, 32, 64] {
        let kripke = random_total_kripke(0xB111D + n as u64, n, 3);
        group.bench_with_input(BenchmarkId::from_parameter(n), &kripke, |b, kripke| {
            b.iter(|| CtlModelChecker::new(black_box(kripke)));
        });
    }
    group.finish();
}

criterion_group!(benches, build_sparse, build_dense, build_random);
criterion_main!(benches);
