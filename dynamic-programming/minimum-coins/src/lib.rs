use firedbg_lib::fire;
use std::collections::HashMap;

pub fn min_coins(m: i32, coins: &[i32]) -> Option<usize> {
    if m == 0 {
        return Some(0);
    }
    let mut count: Option<usize> = None;
    for coin in coins {
        let next = m - coin;
        if next < 0 {
            continue;
        }
        count = match (count, min_coins(next, coins)) {
            (Some(a), Some(b)) => Some(a.min(b + 1)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b + 1),
            (None, None) => None,
        }
    }
    fire::dbg!("return", count)
}

pub fn min_coins_m(m: i32, coins: &[i32], memo: &mut HashMap<i32, Option<usize>>) -> Option<usize> {
    if m == 0 {
        return Some(0);
    }
    if let Some(c) = memo.get(&m) {
        return *c;
    }
    let mut count: Option<usize> = None;
    for coin in coins {
        let next = m - coin;
        if next < 0 {
            continue;
        }
        count = match (count, min_coins_m(next, coins, memo)) {
            (Some(a), Some(b)) => Some(a.min(b + 1)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b + 1),
            (None, None) => None,
        }
    }
    memo.insert(m, count);
    count
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_min_coins_base() {
        assert_eq!(min_coins(2, &[5]), None);
        assert_eq!(min_coins(5, &[2]), None);
        assert_eq!(min_coins(5, &[1, 2]), Some(3));
        assert_eq!(min_coins(13, &[1, 4, 5]), Some(3));
        assert_eq!(min_coins(12, &[1, 4, 5]), Some(3));
        assert_eq!(min_coins(11, &[1, 4, 5]), Some(3));
        assert_eq!(min_coins(10, &[1, 4, 5]), Some(2));
    }

    #[test]
    fn test_min_coins_13() {
        assert_eq!(min_coins(13, &[2, 4, 5]), Some(3));
    }

    #[test]
    fn test_min_coins_13_m() {
        let mut memo = Default::default();
        assert_eq!(min_coins_m(13, &[2, 4, 5], &mut memo), Some(3));
    }

    #[test]
    fn test_min_coins_euro_m() {
        let mut memo = Default::default();
        assert_eq!(
            min_coins_m(734, &[200, 100, 50, 20, 10, 5, 2, 1], &mut memo),
            Some(8)
        );
    }
}
