use firedbg_lib::fire;
use std::iter::repeat_with;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    /// Random seed
    #[structopt(long, default_value = "2525")]
    seed: u64,
    /// Number of random numbers to be sorted
    #[structopt(default_value = "10")]
    n: usize,
}

fn main() {
    let Opt { seed, n } = Opt::from_args();

    fire::dbg!(&seed);
    fire::dbg!(&n);

    fastrand::seed(seed);

    let max = if n <= 10 { 100 } else { 1000 };

    println!("Sort {n} numbers in ascending order");
    let mut numbers: Vec<_> = repeat_with(|| fastrand::i32(1..max)).take(n).collect();

    println!("Input:  {:?}", numbers);
    merge_sort::run(&mut numbers);
    println!("Sorted: {:?}", numbers);

    let mut c = 0;
    for n in numbers {
        assert!(n >= c);
        c = n;
    }
}
