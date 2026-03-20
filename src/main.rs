use std::{time::Duration};

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
            time::snooze(Duration::from_millis(500)).await;
            println!("500");
        }
    });
}

/*
fn main() {
    thread::spawn(|| {
        Runtime::block_on(async {
            loop {
                time::snooze(Duration::from_millis(500)).await;
                println!("500");
            }
        });
    });

    Runtime::block_on(async {
        loop {
            time::snooze(Duration::from_millis(333)).await;
            println!("333");
        }
    });
}
*/
