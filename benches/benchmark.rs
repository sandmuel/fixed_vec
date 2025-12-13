use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use fixed_vec::FixedVec;
use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Vec push", |b| {
        b.iter_batched(
            || Vec::new(),
            |mut vec| {
                for _ in 0..10_000 {
                    black_box(vec.push(black_box(1)));
                }
            },
            BatchSize::SmallInput,
        );
    });

    c.bench_function("FixedVec push", |b| {
        b.iter_batched(
            || FixedVec::new(10_000),
            |vec| {
                for _ in 0..10_000 {
                    _ = black_box(vec.push(black_box(1)));
                }
            },
            BatchSize::SmallInput,
        );
    });

    c.bench_function("Vec get", |b| {
        b.iter_batched(
            || {
                let mut vec = Vec::new();
                for _ in 0..1 {
                    vec.push(1);
                }
                vec
            },
            |vec| _ = black_box(vec.get(black_box(0))),
            BatchSize::SmallInput,
        );
    });

    c.bench_function("FixedVec get", |b| {
        b.iter_batched(
            || {
                let vec = FixedVec::new(1);
                for _ in 0..1 {
                    _ = vec.push(1);
                }
                vec
            },
            |vec| _ = black_box(vec.get(black_box(0))),
            BatchSize::SmallInput,
        );
    });

    c.bench_function("BoxCar get", |b| {
        b.iter_batched(
            || {
                let vec = boxcar::Vec::new();
                for _ in 0..1 {
                    _ = vec.push(1);
                }
                vec
            },
            |vec| _ = black_box(vec.get(black_box(0))),
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
