use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fnv::FnvBuildHasher;

fn bench(c: &mut Criterion) {
    const MAX_CAPACITY: u16 = 10_000;
    const WORKING_SET_SIZES: &[usize] = &[3, 100, 1_000];

    let mut group = c.benchmark_group("AnchorHash");

    for &size in WORKING_SET_SIZES {
        for &capacity in &[size as u16, MAX_CAPACITY] {
            let input = anchorhash::Builder::with_hasher(FnvBuildHasher::default())
                .with_resources(0..size)
                .build(capacity);

            group.bench_with_input(
                BenchmarkId::new(
                    "lookup",
                    format!("capacity={}/resources={}", capacity, size),
                ),
                &input,
                |b, a| b.iter(|| black_box(a.get_resource("k"))),
            );
        }
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
