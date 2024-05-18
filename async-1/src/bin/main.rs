use std::time::Duration;

fn uid() -> u64 {
    fastrand::u16(1000..10000) as u64
}

async fn sleeper(ctx: u64, i: u64) {
    let dtx = uid();
    println!("[{ctx}/{dtx}] sleep {i}");
    tokio::time::sleep(Duration::from_millis(i)).await;
    println!("[{ctx}/{dtx}] awake {i}");
}

async fn sleep2(ctx: u64) {
    let dtx = uid();
    println!("[{ctx}/{dtx}] sleep2::enter");
    sleeper(dtx, 1).await;
    println!("[{ctx}/{dtx}] sleep2::wake");
    sleeper(dtx, 1).await;
    println!("[{ctx}/{dtx}] sleep2::exit");
}

async fn sleep3(ctx: u64) {
    let dtx = uid();
    println!("[{ctx}/{dtx}] sleep3::enter");
    sleeper(dtx, 3).await;
    println!("[{ctx}/{dtx}] sleep3::exit");
}

#[tokio::main]
async fn main() {
    let ctx = uid();
    tokio::join!(sleep2(ctx), sleep3(ctx));
}
