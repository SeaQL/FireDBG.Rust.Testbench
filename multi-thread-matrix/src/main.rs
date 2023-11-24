use firedbg_lib::fire;
use std::sync::mpsc::{channel, Receiver, Sender};
use visioncortex::Matrix;

const D: usize = 3;
const THREADS: usize = 4;
const ITEMS: usize = 100;

fn inverse(m: &Matrix<D, D>) -> Option<Matrix<D, D>> {
    fire::dbg!("return", Matrix::inv(m))
}

fn runner(receiver: Receiver<Matrix<D, D>>, sender: Sender<(Matrix<D, D>, Option<Matrix<D, D>>)>) {
    while let Ok(m) = receiver.recv() {
        // send back the input with solution
        let mm = inverse(&m);
        sender.send((m, mm)).unwrap();
    }
}

fn main() {
    fastrand::seed(2022);

    let (result, collector) = channel();
    let mut senders = Vec::new();
    for _ in 0..THREADS {
        let (sender, receiver) = channel();
        senders.push(sender);
        let result = result.clone();
        std::thread::spawn(move || runner(receiver, result));
    }

    for c in 0..ITEMS {
        let mut m = Matrix::<D, D>::default();
        for i in 0..D {
            for j in 0..D {
                m.m[i][j] = if fastrand::i32(0..9) == 0 {
                    0.0
                } else {
                    fastrand::i32(-10..10) as f64
                };
            }
        }
        senders[c % THREADS].send(m).unwrap();
    }

    for _ in 0..ITEMS {
        let (m, mm) = collector.recv().unwrap();
        println!("Source  = {m:?}");
        println!("Inverse = {mm:?}");
    }
}
