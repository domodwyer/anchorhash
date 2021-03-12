use anchorhash::range_map;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("range_map");

    for v in &[42, 200] {
        for max in &[50, 128] {
            let input = (v, max);
            group.bench_with_input(
                BenchmarkId::new("map", format!("v={} max={}", v, max)),
                &input,
                |b, (&v, &max)| b.iter(|| range_map(black_box(v), max)),
            );
        }
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
