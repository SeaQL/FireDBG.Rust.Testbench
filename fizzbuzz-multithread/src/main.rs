#![allow(non_snake_case, unused_variables)]

use firedbg_lib::fire;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

fn fizzer(sender: SyncSender<()>) {
    for i in 1..=100 {
        if i % 15 == 0 {
            fizz(i);
            sender.send(()).unwrap();
            sender.send(()).unwrap();
        } else if i % 3 == 0 {
            fizz(i);
            println!();
        } else if i % 5 == 0 {
            sender.send(()).unwrap();
            sender.send(()).unwrap();
        } else {
            I(i);
            println!("{i}");
        }
    }
}

fn buzzer(receiver: Receiver<()>) {
    let mut j = 0;
    while let Ok(()) = receiver.recv() {
        j += 5;
        buzz(j);
        receiver.recv().unwrap();
    }
}

#[inline(never)]
fn fizz(i: i32) {
    print!("Fizz");
}

#[inline(never)]
fn buzz(j: i32) {
    println!("Buzz");
}

#[inline(never)]
fn I(i: i32) {
    fire::dbg!(i);
}

fn main() {
    let (sender, receiver) = sync_channel(0);
    std::thread::spawn(move || buzzer(receiver));

    fizzer(sender);
}
