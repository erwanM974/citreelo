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

//! Benchmarks of the CTL parser
//! ([CtlFormulaParser::parse_complete_ctl_formula]) on three synthetic
//! input shapes that stress different parts of the grammar (flat
//! left-associative folding, parenthesis recursion, prefix-operator
//! recursion) plus seeded random formulae re-printed by the test
//! suite's precedence-aware printer.
//!
//! Throughput is reported in bytes so that numbers stay comparable
//! when input sizes change.

use std::hint::black_box;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

#[path = "../tests/common/mod.rs"]
mod common;

use common::generators::{formula_to_string, random_formulas};
use common::parser::parse_complete;

/// `p & q & p & q & ...` with `k` atoms : flat left-associative folding
fn flat_conjunction(k: usize) -> String {
    (0..k)
        .map(|i| if i % 2 == 0 { "p" } else { "q" })
        .collect::<Vec<_>>()
        .join(" & ")
}

/// `((((...p...))))` : parenthesis recursion of the given depth
fn nested_parens(depth: usize) -> String {
    format!("{}p{}", "(".repeat(depth), ")".repeat(depth))
}

/// `AG EF AX ! AG EF ... p` : prefix-operator recursion
fn prefix_chain(depth: usize) -> String {
    let ops = ["AG", "EF", "AX", "!"];
    let mut input = String::new();
    for i in 0..depth {
        input.push_str(ops[i % ops.len()]);
        input.push(' ');
    }
    input.push('p');
    input
}

fn bench_shape(c: &mut Criterion, group_name: &str, shape: impl Fn(usize) -> String) {
    let mut group = c.benchmark_group(group_name);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));
    for size in [16usize, 64, 256] {
        let input = shape(size);
        parse_complete(&input).expect("benchmark input must be a valid formula");
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &input, |b, input| {
            b.iter(|| black_box(parse_complete(black_box(input))));
        });
    }
    group.finish();
}

fn parser_synthetic_shapes(c: &mut Criterion) {
    bench_shape(c, "parser/flat_conjunction", flat_conjunction);
    bench_shape(c, "parser/nested_parens", nested_parens);
    bench_shape(c, "parser/prefix_chain", prefix_chain);
}

fn parser_random_formulas(c: &mut Criterion) {
    // 10 seeded random formulae of depth <= 8, printed with minimal
    // parenthesization, all parsed per iteration
    let inputs: Vec<String> = random_formulas(0x9A55E, 10, 8)
        .iter()
        .map(formula_to_string)
        .collect();
    for input in &inputs {
        parse_complete(input).expect("printed formulae must round-trip");
    }
    let total_bytes: u64 = inputs.iter().map(|input| input.len() as u64).sum();
    let mut group = c.benchmark_group("parser/random_depth8");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));
    group.throughput(Throughput::Bytes(total_bytes));
    group.bench_function("10_formulas", |b| {
        b.iter(|| {
            for input in &inputs {
                black_box(parse_complete(black_box(input))).ok();
            }
        });
    });
    group.finish();
}

criterion_group!(benches, parser_synthetic_shapes, parser_random_formulas);
criterion_main!(benches);
