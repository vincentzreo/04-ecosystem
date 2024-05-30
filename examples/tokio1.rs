use std::{thread, time::Duration};

use tokio::{fs, runtime::Builder, time::sleep};

#[tokio::main]
async fn main() {
    let handle = thread::spawn(|| {
        let rt = Builder::new_current_thread().enable_all().build().unwrap();
        rt.spawn(async {
            println!("Future 1");
            let content = fs::read_to_string("Cargo.toml").await.unwrap();
            println!("Content length: {}", content.len());
        });
        rt.spawn(async {
            println!("Future 2");
            let ret = expensive_blocking_task("Future 2".to_string());
            println!("Hash: {}", ret);
        });
        rt.block_on(async {
            sleep(Duration::from_secs(1)).await;
        });
    });
    handle.join().unwrap();
}

fn expensive_blocking_task(s: String) -> String {
    thread::sleep(Duration::from_millis(800));
    blake3::hash(s.as_bytes()).to_string()
}
