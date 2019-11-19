#![feature(test)]

use std::sync::{Arc, Mutex};

use treiber_stack::TreiberStack;

mod treiber_stack;

pub struct Pool<T> {
    internal: Arc<Mutex<InternalPool<T>>>,
}

impl<T> Pool<T> {
    pub fn new<C, D>(cap: usize, create: C, clear: D) -> Pool<T>
    where
        C: Fn() -> T + 'static,
        D: Fn(&mut T) -> () + 'static,
    {
        Pool {
            internal: Arc::new(Mutex::new(InternalPool::new(cap, create, clear))),
        }
    }

    pub fn get<'a>(&'a self) -> ItemGuard<'a, T> {
        let mut pool = self.internal.lock().unwrap();
        // If the pool is empty, we double the capacity and batch allocate
        // empty elements.
        if pool.free.is_empty() {
            let capacity = pool.free.capacity();
            pool.free.reserve(capacity);
            for _ in 0..capacity {
                let item = (*pool.create)();
                pool.free.push(item);
            }
        }

        ItemGuard {
            item: Some(pool.free.pop().unwrap()),
            pool: self,
        }
    }

    pub fn reintroduce(&self, mut item: T) {
        let mut pool = self.internal.lock().unwrap();
        (*pool.clear)(&mut item);
        pool.free.push(item);
    }
}

impl<T> Clone for Pool<T> {
    fn clone(&self) -> Self {
        Pool {
            internal: self.internal.clone(),
        }
    }
}

struct InternalPool<T> {
    free: Vec<T>,
    create: Box<dyn Fn() -> T>,
    clear: Box<dyn Fn(&mut T)>,
}

impl<T> InternalPool<T> {
    pub fn new<C, D>(cap: usize, create: C, clear: D) -> Self
    where
        C: Fn() -> T + 'static,
        D: Fn(&mut T) -> () + 'static,
    {
        let mut free = Vec::with_capacity(cap);
        for _ in 0..cap {
            free.push(create());
        }
        InternalPool {
            free,
            create: Box::new(create),
            clear: Box::new(clear),
        }
    }
}

pub struct ItemGuard<'a, T> {
    item: Option<T>,
    pool: &'a Pool<T>,
}

impl<'a, T> Drop for ItemGuard<'a, T> {
    fn drop(&mut self) {
        self.pool.reintroduce(self.item.take().unwrap())
    }
}

impl<'a, T> std::ops::Deref for ItemGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item.as_ref().unwrap()
    }
}

impl<'a, T> std::ops::DerefMut for ItemGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item.as_mut().unwrap()
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use test::Bencher;

    use super::*;

    #[test]
    fn it_works() {
        let pool = Pool::<Vec<u8>>::new(
            1024,
            || {
                println!("Allocating new memory");
                Vec::new()
            },
            |v| v.clear(),
        );
        let mut item = pool.get();
        let mut _item2 = pool.get();

        item.push(1);
        item.push(2);
        item.push(3);

        drop(item);

        let mut item = pool.get();

        item.push(1);
        item.push(2);
        item.push(3);
    }

    const ORIGINAL_SIZE: usize = 10;
    const ITERATIONS: usize = 10;

    macro_rules! run_benchmark {
        ($create_item:expr) => {{
            for _ in 0..1000 {
                let mut item = $create_item();

                for n in 0..ORIGINAL_SIZE {
                    item.push(n);
                }

                drop(item);

                for n in 0..ITERATIONS {
                    let mut item = $create_item();

                    for n in 0..(ITERATIONS - n) {
                        item.push(n);
                    }
                }
            }
        }};
    }

    #[bench]
    fn bench_remem(b: &mut Bencher) {
        let pool = Pool::<Vec<usize>>::new(1024, || Vec::new(), |v| v.clear());
        b.iter(|| {
            run_benchmark!(|| pool.get());
        });
    }

    #[bench]
    fn bench_vec(b: &mut Bencher) {
        b.iter(|| run_benchmark!(|| Vec::new()));
    }
}
