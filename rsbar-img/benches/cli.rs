use std::process::Command;

use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use criterion::{criterion_group, criterion_main, Criterion};

const SINGLE_IMAGE_SMALL: &[u8; 210] = include_bytes!("images/qr-code.png");
const NO_CODE_IMAGE_SMALL: &[u8; 7201] = include_bytes!("images/no-code.png");
const MULTIPLE_IMAGE_SMALL: &[u8; 14738] = include_bytes!("images/multiple.png");

const SINGLE_IMAGE_LARGE: &[u8; 22000] = include_bytes!("images/qr-code-large.png");
const NO_CODE_IMAGE_LARGE: &[u8; 7855] = include_bytes!("images/no-code-large.png");
const MULTIPLE_IMAGE_LARGE: &[u8; 16419] = include_bytes!("images/multiple-large.png");

fn rsbarimg_small_benchmark(c: &mut Criterion) {
    let single_file_small = assert_fs::NamedTempFile::new("single.png").unwrap();
    let no_code_file_small = assert_fs::NamedTempFile::new("nocode.png").unwrap();
    let multiple_file_small = assert_fs::NamedTempFile::new("multiple.png").unwrap();

    single_file_small.write_binary(SINGLE_IMAGE_SMALL).unwrap();
    no_code_file_small
        .write_binary(NO_CODE_IMAGE_SMALL)
        .unwrap();
    multiple_file_small
        .write_binary(MULTIPLE_IMAGE_SMALL)
        .unwrap();

    let mut group = c.benchmark_group("small files");

    group.bench_function("single code", |b| {
        b.iter(|| {
            let mut cmd = Command::cargo_bin("rsbar-img").unwrap();

            cmd.arg(single_file_small.path()).unwrap();
        })
    });

    group.bench_function("no code", |b| {
        b.iter(|| {
            let mut cmd = Command::cargo_bin("rsbar-img").unwrap();

            cmd.arg(no_code_file_small.path()).assert().failure();
        })
    });

    group.bench_function("multiple codes", |b| {
        b.iter(|| {
            let mut cmd = Command::cargo_bin("rsbar-img").unwrap();

            cmd.arg(multiple_file_small.path()).unwrap();
        })
    });

    group.finish();
}

fn rsbarimg_large_benchmark(c: &mut Criterion) {
    let single_file_large = assert_fs::NamedTempFile::new("single-large.png").unwrap();
    let no_code_file_large = assert_fs::NamedTempFile::new("nocode-large.png").unwrap();
    let multiple_file_large = assert_fs::NamedTempFile::new("multiple-large.png").unwrap();

    single_file_large.write_binary(SINGLE_IMAGE_LARGE).unwrap();
    no_code_file_large
        .write_binary(NO_CODE_IMAGE_LARGE)
        .unwrap();
    multiple_file_large
        .write_binary(MULTIPLE_IMAGE_LARGE)
        .unwrap();

    let mut group = c.benchmark_group("large files");

    group.bench_function("single code", |b| {
        b.iter(|| {
            let mut cmd = Command::cargo_bin("rsbar-img").unwrap();

            cmd.arg(single_file_large.path()).unwrap();
        })
    });

    group.bench_function("no code", |b| {
        b.iter(|| {
            let mut cmd = Command::cargo_bin("rsbar-img").unwrap();

            cmd.arg(no_code_file_large.path()).assert().failure();
        })
    });

    group.bench_function("multiple codes", |b| {
        b.iter(|| {
            let mut cmd = Command::cargo_bin("rsbar-img").unwrap();

            cmd.arg(multiple_file_large.path()).unwrap();
        })
    });

    group.finish();
}

criterion_group!(benches, rsbarimg_small_benchmark, rsbarimg_large_benchmark);
criterion_main!(benches);
