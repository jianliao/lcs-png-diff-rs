use std::time::Duration;

use base64::encode;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use image::{DynamicImage, GenericImageView};
use lcs_png_diff::{create_table, diff};

fn create_lcs_table(c: &mut Criterion) {
    let old_5_x_5 = [1, 2, 3, 4, 5];
    let new_5_x_5 = [1, 2, 3, 4, 5];
    c.bench_function("create_lcs_table 5 x 5", |b| {
        b.iter(|| create_table(black_box(&old_5_x_5), black_box(&new_5_x_5)))
    });

    let old_50_x_50 = [
        85, 46, 73, 72, 87, 39, 68, 83, 57, 80, 58, 75, 26, 33, 91, 51, 14, 9, 29, 75, 35, 31, 80,
        69, 85, 89, 28, 70, 42, 23, 75, 70, 82, 27, 25, 94, 7, 47, 29, 7, 12, 28, 24, 79, 35, 68,
        91, 91, 12, 2,
    ];

    let new_50_x_50 = [
        84, 35, 10, 59, 70, 34, 91, 79, 24, 70, 45, 24, 57, 42, 47, 26, 6, 16, 14, 91, 72, 42, 29,
        7, 24, 94, 24, 30, 79, 44, 36, 86, 21, 34, 90, 9, 95, 8, 24, 87, 70, 96, 30, 95, 13, 24,
        38, 57, 51, 79,
    ];
    c.bench_function("create_lcs_table 50 x 50", |b| {
        b.iter(|| create_table(black_box(&old_50_x_50), black_box(&new_50_x_50)))
    });
}

fn long_string_lcs_table(c: &mut Criterion) {
    let before_png_1100k: DynamicImage = image::open("tests/fixtures/slider_before.png").unwrap();
    let after_png_1100k: DynamicImage = image::open("tests/fixtures/slider_after.png").unwrap();
    let after_w = after_png_1100k.dimensions().0;
    let before_w = before_png_1100k.dimensions().0;
    let before_encoded_png: Vec<String> = before_png_1100k
        .as_bytes()
        .to_vec()
        .chunks(before_w as usize * 4)
        .map(encode)
        .collect();
    let after_encoded_png: Vec<String> = after_png_1100k
        .as_bytes()
        .to_vec()
        .chunks(after_w as usize * 4)
        .map(encode)
        .collect();

    let mut group = c.benchmark_group("long_string_lcs_table");

    group
        .measurement_time(Duration::new(500, 0))
        .sample_size(10);
    group.bench_function("create_cls_table", |b| {
        b.iter(|| {
            create_table(
                black_box(&before_encoded_png),
                black_box(&after_encoded_png),
            )
        })
    });
    group.finish();
}

fn small_png_diff(c: &mut Criterion) {
    let before_png_118k: DynamicImage =
        image::open("tests/fixtures/backstopjs_pricing_before.png").unwrap();
    let after_png_118k: DynamicImage =
        image::open("tests/fixtures/backstopjs_pricing_after.png").unwrap();
    c.bench_function("png diff 118k", |b| {
        b.iter(|| diff(black_box(&before_png_118k), black_box(&after_png_118k)))
    });
}

fn large_png_diff(c: &mut Criterion) {
    let before_png_1100k: DynamicImage = image::open("tests/fixtures/slider_before.png").unwrap();
    let after_png_1100k: DynamicImage = image::open("tests/fixtures/slider_after.png").unwrap();
    let mut group = c.benchmark_group("large_png_diff");
    group
        .measurement_time(Duration::new(180, 0))
        .sample_size(10);
    group.bench_function("png diff 1100k", |b| {
        b.iter(|| diff(black_box(&before_png_1100k), black_box(&after_png_1100k)))
    });
    group.finish();
}

criterion_group!(
    benches,
    create_lcs_table,
    // long_string_lcs_table,
    small_png_diff,
    large_png_diff
);
criterion_main!(benches);
