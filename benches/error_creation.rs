use std::io;

fn main() {
    divan::main();
}

#[divan::bench]
fn error_from_message() -> anyhow::Error {
    anyhow::anyhow!("something went wrong")
}

#[divan::bench]
fn error_from_formatted_message() -> anyhow::Error {
    let value = 42;
    anyhow::anyhow!("error with value: {}", value)
}

#[divan::bench]
fn error_from_std_error() -> anyhow::Error {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    anyhow::Error::new(io_error)
}

#[divan::bench]
fn error_with_context() -> anyhow::Result<()> {
    use anyhow::Context;
    let result: Result<(), io::Error> = Err(io::Error::new(io::ErrorKind::NotFound, "file not found"));
    result.context("Failed to read configuration file")
}

#[divan::bench]
fn error_with_lazy_context() -> anyhow::Result<()> {
    use anyhow::Context;
    let result: Result<(), io::Error> = Err(io::Error::new(io::ErrorKind::NotFound, "file not found"));
    let path = "/path/to/config.json";
    result.with_context(|| format!("Failed to read file: {}", path))
}

#[divan::bench]
fn error_downcast() -> bool {
    let error = anyhow::Error::new(io::Error::new(io::ErrorKind::NotFound, "not found"));
    error.downcast_ref::<io::Error>().is_some()
}

#[divan::bench]
fn error_chain_iteration(bencher: divan::Bencher) {
    use anyhow::Context;
    let base_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let error = anyhow::Error::new(base_error)
        .context("Failed to read config")
        .err()
        .unwrap();
    
    bencher.bench(|| {
        error.chain().count()
    });
}

#[divan::bench]
fn error_display() -> String {
    let error = anyhow::anyhow!("test error with {} context", "formatted");
    format!("{}", error)
}

#[divan::bench]
fn error_debug() -> String {
    let error = anyhow::anyhow!("test error with {} context", "formatted");
    format!("{:?}", error)
}
