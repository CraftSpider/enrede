use criterion::{criterion_group, criterion_main, Criterion};
use enrede::encoding::{Ascii, Utf16};
use enrede::Encoding;
use rand::{thread_rng, Rng};
use std::hint::black_box;

mod utils;

pub fn validate_ascii(c: &mut Criterion) {
    let mut data: Vec<u8> = Vec::new();
    for _ in 0..1000 {
        data.push(thread_rng().gen_range(0..128))
    }

    c.bench_function("Ascii::validate", |b| {
        b.iter(|| Ascii::validate(black_box(&data)).unwrap())
    });
}

pub fn validate_utf16(c: &mut Criterion) {
    let mut data: Vec<u8> = Vec::new();
    for _ in 0..1000 {
        data.extend(Utf16::encode_char(rand::random::<char>()).unwrap());
    }

    c.bench_function("Utf16::validate", |b| {
        b.iter(|| Utf16::validate(black_box(&data)).unwrap())
    });
}

criterion_group!(name = ascii; config = utils::criterion(); targets = validate_ascii);
criterion_group!(name = utf16; config = utils::criterion(); targets = validate_utf16);
criterion_main!(ascii, utf16);
