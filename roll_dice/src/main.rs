use firedbg_lib::fire;

fn roll(i: i32) -> Result<(), ()> {
    println!("roll {i}");
    let (a, b, c) = (dice(i), dice(i), dice(i));
    a?;
    b?;
    c?;
    if fire::dbg!("roll_u32", fastrand::u32(0..4)) == 0 {
        roll(i - 1)
    } else {
        throw(i - 1)
    }
}

fn throw(i: i32) -> Result<(), ()> {
    println!("throw {i}");
    match fire::dbg!("throw -> i32", fastrand::i32(1..=3)) {
        1 => dice(i)?,
        2 => {
            let (a, b) = (dice(i), dice(i));
            a?;
            b?;
        }
        3 => {
            let (a, b, c) = (dice(i), dice(i), dice(i));
            a?;
            b?;
            c?;
        }
        _ => unreachable!(),
    }
    if fire::dbg!("throw -> bool", fastrand::bool()) {
        roll(i - 1)
    } else {
        throw(i - 1)
    }
}

fn dice(i: i32) -> Result<(), ()> {
    print!("dice {i} = ");
    if fire::dbg!(fastrand::i32(0..i)) == 0 {
        println!("err");
        Err(())
    } else {
        println!("ok");
        Ok(())
    }
}

fn main() {
    roll(25).unwrap();
}
