use std::time::Duration;

use banan::{time, Runtime};

fn main() {
    Runtime::block_on(async {
        Runtime::spawn(async {
            loop {
                time::snooze(Duration::from_millis(333)).await;
                println!("333");
            }
        });
        loop {
            time::snooze(Duration::from_millis(450)).await;
            println!("450");
        }
    });
}
