use std::time::Instant;
use tokio::time::{sleep, Duration};

async fn seriell(input: &[i32], out: &mut [i32]) {
    let mut sum: i32 = input[0];
    out[0] = sum;

    for i in 1..input.len() {
        sum += input[i];
        out[i] = sum;
    }
}

#[tokio::main]
async fn main() {
    let data = get_array(16777216);
    let mut out = get_array(16777216);

    let start = Instant::now();
    seriell(&data, &mut out).await;
    let end = Instant::now();

    //println!("{:?}", out);
    print!("{:?}", end - start);
}

fn get_array(size: usize) -> Vec<i32> {
    let mut arr: Vec<i32> = vec![];

    for _ in 0..size {
        arr.push(1);
    }

    arr
}
