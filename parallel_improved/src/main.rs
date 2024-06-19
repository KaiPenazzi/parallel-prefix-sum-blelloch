use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};
use std::time::Instant;
use tokio::time::{sleep, Duration};

fn pow2(input: usize) -> usize {
    let base: i32 = 2;
    base.pow(input.try_into().unwrap()).try_into().unwrap()
}

fn log2(input: usize) -> usize {
    let float: f32 = input as f32;
    float.log2() as usize
}

async fn sweep_up(arr: Arc<[AtomicI32]>, d: usize, i: usize) {
    let a: usize = i + pow2(d + 1) - 1;
    let b: usize = i + pow2(d) - 1;
    let c: usize = i + pow2(d + 1) - 1;

    //println!("a: {}, b: {}, c: {}, d: {}, i: {}", a, b, c, d, i);

    //arr[a] = arr[b].load() + arr[c].load();
    //let _ = sleep(Duration::from_millis(1));
    arr[a].store(
        arr[b].load(Ordering::Relaxed) + arr[c].load(Ordering::Relaxed),
        Ordering::Relaxed,
    );
}

async fn sweep_down(arr: Arc<[AtomicI32]>, d: usize, i: usize) {
    let t: i32 = arr[i + pow2(d) - 1].load(Ordering::Relaxed);
    arr[i + pow2(d) - 1].store(
        arr[i + pow2(d + 1) - 1].load(Ordering::Relaxed),
        Ordering::Relaxed,
    );
    arr[i + pow2(d + 1) - 1].store(
        t + arr[i + pow2(d + 1) - 1].load(Ordering::Relaxed),
        Ordering::Relaxed,
    );
    //let _ = sleep(Duration::from_millis(1));
}

async fn reduce(arr: &Arc<[AtomicI32]>) {
    //println!("reduce: {:?}", arr);
    let n = arr.len();

    for d in 0..log2(n) {
        let mut handles = vec![];

        for i in (0..n).step_by(pow2(d + 1)) {
            let ptr = Arc::clone(arr);
            let handle = tokio::spawn(async move {
                sweep_up(ptr, d, i).await;
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.await;
        }
    }
}

async fn scan(arr: &Arc<[AtomicI32]>, i: i32) {
    //println!("scan: {:?}", arr);
    let n = arr.len();
    arr[n - 1].store(i, Ordering::Relaxed);
    for d in (0..log2(n)).rev() {
        let mut handles = vec![];
        for i in (0..n).step_by(pow2(d + 1)) {
            let ptr = Arc::clone(arr);
            let handle = tokio::spawn(async move {
                sweep_down(ptr, d, i).await;
            });

            handles.push(handle);
        }
        for handle in handles {
            let _ = handle.await;
        }
    }
}

async fn prefix_sum(arr: &Arc<[AtomicI32]>, p: usize) {
    //up sweep or + reduce
    let n = arr.len();
    let y = n / p;
    let sum: Arc<[AtomicI32]> = get_array(p);
    let mut handles1 = vec![];
    let mut handles2 = vec![];
    let mut handles3 = vec![];

    let slices = make_slices(arr, p);

    for i in 0..p {
        let ptr_slice = Arc::clone(&slices[i]);
        //let t = sum[i].load(Ordering::Relaxed);

        let handle = tokio::spawn(async move {
            reduce(&ptr_slice).await;
        });
        handles1.push(handle);
    }

    for i in 0..p {
        let ptr_a = Arc::clone(arr);
        let ptr_s = Arc::clone(&sum);
        let handle = tokio::spawn(async move {
            ptr_s[i].store(ptr_a[y * i].load(Ordering::Relaxed), Ordering::Relaxed);
            for j in 1..y {
                ptr_s[i].fetch_add(ptr_a[y * i + j].load(Ordering::Relaxed), Ordering::Relaxed);
            }
        });
        handles2.push(handle);
    }

    for handle in handles2 {
        _ = handle.await;
    }

    //println!("{:?}", sum);
    reduce(&sum).await;
    //println!("{:?}", sum);
    scan(&sum, 0).await;
    //println!("{:?}", sum);

    //println!("{:?}", slices);

    for handle in handles1 {
        _ = handle.await;
    }

    for i in 0..p {
        let ptr_slice = Arc::clone(&slices[i]);
        let t = sum[i].load(Ordering::Relaxed);

        let handle = tokio::spawn(async move {
            scan(&ptr_slice, t).await;
        });
        handles3.push(handle);
    }

    for handle in handles3 {
        _ = handle.await;
    }

    //println!("{:?}", slices[0].last());
    //println!("{:?}", slices);
    //

    for i in 0..slices.len() {
        for j in 0..slices[i].len() {
            arr[(i * y) + j].store(slices[i][j].load(Ordering::Relaxed), Ordering::Relaxed);
        }
    }
}

#[tokio::main]
async fn main() {
    let arr = get_array(16777216);
    //let arr = get_array(8);

    //arr[0].store(1, Ordering::Relaxed);
    //arr[1].store(2, Ordering::Relaxed);
    //arr[2].store(3, Ordering::Relaxed);
    //arr[3].store(4, Ordering::Relaxed);
    //arr[4].store(5, Ordering::Relaxed);
    //arr[5].store(6, Ordering::Relaxed);
    //arr[6].store(7, Ordering::Relaxed);
    //arr[7].store(8, Ordering::Relaxed);

    let start = Instant::now();
    prefix_sum(&arr, 4).await;
    let end = Instant::now();

    //println!("{:?}", arr);
    println!("{:?}", end - start);
}

fn get_array(n: usize) -> Arc<[AtomicI32]> {
    let mut atomic_vec = vec![];

    for _ in 0..n {
        atomic_vec.push(AtomicI32::new(1));
    }

    Arc::from(atomic_vec.into_boxed_slice())
}

fn make_slices(arr: &Arc<[AtomicI32]>, n: usize) -> Vec<Arc<[AtomicI32]>> {
    let mut ret = vec![];
    let size = arr.len() / n;

    for i in 0..n {
        let slice = get_array(size);

        for j in 0..slice.len() {
            let d = (i * size) + j;
            slice[j].store(arr[d].load(Ordering::Relaxed), Ordering::Relaxed);
        }

        ret.push(slice);
    }

    ret
}
