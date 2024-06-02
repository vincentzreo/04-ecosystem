use loom::sync::atomic::AtomicUsize;
use loom::sync::atomic::Ordering::Relaxed;
use loom::sync::Arc;
use loom::thread;

#[test]
fn buggy_concurrent_inc() {
    loom::model(|| {
        let num = Arc::new(AtomicUsize::new(0));

        let this: Vec<_> = (0..2)
            .map(|_| {
                let num = num.clone();
                thread::spawn(move || {
                    /* let curr = num.load(Acquire);
                    num.store(curr + 1, Release); */

                    // fix
                    num.fetch_add(1, Relaxed);
                })
            })
            .collect();

        for th in this {
            th.join().unwrap();
        }

        assert_eq!(2, num.load(Relaxed));
    });
}
