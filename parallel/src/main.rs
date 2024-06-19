use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};
use std::time::Instant;
//use tokio::time::{sleep, Duration};

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

    //arr[a] = arr[b].load() + arr[c].load();
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
}

async fn reduce(arr: &Arc<[AtomicI32]>) {
    let n = arr.len();
    let mut handles = vec![];

    for d in 0..log2(n) {
        for i in (0..n).step_by(pow2(d + 1)) {
            let ptr = Arc::clone(arr);
            let handle = tokio::spawn(async move {
                //println!("start");
                //sleep(Duration::from_secs(1)).await;
                sweep_up(ptr, d, i).await;
                //println!("stop");
            });
            handles.push(handle);
        }
    }

    for handle in handles {
        let _ = handle.await;
    }
}

async fn scan(arr: &Arc<[AtomicI32]>) {
    let n = arr.len();
    let mut handles = vec![];
    arr[n - 1].store(0, Ordering::Relaxed);
    for d in (0..log2(n)).rev() {
        for i in (0..n).step_by(pow2(d + 1)) {
            let ptr = Arc::clone(arr);
            let handle = tokio::spawn(async move {
                sweep_down(ptr, d, i).await;
            });

            handles.push(handle);
        }
    }

    for handle in handles {
        let _ = handle.await;
    }
}

async fn prefix_sum(arr: &Arc<[AtomicI32]>) {
    //up sweep or + reduce
    reduce(arr).await;
    scan(arr).await;
}

#[tokio::main]
async fn main() {
    let arr = get_array(1024);

    let start = Instant::now();
    prefix_sum(&arr).await;
    let end = Instant::now();

    println!("{:?}", arr);
    println!("{:?}", end - start);
}

fn get_array(n: usize) -> Arc<[AtomicI32]> {
    let mut atomic_vec = vec![];

    for _ in 0..n {
        atomic_vec.push(AtomicI32::new(1));
    }

    Arc::from(atomic_vec.into_boxed_slice())
}
