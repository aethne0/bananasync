use std::time::Duration;

fn main() {
    let mut rt = banan::Runtime::new();

    rt.block_on(async {
        loop {
            banan::time::snooze(Duration::from_millis(500)).await;
            println!("haha");
        }
    });
}
