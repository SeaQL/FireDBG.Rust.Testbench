use firedbg_lib::fire;

pub fn run<T: PartialOrd>(arr: &mut [T]) {
    let len = arr.len();
    quick_sort(arr, 0, (len - 1) as isize);
}

fn quick_sort<T: PartialOrd>(arr: &mut [T], low: isize, high: isize) {
    if low < high {
        let p = partition(arr, low, high);
        quick_sort(arr, low, p - 1);
        quick_sort(arr, p + 1, high);
    }
}

fn partition<T: PartialOrd>(arr: &mut [T], low: isize, high: isize) -> isize {
    let pivot = high as usize;
    let mut store_index = low - 1;
    let mut last_index = high;

    loop {
        store_index += 1;
        while arr[store_index as usize] < arr[pivot] {
            store_index += 1;
        }
        last_index -= 1;
        while last_index >= 0 && arr[last_index as usize] > arr[pivot] {
            last_index -= 1;
        }
        if store_index >= last_index {
            break;
        } else {
            fire::dbg!("swap", &arr[store_index as usize..=last_index as usize]);
            swap(&mut arr[store_index as usize..=last_index as usize]);
        }
    }
    fire::dbg!("swap", &arr[store_index as usize..=pivot as usize]);
    swap(&mut arr[store_index as usize..=pivot as usize]);
    store_index
}

fn swap<T: PartialOrd>(arr: &mut [T]) {
    arr.swap(0, arr.len() - 1);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_quicksort_1() {
        let mut numbers = [4, 65, 2, -31, 0, 99, 2, 83, 782, 1];
        run(&mut numbers);
        assert_eq!(numbers, [-31, 0, 1, 2, 2, 4, 65, 83, 99, 782]);
    }

    #[test]
    fn test_quicksort_2() {
        let mut numbers = [1, 2, 2];
        run(&mut numbers);
        assert_eq!(numbers, [1, 2, 2]);
    }
}
