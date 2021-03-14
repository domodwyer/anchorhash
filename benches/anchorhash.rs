use anchorhash::AnchorHash;
use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use fnv::FnvBuildHasher;

fn bench(c: &mut Criterion) {
    const MAX_CAPACITY: u16 = 10_000;
    const WORKING_SET_SIZES: &[usize] = &[3, 100, 1_000];

    let mut group = c.benchmark_group("AnchorHash");

    for &size in WORKING_SET_SIZES {
        for &capacity in &[size as u16 + 1, MAX_CAPACITY] {
            {
                let input = new(size, capacity);

                group.bench_with_input(
                    BenchmarkId::new(
                        "lookup",
                        format!("capacity={}/resources={}", capacity, size),
                    ),
                    &input,
                    |b, a| b.iter(|| black_box(a.get_resource("k"))),
                );
            }

            {
                let input = new(size, capacity);
                group.bench_function(
                    BenchmarkId::new("add", format!("capacity={}/resources={}", capacity, size)),
                    move |b| {
                        b.iter_batched(
                            || input.clone(),
                            |mut a| a.add_resource(size + 1).unwrap(),
                            BatchSize::SmallInput,
                        )
                    },
                );
            }

            // NOTE: these remove benchmarks use small, simple resources that
            // fairly cache friendly.
            //
            // More complex resource types (like connection pools / structs /
            // etc) may have slower runtime.

            {
                let input = new(size, capacity);
                group.bench_function(
                    BenchmarkId::new(
                        "remove_first",
                        format!("capacity={}/resources={}", capacity, size),
                    ),
                    move |b| {
                        b.iter_batched(
                            || input.clone(),
                            |mut a| a.remove_resource(&0).unwrap(),
                            BatchSize::NumIterations(1),
                        )
                    },
                );
            }

            {
                let input = new(size, capacity);
                group.bench_function(
                    BenchmarkId::new(
                        "remove_last",
                        format!("capacity={}/resources={}", capacity, size),
                    ),
                    move |b| {
                        b.iter_batched(
                            || input.clone(),
                            |mut a| a.remove_resource(&(size - 1)).unwrap(),
                            BatchSize::NumIterations(1),
                        )
                    },
                );
            }
        }
    }
}

fn new(size: usize, capacity: u16) -> AnchorHash<&'static str, usize, FnvBuildHasher> {
    anchorhash::Builder::with_hasher(FnvBuildHasher::default())
        .with_resources(0..size)
        .build(capacity)
}

criterion_group!(benches, bench);
criterion_main!(benches);
