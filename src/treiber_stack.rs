//! Treiber stack, copied from crossbeam's examples directory.
//!
//! https://github.com/crossbeam-rs/crossbeam/blob/master/crossbeam-epoch/examples/treiber_stack.rs

use std::mem::ManuallyDrop;
use std::ptr;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};

use crossbeam_epoch::{Atomic, Owned};
use crossbeam_utils::thread::scope;

/// Treiber's lock-free stack.
///
/// Usable with any number of producers and consumers.
#[derive(Debug)]
pub struct TreiberStack<T> {
    head: Atomic<Node<T>>,
}

#[derive(Debug)]
pub(super) struct Node<T> {
    data: ManuallyDrop<T>,
    next: Atomic<Node<T>>,
}

impl<T> TreiberStack<T> {
    /// Creates a new, empty stack.
    pub(super) fn new() -> TreiberStack<T> {
        TreiberStack {
            head: Atomic::null(),
        }
    }

    /// Pushes a value on top of the stack.
    pub(super) fn push(&self, t: T) {
        let mut n = Owned::new(Node {
            data: ManuallyDrop::new(t),
            next: Atomic::null(),
        });

        let guard = crossbeam_epoch::pin();

        loop {
            let head = self.head.load(Relaxed, &guard);
            n.next.store(head, Relaxed);

            match self.head.compare_and_set(head, n, Release, &guard) {
                Ok(_) => break,
                Err(e) => n = e.new,
            }
        }
    }

    /// Attempts to pop the top element from the stack.
    ///
    /// Returns `None` if the stack is empty.
    pub(super) fn pop(&self) -> Option<T> {
        let guard = crossbeam_epoch::pin();
        loop {
            let head = self.head.load(Acquire, &guard);

            match unsafe { head.as_ref() } {
                Some(h) => {
                    let next = h.next.load(Relaxed, &guard);

                    if self
                        .head
                        .compare_and_set(head, next, Release, &guard)
                        .is_ok()
                    {
                        unsafe {
                            guard.defer_destroy(head);
                            return Some(ManuallyDrop::into_inner(ptr::read(&(*h).data)));
                        }
                    }
                }
                None => return None,
            }
        }
    }

    /// Returns `true` if the stack is empty.
    pub(super) fn is_empty(&self) -> bool {
        let guard = crossbeam_epoch::pin();
        self.head.load(Acquire, &guard).is_null()
    }
}

impl<T> Drop for TreiberStack<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}
