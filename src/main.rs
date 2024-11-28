mod runtime;
mod task;
mod timer;
mod waker;

use futures::future::join;
use runtime::Runtime;
use std::time::{Duration, Instant};
use timer::Timeout;

fn main() {
    let runtime = Runtime::new();
    let r = runtime.clone();

    runtime.run(async move {
        let start_at = Instant::now();
        r.spawn(async move {
            Timeout::set(Duration::from_millis(5000)).await;
            println!("5000ms: {}ms", start_at.elapsed().as_millis());
        });
        join(
            async {
                Timeout::set(Duration::from_millis(1000)).await;
                println!("1000ms: {}ms", start_at.elapsed().as_millis());
                Timeout::set(Duration::from_millis(2000)).await;
                println!("3000ms: {}ms", start_at.elapsed().as_millis());
                Timeout::set(Duration::from_millis(1000)).await;
                println!("4000ms: {}ms", start_at.elapsed().as_millis());
            },
            async {
                Timeout::set(Duration::from_millis(2000)).await;
                println!("2000ms: {}ms", start_at.elapsed().as_millis());
            },
        )
        .await;
        println!("joined: {}ms", start_at.elapsed().as_millis());
    });
}
