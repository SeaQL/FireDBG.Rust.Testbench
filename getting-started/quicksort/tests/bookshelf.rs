#[test]
fn test_quicksort_1() {
    let mut books = [
        "The Rust Programming Language",
        "Beginning Rust: From Novice to Professional",
        "Rust in Action",
        "Programming Rust: Fast, Safe Systems Development",
        "Rust Programming Language for Beginners",
    ];
    quicksort::run(&mut books);
    assert_eq!(
        books,
        [
            "Beginning Rust: From Novice to Professional",
            "Programming Rust: Fast, Safe Systems Development",
            "Rust Programming Language for Beginners",
            "Rust in Action",
            "The Rust Programming Language",
        ]
    );
}
