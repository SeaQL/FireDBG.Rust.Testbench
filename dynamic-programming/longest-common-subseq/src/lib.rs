use firedbg_lib::fire;
use std::collections::HashMap;

struct Memo {
    memo: HashMap<(usize, usize), usize>,
}

pub fn lcs<T: Copy + Eq>(left: &[T], right: &[T]) -> Vec<T> {
    let mut state = Memo {
        memo: Default::default(),
    };
    let length = lcs_impl(left, right, &mut state);

    // reconstruct the common subsequence
    let mut seq = Vec::new();
    let (mut i, mut j) = (left.len() - 1, right.len() - 1);
    while seq.len() < length {
        if left[i] == right[j] {
            seq.push(left[i]);
            if seq.len() == length {
                break;
            }
            i -= 1;
            j -= 1;
        } else {
            if i > 0 && state.memo.get(&(i, j)).unwrap() == state.memo.get(&(i - 1, j)).unwrap() {
                i -= 1;
            } else {
                j -= 1;
            }
        }
    }
    seq.reverse();
    seq
}

fn lcs_impl<T: Copy + Eq>(left: &[T], right: &[T], state: &mut Memo) -> usize {
    if left.is_empty() || right.is_empty() {
        return 0;
    }
    fire::dbg!("(i, j)", (left.len() - 1, right.len() - 1));
    if let Some(n) = state.memo.get(&(left.len() - 1, right.len() - 1)) {
        return *n;
    }
    let n = if left[left.len() - 1] == right[right.len() - 1] {
        1 + lcs_impl(&left[..left.len() - 1], &right[..right.len() - 1], state)
    } else {
        let leftn = lcs_impl(&left[..left.len() - 1], right, state);
        let rightn = lcs_impl(left, &right[..right.len() - 1], state);
        leftn.max(rightn)
    };
    state.memo.insert((left.len() - 1, right.len() - 1), n);
    n
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lcs_0() {
        assert_eq!(lcs(&[1, 4], &[2, 5]), []);
    }

    #[test]
    fn test_lcs_1() {
        assert_eq!(
            lcs(&[1, 4, 5, 6, 9, 10, 11], &[6, 4, 5, 9, 11]),
            [4, 5, 9, 11]
        );
    }

    #[test]
    fn test_lcs_2() {
        assert_eq!(
            lcs(&[1, 4, 5, 6, 9, 10, 11], &[6, 4, 5, 9, 11, 12]),
            [4, 5, 9, 11]
        );
    }

    #[test]
    fn test_lcs_3() {
        assert_eq!(
            lcs(&[3, 1, 4, 5, 9, 12, 1], &[2, 3, 5, 1, 4, 9, 2, 12]),
            [3, 1, 4, 9, 12]
        );
    }

    #[test]
    fn test_lcs_4() {
        assert_eq!(lcs(&[1, 2, 3, 1, 2, 3], &[1, 2, 3, 4]), [1, 2, 3]);
    }
}
