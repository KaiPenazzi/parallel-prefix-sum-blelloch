use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};
//use std::time::Instant;
//use tokio::time::{sleep, Duration};

fn pow2(input: usize) -> usize {
    let base: i32 = 2;
    base.pow(input.try_into().unwrap()).try_into().unwrap()
}

fn log2(input: usize) -> usize {
    let float: f32 = input as f32;
    float.log2() as usize
}

async fn reduce(arr: &Arc<[AtomicI32]>) {
    let n = arr.len();

    for d in 0..log2(n) {
        let mut handles = vec![];

        for i in (0..n).step_by(pow2(d + 1)) {
            let ptr = Arc::clone(arr);

            let handle = tokio::spawn(async move {
                ptr[i + pow2(d + 1) - 1].store(
                    ptr[i + pow2(d) - 1].load(Ordering::Relaxed)
                        + ptr[i + pow2(d + 1) - 1].load(Ordering::Relaxed),
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
    arr[1].store(7, Ordering::Relaxed);
    arr[2].store(11, Ordering::Relaxed);
    arr[3].store(15, Ordering::Relaxed);

    println!("{:?}", arr);
    reduce(&arr).await;
    println!("{:?}", arr);
}

fn get_array(n: usize) -> Arc<[AtomicI32]> {
    let mut atomic_vec = vec![];

    for x in 0..n {
        atomic_vec.push(AtomicI32::new((x + 1).try_into().unwrap()));
    }

    Arc::from(atomic_vec.into_boxed_slice())
}
