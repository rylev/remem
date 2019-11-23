//! # Thread-safe object reuse
//!
//! In many programs the allocator can be a bottleneck, especially when
//! allocating larger structures in a short amount of time. The allocator will
//! have to spend time finding free memory to fit new allocations, and
//! defragmenting memory to free up new space for new ones.
//!
//! However often we'll be allocating in a loop, and right after we drop an
//! object we'll want a new object of the same size. This is where `remem` comes
//! in: it allows you to reuse memory in a thread-safe way.
//!
//! This is useful when writing networked services, performing file reads, or
//! anything else that might allocate a lot. Internally it's implemented using a
//! crossbeam's `SegQueue` which is a really fast algorithm that makes `remem`
//! safe to use between threads!
//!
//! # Example
//!
//! ```rust
//! use remem::Pool;
//! use std::thread;
//!
//! // Create a new Pool instance where new items are initialized as
//! // 1kb zero-filled byte vecs.
//! let p = Pool::new(|| vec![0usize; 1024]);
//!
//! // Create a new handle onto the pool and send it to a new thread.
//! let p2 = p.clone();
//! let t = thread::spawn(move || {
//!     // Get a new vec from the pool and push two values into it.
//!     let mut v = p2.get();
//!     v.push(1);
//!     v.push(2);
//! });
//!
//! // Meanwhile we can still access the original handle from the main
//! // thread and use it to get new vecs from.
//! let mut v = p.get();
//! v.push(1);
//! v.push(2);
//!
//! // Wait for the other thread to complete
//! t.join().unwrap();
//!
//! // When the vec is dropped, it's returned to the pool and is ready to be
//! // used again from a next call to `p.get()`.
//! drop(v);
//! ```

#![forbid(rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, missing_doc_code_examples, unreachable_pub)]
#![cfg_attr(test, deny(warnings))]

use crossbeam_queue::SegQueue;
use std::fmt::{self, Debug};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

struct Internal<T> {
    queue: SegQueue<T>,
    create: Box<dyn Fn() -> T + Send + Sync>,
    clear: Box<dyn Fn(&mut T) + Send + Sync>,
}

impl<T> Internal<T> {
    fn new<C, D>(create: C, clear: D) -> Self
    where
        C: Fn() -> T + Send + Sync + 'static,
        D: Fn(&mut T) -> () + Send + Sync + 'static,
    {
        Internal {
            queue: SegQueue::new(),
            create: Box::new(create),
            clear: Box::new(clear),
        }
    }
}

/// A pool of reusable memory.
pub struct Pool<T> {
    internal: Arc<Internal<T>>,
}

impl<T> Debug for Pool<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pool")
            .field("queue", &format!("[T; {}]", self.internal.queue.len()))
            .field("create", &"Box<dyn Fn() -> T + Send + Sync>")
            .field("clear", &"Box<dyn Fn(&mut T) + Send + Sync>")
            .finish()
    }
}

impl<T> Pool<T> {
    /// Create a new Pool from an initializer function.
    pub fn new<C>(create: C) -> Pool<T>
    where
        C: Fn() -> T + Send + Sync + 'static,
    {
        Pool {
            internal: Arc::new(Internal::new(create, |_| {})),
        }
    }

    /// Create a new Pool from an initializer function and a clear function.
    ///
    /// The clear function will be called whenever the `ItemGuard` is dropped,
    /// and provides an opportunity to clear values and remove potentially
    /// sensitive information from the items before returning it to the memory
    /// pool.
    ///
    /// Note that `drop` in Rust is not guaranteed to run, so this function
    /// is not a replacement for proper security measures.
    pub fn with_clear<C, D>(create: C, clear: D) -> Pool<T>
    where
        C: Fn() -> T + Send + Sync + 'static,
        D: Fn(&mut T) -> () + Send + Sync + 'static,
    {
        Pool {
            internal: Arc::new(Internal::new(create, clear)),
        }
    }

    /// Get an item from the pool.
    pub fn get(&self) -> ItemGuard<T> {
        let pool = &self.internal;
        let item = pool.queue.pop();
        ItemGuard {
            item: Some(item.unwrap_or_else(|_| (*self.internal.create)())),
            pool: self.clone(),
        }
    }

    /// Store an item back inside the pool.
    fn push(&self, mut item: T) {
        (*self.internal.clear)(&mut item);
        self.internal.queue.push(item);
    }
}

impl<T> Clone for Pool<T> {
    fn clone(&self) -> Self {
        Pool {
            internal: self.internal.clone(),
        }
    }
}

/// RAII structure used to reintroduce an item into the pool when dropped.
pub struct ItemGuard<T> {
    item: Option<T>,
    pool: Pool<T>,
}

impl<T: Debug> Debug for ItemGuard<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ItemGuard")
            .field("item", &self.item)
            .finish()
    }
}

impl<T> Drop for ItemGuard<T> {
    fn drop(&mut self) {
        self.pool.push(self.item.take().unwrap())
    }
}

impl<T> Deref for ItemGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item.as_ref().unwrap()
    }
}

impl<T> DerefMut for ItemGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item.as_mut().unwrap()
    }
}
