use std::path::Path;

use brr_conv_lib::convert;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn conv_benchmark(c: &mut Criterion) {
    c.bench_function("conv test", |b| {
        b.iter(|| convert(black_box(Path::new("benches/test.wav")), false))
    });
}

criterion_group!(benches, conv_benchmark);
criterion_main!(benches);
