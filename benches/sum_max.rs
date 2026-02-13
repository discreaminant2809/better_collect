use std::{hint::black_box, time::Duration};

use better_collect::{cmp::Max, iter::Fold, prelude::*};
use criterion::{Criterion, criterion_group, criterion_main};
use rand::{RngExt, SeedableRng, rngs::StdRng};

fn sum_max(criterion: &mut Criterion) {
    let seed = 0;
    let mut rng = StdRng::seed_from_u64(seed);

    let nums: Box<_> = std::iter::repeat_with(|| rng.random_range(-10_000..=10_000))
        .take(500_000)
        .collect();

    println!("Seed: {seed}");
    println!("First 10 elements: {:?}", &nums[..10]);

    let mut group = criterion.benchmark_group("sum_max");

    group.bench_function("with_initial", |bencher| {
        // bencher.iter(|| black_box(fold_w_initial(&nums)));
        bencher.iter(|| black_box(bc_tee_with_fold(&nums)));
    });

    group.bench_function("without_initial", |bencher| {
        // bencher.iter(|| black_box(fold_wo_initial(&nums)));
        bencher.iter(|| black_box(bc_tee_with_max(&nums)));
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(5))
        .measurement_time(Duration::from_secs(15))
        .sample_size(300);
    targets = sum_max
}
criterion_main!(benches);

#[allow(unused)]
fn fold_w_initial(nums: &[i32]) -> (i32, i32) {
    nums.iter()
        .copied()
        .fold((0, i32::MIN), |(sum, max), num| (sum + num, max.max(num)))
}

#[allow(unused)]
fn fold_wo_initial(nums: &[i32]) -> (i32, Option<i32>) {
    nums.iter()
        .copied()
        .fold((0, None), |(sum, max), num| (sum + num, max.max(Some(num))))
}

#[allow(unused)]
fn bc_tee_with_max(nums: &[i32]) -> (i32, Option<i32>) {
    nums.iter()
        .copied()
        .feed_into(i32::adding().tee(Max::new()))
}

#[allow(unused)]
fn bc_tee_with_fold(nums: &[i32]) -> (i32, i32) {
    nums.iter()
        .copied()
        .feed_into(i32::adding().tee(Fold::new(i32::MIN, |max, num| *max = (*max).max(num))))
}
