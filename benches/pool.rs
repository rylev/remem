#![feature(test)]

extern crate test;

use remem::Pool;
use std::thread;
use test::Bencher;

#[bench]
fn create(b: &mut Bencher) {
    b.iter(|| Pool::<Vec<()>>::new(|| Vec::new(), |v| v.clear()));
}

#[bench]
fn contention(b: &mut Bencher) {
    b.iter(|| run(10, 1000));
}

#[bench]
fn no_contention(b: &mut Bencher) {
    b.iter(|| run(1, 10000));
}

fn run(thread: usize, iter: usize) {
    let p = Pool::<Vec<usize>>::new(|| vec![], |v| v.clear());
    let mut threads = Vec::new();

    for _ in 0..thread {
        let p = p.clone();
        threads.push(thread::spawn(move || {
            for _ in 0..iter {
                let mut v = p.get();
                v.push(1);
            }
        }));
    }

    for t in threads {
        t.join();
    }
}

// #[bench]
// fn bench_remem(b: &mut Bencher) {
//     let pool = Pool::<Vec<usize>>::new(1024, || Vec::new(), |v| v.clear());
//     b.iter(|| {
//         run_benchmark!(|| pool.get());
//     });
// }

// #[bench]
// fn bench_vec(b: &mut Bencher) {
//     b.iter(|| run_benchmark!(|| Vec::new()));
// }
