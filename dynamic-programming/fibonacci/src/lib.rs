use firedbg_lib::fire;
use std::{collections::HashMap, sync::Mutex};

lazy_static::lazy_static! {
    static ref MEMO: Mutex<HashMap<i32, i32>> = Mutex::new(HashMap::new());
}

pub fn fib(n: i32) -> i32 {
    if n <= 2 {
        return 1;
    }
    fire::dbg!("return", fib(n - 1) + fib(n - 2))
}

pub fn fibm(n: i32) -> i32 {
    if n <= 2 {
        return 1;
    }
    {
        let mut memo = MEMO.lock().unwrap();
        if let Some(o) = memo.get(&n) {
            return *o;
        }
        // we have to release the lock here
    }
    let res = fibm(n - 1) + fibm(n - 2);
    let mut memo = MEMO.lock().unwrap();
    memo.insert(n, res);
    res
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fib_5() {
        assert_eq!(fib(5), 5);
    }

    #[test]
    fn test_fibm_5() {
        assert_eq!(fibm(5), 5);
    }

    #[test]
    fn test_fib_10() {
        assert_eq!(fib(10), 55);
    }

    #[test]
    fn test_fibm_10() {
        assert_eq!(fibm(10), 55);
    }
}
