use crate::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct TestCtx {
    account_id: i32,
}

#[derive(Args, Debug)]
#[args(desc = "foo")]
pub struct TestArgs2 {
    #[arg(desc = "2", i = Init::None)]
    names: Vec<String>,
}
impl Run<C> for TestArgs2 {
    type R = String;
    fn run(_c: &C, a: Self) -> Result<Self::R, String> {
        let _ = a.names;
        Ok(format!(""))
    }
}

#[derive(Args, Debug)]
#[args(desc = "bar")]
pub struct TestArgs {
    #[arg(desc = "1", i = Init::None)]
    name: String,
    a: TestArgs2,
}
impl Run<C> for TestArgs {
    type R = ();
    fn run(_c: &C, a: Self) -> Result<Self::R, String> {
        let _ = a.a;
        let _ = a.name;
        Ok(())
    }
}

#[derive(Acts)]
#[acts(desc = "y")]
#[allow(dead_code)]
struct Y(TestArgs2);

type Z = TestArgs;

#[derive(Acts)]
#[acts(desc = "main")]
#[allow(dead_code)]
struct Main(Z, Y);

type C = TestCtx;

#[test]
fn test() -> Result<(), ()> {
    use std::fs::File;
    let ctx: TestCtx = serde_json::from_reader(File::open("config.json").expect("Failed to open"))
        .expect("Failed to parse");

    match Main::run(&ctx) {
        Ok(o) => Ok(o),
        Err(e) => Err(println!("{e}")),
    }
}
