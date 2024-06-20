use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use enrede::encoding::{Ascii, Utf16};
use enrede::{Encoding, String};
use rand::{thread_rng, Rng};
use std::hint::black_box;

mod utils;

pub fn validate_ascii(c: &mut Criterion) {
    c.bench_function("Ascii::validate", |b| {
        b.iter_batched_ref(
            || {
                let mut data: Vec<u8> = Vec::new();
                for _ in 0..1024 {
                    data.push(thread_rng().gen_range(0..128))
                }
                data
            },
            |data| Ascii::validate(black_box(data)).unwrap(),
            BatchSize::SmallInput,
        )
    });
}

pub fn encode_ascii(c: &mut Criterion) {
    let mut rng = thread_rng();
    c.bench_function("Ascii::encode", |b| {
        b.iter_batched(
            || char::from(rng.gen_range(0..128)),
            |char| Ascii::encode(black_box(char), black_box(&mut [0])).unwrap(),
            BatchSize::SmallInput,
        )
    });
}

pub fn decode_ascii(c: &mut Criterion) {
    let mut rng = thread_rng();
    c.bench_function("Ascii::decode", |b| {
        b.iter_batched_ref(
            || {
                let mut str = String::<Ascii>::new();
                str.push(char::from(rng.gen_range(0u8..128)));
                str
            },
            |str: &mut String<_>| {
                Ascii::decode_char(black_box(str));
            },
            BatchSize::SmallInput,
        )
    });
}

pub fn validate_utf16(c: &mut Criterion) {
    c.bench_function("Utf16::validate", |b| {
        b.iter_batched_ref(
            || {
                let mut data: Vec<u8> = Vec::new();
                while data.len() < 1024 {
                    data.extend(Utf16::encode_char(rand::random::<char>()).unwrap());
                }
                data
            },
            |data| Utf16::validate(black_box(data)).unwrap(),
            BatchSize::SmallInput,
        )
    });
}

pub fn encode_utf16(c: &mut Criterion) {
    let mut rng = thread_rng();
    c.bench_function("Utf16::encode", |b| {
        b.iter_batched(
            || rng.gen::<char>(),
            |char| Utf16::encode(black_box(char), black_box(&mut [0, 0, 0, 0])).unwrap(),
            BatchSize::SmallInput,
        )
    });
}

pub fn decode_utf16(c: &mut Criterion) {
    let mut rng = thread_rng();
    c.bench_function("Utf16::decode", |b| {
        b.iter_batched_ref(
            || {
                let mut str = String::<Utf16>::new();
                str.push(rng.gen::<char>());
                str
            },
            |str: &mut String<_>| {
                Utf16::decode_char(black_box(str));
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(name = validate; config = utils::criterion(); targets = validate_ascii, validate_utf16);
criterion_group!(name = encode; config = utils::criterion(); targets = encode_ascii, encode_utf16);
criterion_group!(name = decode; config = utils::criterion(); targets = decode_ascii, decode_utf16);
criterion_main!(validate, encode, decode);
