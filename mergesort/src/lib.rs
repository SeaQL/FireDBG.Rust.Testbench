use firedbg_lib::fire;

pub fn run<T: PartialOrd + Copy>(arr: &mut [T]) {
    let len = arr.len();
    merge_sort(arr, 0, len);
    fire::dbg!("sorted", arr);
}

fn merge_sort<T: PartialOrd + Copy>(arr: &mut [T], from: usize, till: usize) {
    // here the convention is `till` is always exclusive
    if from + 1 == till {
        fire::dbg!("do nothing", ());
    } else if from + 2 == till {
        if arr[from] > arr[from + 1] {
            fire::dbg!("swap", (from, from + 1));
            arr.swap(from, from + 1);
        }
    } else {
        let mid = (from + till) / 2;
        merge_sort(arr, from, mid);
        merge_sort(arr, mid, till);
        merge(arr, from, mid, till);
    }
}

fn merge<T: PartialOrd + Copy>(arr: &mut [T], from: usize, mid: usize, till: usize) {
    let left = arr[from..mid].to_vec();
    let right = arr[mid..till].to_vec();
    let (mut i, mut j, mut k) = (0, 0, from);

    // until destination fully filled
    while k < till {
        // if left is used up already, always use right. vice versa
        // pick the smaller candidate from left or right
        if j == right.len() || (i < left.len() && left[i] <= right[j]) {
            arr[k] = left[i];
            i += 1;
        } else {
            arr[k] = right[j];
            j += 1;
        }
        k += 1; // filling destination
    }
}
