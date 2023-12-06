use std::iter::repeat_with;

fn main() {
    const SEED: u64 = 1216;
    const N: usize = 10;

    fastrand::seed(SEED);

    println!("Sort {N} numbers in ascending order");
    let mut numbers: Vec<_> = repeat_with(|| fastrand::i32(1..100)).take(N).collect();

    println!("Input:  {:?}", numbers);
    quicksort::run(&mut numbers);
    println!("Sorted: {:?}", numbers);

    let mut c = 0;
    for n in numbers {
        assert!(n >= c);
        c = n;
    }
}
