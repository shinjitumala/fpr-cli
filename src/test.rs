use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct TestCtx {
    account_id: i32,
}

#[derive(Args, Debug)]
#[args(desc = "Execute test2.")]
#[allow(dead_code)]
struct TestArgs2 {
    #[arg(desc = "2", i = Init::None)]
    names: Vec<String>,
}
impl Run<TestCtx> for TestArgs2 {
    fn run(c: &TestCtx, a: Self) -> Result<(), String> {
        todo!()
    }
}

#[derive(Args, Debug)]
#[args(desc = "Execute test.")]
#[allow(dead_code)]
struct TestArgs {
    #[arg(desc = "1", i = Init::None)]
    name: String,
    a: TestArgs2,
}
impl Run<TestCtx> for TestArgs {
    fn run(c: &TestCtx, a: Self) -> Result<(), String> {
        todo!()
    }
}

#[derive(Acts)]
#[acts(desc = "foo")]
#[allow(dead_code)]
struct TestActs2 {
    act1: TestArgs,
}

#[derive(Acts)]
#[acts(desc = "foo")]
#[allow(dead_code)]
struct TestActs {
    map1: TestActs2,
    act2: TestArgs2,
}

#[test]
fn test() -> Result<(), String> {
    use std::fs::File;
    let ctx: TestCtx = serde_json::from_reader(File::open("config.json").expect("Failed to open"))
        .expect("Failed to parse");

    // run::<TestCtx, TestActs>(&ctx)
    // todo!()
}
