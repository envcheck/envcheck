use criterion::{black_box, criterion_group, criterion_main, Criterion};
use envcheck::parser::EnvFile;
use envcheck::rules::check_file;
use std::path::PathBuf;

fn bench_parse_env_file(c: &mut Criterion) {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("env")
        .join("valid.env");

    c.bench_function("parse_env_file", |b| {
        b.iter(|| {
            let env_file = EnvFile::parse(black_box(&fixture)).unwrap();
            black_box(env_file)
        });
    });
}

fn bench_lint_rules(c: &mut Criterion) {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("env")
        .join("valid.env");

    let env_file = EnvFile::parse(&fixture).unwrap();

    c.bench_function("lint_rules", |b| {
        b.iter(|| {
            let diagnostics = check_file(black_box(&env_file));
            black_box(diagnostics)
        });
    });
}

fn bench_parallel_vs_sequential(c: &mut Criterion) {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("env");

    // Collect all .env files in the fixture directory
    let files: Vec<PathBuf> = std::fs::read_dir(&fixture_dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "env"))
        .map(|e| e.path())
        .collect();

    let mut group = c.benchmark_group("parallel_vs_sequential");

    group.bench_function("sequential", |b| {
        b.iter(|| {
            for path in &files {
                let env_file = EnvFile::parse(path).unwrap();
                let _ = check_file(&env_file);
            }
        });
    });

    group.bench_function("parallel", |b| {
        use rayon::prelude::*;
        b.iter(|| {
            files.par_iter().for_each(|path| {
                let env_file = EnvFile::parse(path).unwrap();
                let _ = check_file(&env_file);
            });
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parse_env_file,
    bench_lint_rules,
    bench_parallel_vs_sequential
);
criterion_main!(benches);
