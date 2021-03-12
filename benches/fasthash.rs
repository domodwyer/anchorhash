use anchorhash::fasthash;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench(c: &mut Criterion) {
    c.bench_function("fasthash", |b| {
        b.iter(|| {
            fasthash(black_box(42), 0xFEED4242);
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
