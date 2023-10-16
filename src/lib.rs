use std::future::Future;
use std::task::{Context, Poll};

use pin_utils::pin_mut;

pub fn block_on<Fut: Future>(fut: Fut) -> Fut::Output {
    let waker = waker::new(std::thread::current());
    let mut context = Context::from_waker(&waker);

    pin_mut!(fut);

    loop {
        if let Poll::Ready(out) = fut.as_mut().poll(&mut context) {
            return out;
        }

        std::thread::park();
    }
}

mod waker {
    use std::task::{RawWakerVTable, RawWaker, Waker};
    use std::thread::Thread;
    use super::thread;

    static VTABLE: RawWakerVTable = RawWakerVTable::new(
        clone,
        wake,
        wake_by_ref,
        drop,
    );

    pub fn new(thread: Thread) -> Waker {
        unsafe { Waker::from_raw(new_raw(thread)) }
    }

    pub fn new_raw(thread: Thread) -> RawWaker {
        let ptr = unsafe { thread::into_ptr(thread) };
        RawWaker::new(ptr, &VTABLE)
    }

    unsafe fn clone(ptr: *const ()) -> RawWaker {
        let thread = thread::from_ptr_ref(&ptr);
        let thread = thread.clone();
        new_raw(thread)
    }

    unsafe fn wake(ptr: *const ()) {
        thread::from_ptr(ptr).unpark();
    }

    unsafe fn wake_by_ref(ptr: *const ()) {
        thread::from_ptr_ref(&ptr).unpark();
    }

    unsafe fn drop(ptr: *const ()) {
        thread::from_ptr(ptr);
    }
}

/// Functions for casting a [`std::thread::Thread`] to and from a raw pointer
/// for use in [`waker_impl`]. This is just a set of wrappers around transmute
/// for supported cast operations. The soundness of the transmutes is assured
/// with static assertions.
mod thread {
    use std::thread::Thread;

    static_assertions::assert_eq_size!(Thread, *const ());
    static_assertions::assert_eq_align!(Thread, *const ());

    pub unsafe fn from_ptr(ptr: *const ()) -> Thread {
        std::mem::transmute(ptr)
    }

    pub unsafe fn from_ptr_ref(ptr: &*const ()) -> &Thread {
        std::mem::transmute(ptr)
    }

    pub unsafe fn into_ptr(thread: Thread) -> *const () {
        std::mem::transmute(thread)
    }
}
