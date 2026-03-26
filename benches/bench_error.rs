use criterion::{black_box, criterion_group, criterion_main, Criterion};

use anyhow::{anyhow, Context, Error, Result};
use std::fmt;
use std::io;

// ---------------------------------------------------------------------------
// Helper types
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct CustomError {
    code: u32,
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "custom error (code {})", self.code)
    }
}

impl std::error::Error for CustomError {}

// ---------------------------------------------------------------------------
// Error creation benchmarks
// ---------------------------------------------------------------------------

fn bench_error_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_creation");

    group.bench_function("from_std_error", |b| {
        b.iter(|| {
            let err = io::Error::new(io::ErrorKind::NotFound, "file not found");
            black_box(Error::new(err));
        });
    });

    group.bench_function("from_message_static", |b| {
        b.iter(|| {
            black_box(Error::msg("something went wrong"));
        });
    });

    group.bench_function("from_message_formatted", |b| {
        b.iter(|| {
            black_box(anyhow!("operation {} failed at step {}", "upload", 3));
        });
    });

    group.bench_function("from_custom_error", |b| {
        b.iter(|| {
            let err = CustomError { code: 42 };
            black_box(Error::new(err));
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Context benchmarks
// ---------------------------------------------------------------------------

fn bench_context(c: &mut Criterion) {
    let mut group = c.benchmark_group("context");

    group.bench_function("context_on_result_err", |b| {
        b.iter(|| {
            let result: Result<(), _> = Err(io::Error::new(io::ErrorKind::Other, "base error"));
            let _ = black_box(result.context("additional context"));
        });
    });

    group.bench_function("context_on_result_ok", |b| {
        b.iter(|| {
            let result: Result<u32, io::Error> = Ok(42);
            black_box(result.context("additional context").unwrap());
        });
    });

    group.bench_function("with_context_on_result_err", |b| {
        b.iter(|| {
            let result: Result<(), _> = Err(io::Error::new(io::ErrorKind::Other, "base error"));
            let _ = black_box(result.with_context(|| format!("failed at step {}", 5)));
        });
    });

    group.bench_function("nested_context_3_levels", |b| {
        b.iter(|| {
            let err: Result<()> = Err(anyhow!("root cause"));
            let err = err.context("level 1");
            let err = err.context("level 2");
            let _ = black_box(err.context("level 3"));
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Downcast benchmarks
// ---------------------------------------------------------------------------

fn bench_downcast(c: &mut Criterion) {
    let mut group = c.benchmark_group("downcast");

    group.bench_function("downcast_ref_hit", |b| {
        let err = Error::new(CustomError { code: 42 });
        b.iter(|| {
            black_box(err.downcast_ref::<CustomError>());
        });
    });

    group.bench_function("downcast_ref_miss", |b| {
        let err = Error::new(io::Error::new(io::ErrorKind::Other, "io error"));
        b.iter(|| {
            black_box(err.downcast_ref::<CustomError>());
        });
    });

    group.bench_function("downcast_ref_through_context", |b| {
        let err = Error::new(CustomError { code: 42 }).context("wrapped");
        b.iter(|| {
            black_box(err.downcast_ref::<CustomError>());
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Display / formatting benchmarks
// ---------------------------------------------------------------------------

fn bench_display(c: &mut Criterion) {
    let mut group = c.benchmark_group("display");

    group.bench_function("display_simple", |b| {
        let err = anyhow!("something went wrong");
        b.iter(|| {
            black_box(format!("{}", err));
        });
    });

    group.bench_function("display_alternate_with_chain", |b| {
        let err: Result<()> = Err(anyhow!("root cause"));
        let err = err
            .context("middle context")
            .context("top context")
            .unwrap_err();
        b.iter(|| {
            black_box(format!("{:#}", err));
        });
    });

    group.bench_function("debug_with_chain", |b| {
        let err: Result<()> = Err(anyhow!("root cause"));
        let err = err
            .context("middle context")
            .context("top context")
            .unwrap_err();
        b.iter(|| {
            black_box(format!("{:?}", err));
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Chain iteration benchmarks
// ---------------------------------------------------------------------------

fn bench_chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("chain");

    group.bench_function("chain_length_1", |b| {
        let err = anyhow!("single error");
        b.iter(|| {
            black_box(err.chain().count());
        });
    });

    group.bench_function("chain_length_5", |b| {
        let err: Result<()> = Err(anyhow!("root"));
        let err = err
            .context("ctx 1")
            .context("ctx 2")
            .context("ctx 3")
            .context("ctx 4")
            .unwrap_err();
        b.iter(|| {
            black_box(err.chain().count());
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_error_creation,
    bench_context,
    bench_downcast,
    bench_display,
    bench_chain,
);
criterion_main!(benches);
