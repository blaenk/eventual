extern crate syncbox;

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use eventual::{background, defer, join, Future, Async};

// TODO figure out how to get rid of unused import error here
use syncbox::ThreadPool;
#[test]
fn test_defer_runs_on_thread_pool() {
    // Set thread local
    let pool = ThreadPool::single_thread();
    let (complete, future) = Future::<i32, ()>::pair();
    let res = defer(pool, future).and_then(|v: i32| {
        // Assert thread local is not present here
        Ok(v + 5)
    });
    complete.complete(7);
    assert_eq!(Ok(7 + 5), res.await());
}

#[test]
fn test_defer_can_be_joined() {
    let pool = ThreadPool::single_thread();
    let mut futures = vec![];
    let mut completes = vec![];

    for _ in (1 .. 6) {
        let (complete, future) = Future::<i32, ()>::pair();
        let res = defer(pool.clone(), future).and_then(|v: i32| {
            Ok(v + 1)
        });

        futures.push(res);
        completes.push(complete);
    }

    for (complete, num) in completes.into_iter().zip((1 .. 6)) {
        complete.complete(num);
    }

    let res: Vec<i32> = join(futures).await().unwrap();
    assert_eq!(res, vec![2, 3, 4, 5, 6]);
}

#[test]
fn test_threadpool_background() {
    // Set thread local
    let pool = ThreadPool::single_thread();
    let flag = Arc::new(AtomicBool::new(false));
    let f = flag.clone();
    let result = background(pool, Box::new(move || {
        assert!(f.load(Ordering::Relaxed));
        5
    }));
    // Wait for a bit to make sure that the background task hasn't run

    thread::sleep_ms(100);
    // Set the flag
    flag.store(true, Ordering::Relaxed);
    assert_eq!(Ok(5), result.await());
}
