//! Benchmarks for the core `anyhow::Error` operations: construction,
//! context wrapping, downcasting, source-chain traversal and formatting.

use anyhow::{anyhow, bail, ensure, Context, Error, Result};
use divan::{black_box, Bencher};
use std::fmt::Write as _;
use std::io;

fn main() {
    divan::main();
}

#[derive(thiserror::Error, Debug)]
#[error("custom error with code {code}")]
struct CustomError {
    code: u32,
}

fn io_error() -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, "file not found")
}

/// Builds an error with `depth` layers of context on top of an io::Error.
fn deep_error(depth: usize) -> Error {
    let mut err = Error::new(io_error());
    for i in 0..depth {
        err = err.context(format!("layer {i}"));
    }
    err
}

mod construct {
    use super::*;

    #[divan::bench]
    fn msg_static_str() -> Error {
        anyhow!(black_box("static message"))
    }

    #[divan::bench]
    fn msg_formatted() -> Error {
        let code = black_box(42);
        anyhow!("failed with code {code}")
    }

    #[divan::bench]
    fn from_std_error() -> Error {
        Error::new(black_box(io_error()))
    }

    #[divan::bench]
    fn from_custom_error() -> Error {
        Error::from(black_box(CustomError { code: 7 }))
    }

    #[divan::bench]
    fn question_mark_conversion() -> Result<()> {
        fn inner() -> Result<()> {
            Err(io_error())?;
            Ok(())
        }
        black_box(inner())
    }

    #[divan::bench]
    fn bail_macro() -> Result<()> {
        fn inner(code: u32) -> Result<()> {
            bail!("bailed with code {code}");
        }
        inner(black_box(3))
    }

    #[divan::bench]
    fn ensure_failing() -> Result<()> {
        fn inner(a: u32, b: u32) -> Result<()> {
            ensure!(a == b, "values differ");
            Ok(())
        }
        inner(black_box(1), black_box(2))
    }

    #[divan::bench]
    fn ensure_passing() -> Result<()> {
        fn inner(a: u32, b: u32) -> Result<()> {
            ensure!(a == b, "values differ");
            Ok(())
        }
        inner(black_box(1), black_box(1))
    }
}

mod context {
    use super::*;

    #[divan::bench]
    fn context_on_result() -> Result<()> {
        let res: std::result::Result<(), io::Error> = Err(black_box(io_error()));
        res.context("failed to read config")
    }

    #[divan::bench]
    fn with_context_on_result() -> Result<()> {
        let res: std::result::Result<(), io::Error> = Err(black_box(io_error()));
        let path = black_box("/etc/app/config.toml");
        res.with_context(|| format!("failed to read {path}"))
    }

    #[divan::bench]
    fn context_on_option() -> Result<u32> {
        black_box(None::<u32>).context("value missing")
    }

    #[divan::bench]
    fn with_context_ok_path() -> Result<u32> {
        let res: std::result::Result<u32, io::Error> = Ok(black_box(1));
        res.with_context(|| format!("never evaluated {}", 1))
    }

    #[divan::bench(args = [1, 8, 32])]
    fn nested_layers(depth: usize) -> Error {
        deep_error(black_box(depth))
    }
}

mod downcast {
    use super::*;

    #[divan::bench]
    fn downcast_ref_hit(bencher: Bencher) {
        let err = Error::new(CustomError { code: 1 });
        bencher.bench_local(|| black_box(&err).downcast_ref::<CustomError>().is_some());
    }

    #[divan::bench]
    fn downcast_ref_miss(bencher: Bencher) {
        let err = Error::new(CustomError { code: 1 });
        bencher.bench_local(|| black_box(&err).downcast_ref::<io::Error>().is_some());
    }

    #[divan::bench]
    fn downcast_ref_through_context(bencher: Bencher) {
        let err = Error::new(io_error()).context("outer context");
        bencher.bench_local(|| black_box(&err).downcast_ref::<io::Error>().is_some());
    }

    #[divan::bench]
    fn downcast_owned(bencher: Bencher) {
        bencher
            .with_inputs(|| Error::new(CustomError { code: 1 }))
            .bench_local_values(|err| err.downcast::<CustomError>());
    }
}

mod chain {
    use super::*;

    #[divan::bench(args = [1, 8, 32])]
    fn iterate(bencher: Bencher, depth: usize) {
        let err = deep_error(depth);
        bencher.bench_local(|| black_box(&err).chain().count());
    }

    #[divan::bench(args = [1, 8, 32])]
    fn root_cause(bencher: Bencher, depth: usize) {
        let err = deep_error(depth);
        bencher.bench_local(|| black_box(&err).root_cause().to_string().len());
    }
}

mod format {
    use super::*;

    fn render(bencher: Bencher, depth: usize, f: fn(&mut String, &Error)) {
        let err = deep_error(depth);
        let mut buf = String::with_capacity(4096);
        bencher.bench_local(|| {
            buf.clear();
            f(&mut buf, black_box(&err));
            buf.len()
        });
    }

    #[divan::bench(args = [1, 8])]
    fn display(bencher: Bencher, depth: usize) {
        render(bencher, depth, |buf, err| write!(buf, "{err}").unwrap());
    }

    #[divan::bench(args = [1, 8])]
    fn display_alternate(bencher: Bencher, depth: usize) {
        render(bencher, depth, |buf, err| write!(buf, "{err:#}").unwrap());
    }

    #[divan::bench(args = [1, 8])]
    fn debug(bencher: Bencher, depth: usize) {
        render(bencher, depth, |buf, err| write!(buf, "{err:?}").unwrap());
    }

    #[divan::bench(args = [1, 8])]
    fn debug_alternate(bencher: Bencher, depth: usize) {
        render(bencher, depth, |buf, err| write!(buf, "{err:#?}").unwrap());
    }
}
