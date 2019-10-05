#![feature(test)]

extern crate test;

use growable::{GrowablePoolBuilder, Reusable};
use std::{collections::VecDeque, fmt::Debug};
use test::Bencher;

#[bench]
fn bench_box(bencher: &mut Bencher) {
    let mut buffer: VecDeque<Box<dyn Debug>> = VecDeque::with_capacity(1024);
    bencher.iter(|| {
        for i in 0 .. 1024 {
            let item: Box<dyn Debug> = match i % 3 {
                0 => Box::new("Hello World"),
                1 => Box::new(365),
                2 => Box::new(['?'; 24]),
                _ => unreachable!(),
            };
            buffer.push_back(item);
        }
        for _ in 0 .. 1024 {
            let _ = buffer.pop_front().unwrap();
        }
    });
}

#[bench]
fn bench_growable(bencher: &mut Bencher) {
    let mut buffer: VecDeque<Reusable<dyn Debug>> = VecDeque::with_capacity(1024);
    let mut pool = GrowablePoolBuilder::default()
        .with_default_capacity(24)
        .with_default_ptr_alignment(8)
        .with_capacity(1024)
        .build();
    bencher.iter(|| {
        for i in 0 .. 1024 {
            let item: Reusable<dyn Debug> = match i % 3 {
                0 => pool.allocate("Hello World"),
                1 => pool.allocate(365),
                2 => pool.allocate(['?'; 24]),
                _ => unreachable!(),
            };
            buffer.push_back(item);
        }
        for _ in 0 .. 1024 {
            pool.free(buffer.pop_front().unwrap());
        }
    });
}
