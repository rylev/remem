#![feature(test)]

extern crate test;

const CAPACITY: usize = 4_000; // 4kb

mod remem {
    use super::CAPACITY;
    use remem::Pool;
    use std::thread;
    use test::{black_box, Bencher};

    #[bench]
    fn create(b: &mut Bencher) {
        b.iter(|| Pool::<Vec<usize>>::new(|| vec![0; CAPACITY]));
    }

    #[bench]
    fn contention(b: &mut Bencher) {
        b.iter(|| run(10, 1000));
    }

    #[bench]
    fn no_contention(b: &mut Bencher) {
        b.iter(|| run(1, 1000));
    }

    fn run(thread: usize, iter: usize) {
        let p = Pool::new(|| vec![0usize; CAPACITY]);
        let mut threads = Vec::new();

        for _ in 0..thread {
            let p = p.clone();
            threads.push(thread::spawn(move || {
                for _ in 0..iter {
                    black_box(p.get());
                }
            }));
        }

        for t in threads {
            t.join().unwrap();
        }
    }
}

mod byte_pool {
    use super::CAPACITY;
    use byte_pool::BytePool;
    use std::thread;
    use test::{black_box, Bencher};

    // #[bench]
    // fn create(b: &mut Bencher) {
    //     b.iter(|| Pool::<Vec<usize>>::new(|| vec![0; CAPACITY]));
    // }

    #[bench]
    fn contention(b: &mut Bencher) {
        b.iter(|| run(10, 1000));
    }

    #[bench]
    fn no_contention(b: &mut Bencher) {
        b.iter(|| run(1, 1000));
    }

    fn run(thread: usize, iter: usize) {
        let p = std::sync::Arc::new(BytePool::new());
        let mut threads = Vec::new();

        for _ in 0..thread {
            let p = p.clone();
            threads.push(thread::spawn(move || {
                for _ in 0..iter {
                    black_box(p.alloc(CAPACITY));
                }
            }));
        }

        for t in threads {
            t.join().unwrap();
        }
    }
}

mod vec {
    use super::CAPACITY;
    use std::thread;
    use test::{black_box, Bencher};

    #[bench]
    fn create(b: &mut Bencher) {
        b.iter(|| vec![0; CAPACITY]);
    }

    #[bench]
    fn contention(b: &mut Bencher) {
        b.iter(|| run(10, 1000));
    }

    #[bench]
    fn no_contention(b: &mut Bencher) {
        b.iter(|| run(1, 1000));
    }

    fn run(thread: usize, iter: usize) {
        let mut threads = Vec::new();

        for _ in 0..thread {
            threads.push(thread::spawn(move || {
                for _ in 0..iter {
                    black_box(vec![0usize; CAPACITY]);
                }
            }));
        }

        for t in threads {
            t.join().unwrap();
        }
    }
}
