mod switch;
mod rpc;
mod memory;
mod focus;

fn main() {
    sleep(Duration::from_millis(1000));
    switch::test();
    use std::thread::sleep;
    use std::time::Duration;
    sleep(Duration::from_millis(1000));
}

