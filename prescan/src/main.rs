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

async fn sweep_down(arr: Arc<[AtomicI32]>, d: usize, i: usize) {
    println!("{:?}", arr);
    let a = i + pow2(d) - 1;
    let b = i + pow2(d + 1) - 1;

    println!("{a:} = {b:}, {b:} = {a:} + {b:}",);

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

async fn prescan(arr: &Arc<[AtomicI32]>, identity: i32) {
    let n = arr.len();
    arr[n - 1].store(identity, Ordering::Relaxed);
    for d in (0..log2(n)).rev() {
        let mut handles = vec![];
        for i in (0..n).step_by(pow2(d + 1)) {
            let ptr = Arc::clone(arr);
            let handle = tokio::spawn(async move {
                let t: i32 = ptr[i + pow2(d) - 1].load(Ordering::Relaxed);
                ptr[i + pow2(d) - 1].store(
                    ptr[i + pow2(d + 1) - 1].load(Ordering::Relaxed),
                    Ordering::Relaxed,
                );
                ptr[i + pow2(d + 1) - 1].store(
                    t + ptr[i + pow2(d + 1) - 1].load(Ordering::Relaxed),
                    Ordering::Relaxed,
                );
            });

            handles.push(handle);
        }
        for handle in handles {
            let _ = handle.await;
        }
    }
}

#[tokio::main]
async fn main() {
    let arr = get_array(4);
    arr[0].store(3, Ordering::Relaxed);
    arr[1].store(10, Ordering::Relaxed);
    arr[2].store(11, Ordering::Relaxed);
    arr[3].store(36, Ordering::Relaxed);

    let start = Instant::now();
    prescan(&arr, 0).await;
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
