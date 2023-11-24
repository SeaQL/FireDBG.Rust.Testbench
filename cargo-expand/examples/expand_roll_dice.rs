use cargo_expand::*;
use std::{io, io::Write, process};

fn main() {
    let expand = Expand {
        manifest_path: Some(concat!(env!("CARGO_MANIFEST_DIR"), "/../roll_dice/Cargo.toml").into()),
        ..Default::default()
    };
    let result = cargo_expand(expand);
    process::exit(match result {
        Ok(code) => code,
        Err(err) => {
            let _ = writeln!(io::stderr(), "{}", err);
            1
        }
    });
}
