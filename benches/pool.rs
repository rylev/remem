#![feature(test)]

extern crate test;

mod remem {
    use remem::Pool;
    use std::thread;
    use test::Bencher;

    #[bench]
    fn create(b: &mut Bencher) {
        b.iter(|| Pool::<Vec<()>>::new(|| Vec::new(), |v| v.clear()));
    }

    #[bench]
    fn contention(b: &mut Bencher) {
        b.iter(|| run(10, 10000));
    }

    #[bench]
    fn no_contention(b: &mut Bencher) {
        b.iter(|| run(1, 10000));
    }

    fn run(thread: usize, iter: usize) {
        let p = Pool::<Vec<usize>>::new(|| Vec::with_capacity(1), |v| v.clear());
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
            t.join().unwrap();
        }
    }
}

mod vec {
    use std::thread;
    use test::Bencher;

    #[bench]
    fn create(b: &mut Bencher) {
        b.iter(|| Vec::<usize>::with_capacity(1));
    }

    #[bench]
    fn contention(b: &mut Bencher) {
        b.iter(|| run(10, 10000));
    }

    #[bench]
    fn no_contention(b: &mut Bencher) {
        b.iter(|| run(1, 10000));
    }

    fn run(thread: usize, iter: usize) {
        let mut threads = Vec::new();

        for _ in 0..thread {
            threads.push(thread::spawn(move || {
                for _ in 0..iter {
                    let mut v: Vec<usize> = Vec::with_capacity(1);
                    v.push(1);
                }
            }));
        }

        for t in threads {
            t.join().unwrap();
        }
    }
}
