use cli::cl::*;
use cli::cl2::*;
use std::env;
use std::process::Command;

pub struct Ctx {}

include!(concat!(env!("OUT_DIR"), "/gen.rs"));

#[test]
fn test() {
    let args = vec![
        format!("--foo"),
        format!("aaa"),
        format!("--bar"),
        format!("aaa"),
    ];
    let r = test_from_args(&args).expect("Error");
    println!("{:?}", r);
    let r = test3_from_args(&args).expect("Error");
    println!("{:?}", r);
}
