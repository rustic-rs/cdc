use criterion::{criterion_group, criterion_main, Criterion};
use rustic_cdc::{Rabin64, RollingHash64};

pub fn slide_benchmarks(c: &mut Criterion) {
    for i in [1_000, 10_000, 100_000] {
        c.bench_function(&format!("slide {}x", i), |b| {
            let data: u8 = 16; //arbitrary value
            b.iter(|| {
                let mut rabin = Rabin64::new(5);
                for _ in 0..i {
                    rabin.slide(data)
                }
            })
        });
    }
}

criterion_group!(benches, slide_benchmarks);
criterion_main!(benches);
