use std::{thread, time::Duration};

use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel(32);
    let handle = worker(rx);
    tokio::spawn(async move {
        let mut i = 0;
        loop {
            i += 1;
            println!("sending {}", i);
            tx.send(format!("task {i}")).await?;
        }
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    });
    handle.join().unwrap();
    Ok(())
}

fn worker(mut rx: mpsc::Receiver<String>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while let Some(s) = rx.blocking_recv() {
            let s = expensive_blocking_task(s);
            println!("result: {}", s);
        }
    })
}

fn expensive_blocking_task(s: String) -> String {
    thread::sleep(Duration::from_millis(800));
    blake3::hash(s.as_bytes()).to_string()
}
