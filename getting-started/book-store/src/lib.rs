use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn inventory(path: &str) -> Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut books = Vec::new();
    for line in reader.lines() {
        let book = line?.trim().to_owned();
        books.push(book);
    }
    quicksort::run(&mut books);
    Ok(books)
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_inventory_1() -> Result<()> {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/books.txt");
        let books = inventory(path)?;
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
        Ok(())
    }
}
