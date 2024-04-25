use crate::cl::*;
use crate::cl2::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct TestCtx {
    account_id: i32,
}

#[derive(Args, SmartDefault)]
#[ctx(TestCtx)]
struct Test2 {
    #[default(_code = "Arg::s(\"foo\")")]
    next: Arg<TestCtx, Opt<One<String>>>,
}

#[derive(Args, SmartDefault)]
#[ctx(TestCtx)]
struct Test {
    #[default(_code = "Arg::new(Desc::Dyn(|c| format!(\"FOO{}\",c.account_id)),Init::Const(1))")]
    name: Arg<TestCtx, Req<One<i32>>>,
    #[default(_code = "Arg::s(\"foo\")")]
    id: Arg<TestCtx, Opt<One<A>>>,
    #[default(_code = "Arg::s(\"focwo\")")]
    ids: Arg<TestCtx, Opt<Vec<i32>>>,
    test: Test2,
}

#[derive(Debug, Clone)]
struct A {
    _name: String,
}
impl Parse for A {
    fn parse(_name: &'static str, tkn: &String) -> ParseResult<Self> {
        Ok(Self {
            _name: tkn.to_owned(),
        })
    }
}

#[derive(Acts, SmartDefault)]
#[ctx(TestCtx)]
#[desc("foo")]
struct TestActMap2 {
    #[default(_code = "Act::new(\"1\",|_,_|{println!(\"act1\")})")]
    act1: Act<TestCtx>,
    #[default(_code = "Act::new(\"2\",|_,_|{println!(\"act2\")})")]
    act2: Act<TestCtx>,
}

#[derive(Acts, SmartDefault)]
#[ctx(TestCtx)]
#[desc("foo")]
struct TestActMap {
    map1: TestActMap2,
}

#[test]
fn test() {
    use std::fs::File;

    let args = vec![
        format!("map2"),
        // format!("act3"),
        // format!("--help"),
        // format!("-name"),
        // format!("10"),
        // format!("10a"),
        // format!("-nyom"),
    ];

    let ctx: TestCtx = serde_json::from_reader(File::open("config.json").expect("Failed to open"))
        .expect("Failed to parse");

    TestActMap::parse(&ctx, &args);

    // let x = parse::<Ctx, Test>(&ctx, &args).expect("Parse failed");
    // println!("{:?}", x);
}
