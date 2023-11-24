use firedbg_lib::fire;

pub fn run<T: PartialOrd>(arr: &mut [T]) {
    let len = arr.len();
    quick_sort(arr, 0, (len - 1) as isize);
    fire::dbg!("sorted", arr);
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
            arr.swap(store_index as usize, last_index as usize);
        }
    }
    arr.swap(store_index as usize, pivot as usize);
    fire::dbg!("after", arr);
    store_index
}
