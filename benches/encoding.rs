use byte_unit::{Byte, Unit};
use core::hint::black_box;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use enrede::encoding::{
    ArrayLike, Ascii, ExtendedAscii, Iso8859_15, Iso8859_2, JisX0201, JisX0208, Utf16BE, Utf16LE,
    Utf32, Win1251, Win1252, Win1252Loose,
};
use enrede::{Encoding, String};
use rand::distributions::Distribution;
use rand::{thread_rng, Rng};

mod utils;

const KILOBYTE: Byte = match Byte::from_u64_with_unit(1, Unit::KiB) {
    Some(b) => b,
    None => panic!(),
};
const MEGABYTE: Byte = match Byte::from_u64_with_unit(1, Unit::MiB) {
    Some(b) => b,
    None => panic!(),
};

fn bench_validate<E: Encoding + Distribution<char>>(c: &mut Criterion, bytes: Byte) {
    let mut rng = thread_rng();
    c.bench_function(&format!("{}::validate ({})", E::shorthand(), bytes), |b| {
        b.iter_batched_ref(
            || {
                let mut data: Vec<u8> = Vec::new();
                while (data.len() as u64) < bytes.as_u64() {
                    let char = rng.sample(E::default());
                    let bytes = E::encode_char(char).unwrap();
                    data.extend(bytes.slice());
                }
                data
            },
            |data| E::validate(black_box(data)).unwrap(),
            BatchSize::SmallInput,
        )
    });
}

fn bench_encode<E: Encoding + Distribution<char>>(c: &mut Criterion) {
    let mut rng = thread_rng();
    c.bench_function(&format!("{}::encode", E::shorthand()), |b| {
        b.iter_batched(
            || rng.sample(E::default()),
            |char| E::encode(black_box(char), black_box(&mut [0, 0, 0, 0])).unwrap(),
            BatchSize::SmallInput,
        )
    });
}

fn bench_decode<E: Encoding + Distribution<char>>(c: &mut Criterion) {
    let mut rng = thread_rng();
    c.bench_function(&format!("{}::decode", E::shorthand()), |b| {
        b.iter_batched_ref(
            || {
                let mut str = String::<E>::new();
                str.push(rng.sample(E::default()));
                str
            },
            |str: &mut String<_>| {
                E::decode_char(black_box(str));
            },
            BatchSize::SmallInput,
        )
    });
}

pub fn bench_encoding<E: Encoding + Distribution<char>>(c: &mut Criterion) {
    bench_validate::<E>(c, KILOBYTE);
    bench_validate::<E>(c, MEGABYTE);
    bench_encode::<E>(c);
    bench_decode::<E>(c);
}

pub fn bench_all(c: &mut Criterion) {
    bench_encoding::<Ascii>(c);
    bench_encoding::<ExtendedAscii>(c);

    bench_encoding::<Utf16LE>(c);
    bench_encoding::<Utf16BE>(c);
    bench_encoding::<Utf32>(c);

    bench_encoding::<Win1251>(c);
    bench_encoding::<Win1252>(c);
    bench_encoding::<Win1252Loose>(c);

    bench_encoding::<Iso8859_2>(c);
    bench_encoding::<Iso8859_15>(c);

    bench_encoding::<JisX0201>(c);
    // bench_encoding::<JisX0208>(c);
}

criterion_group!(name = benches; config = utils::criterion(); targets = bench_all);
criterion_main!(benches);
