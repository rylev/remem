#![feature(test)]

extern crate test;

const CAPACITY: usize = 4_000; // 4kb

mod remem {
    use super::CAPACITY;
    use remem::Pool;
    use std::thread;
    use test::Bencher;

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
        let p = Pool::<Vec<usize>>::new(|| vec![0; CAPACITY]);
        let mut threads = Vec::new();

        for _ in 0..thread {
            let p = p.clone();
            threads.push(thread::spawn(move || {
                for _ in 0..iter {
                    let mut v = p.get();
                    v[0] = 1;
                    v[CAPACITY / 4] = 1;
                    v[CAPACITY / 2] = 1;
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
    use test::Bencher;

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
                    let mut v: Vec<usize> = vec![0; CAPACITY];
                    v[0] = 1;
                    v[CAPACITY / 4] = 1;
                    v[CAPACITY / 2] = 1;
                }
            }));
        }

        for t in threads {
            t.join().unwrap();
        }
    }
}
