use crate::cl::*;
use crate::cl2::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct TestCtx {
    account_id: i32,
}

#[derive(Args)]
#[args(ctx = TestCtx)]
struct Test2 {
    // #[default(_code = "Arg::s(\"foo\")")]
    #[arg(desc = (""), i = (""), act = (""))]
    next: Arg<TestCtx, Opt<One<String>>>,
}

fn x() {
    Arg::<TestCtx, Opt<One<String>>>::new("".into(), Init::Const("".into()));
}

#[derive(Args)]
// #[ctx(TestCtx)]
#[args(ctx = TestCtx)]
struct Test {
    // #[default(_code = "Arg::new(Desc::Dyn(|c| format!(\"FOO{}\",c.account_id)),Init::Const(1))")]
    #[arg(desc = (""), i = 1, act = (""))]
    name: Arg<TestCtx, Req<One<i32>>>,
    // #[default(_code = "Arg::new(Desc::Const(\"foo\"),Init::None)")]
    #[arg(desc = (""), i = Init::None, act = (""))]
    id: Arg<TestCtx, Opt<One<A>>>,
    // #[default(_code = "Arg::s(\"focwo\")")]
    #[arg(desc = (""), i = Init::None, act = (""))]
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

fn foox(c: &TestCtx, x: Ret<TestCtx, Test>) {
    println!("x: {:?}", x);
    println!("c: {:?}", c);
}

#[derive(Acts, SmartDefault)]
#[ctx(TestCtx)]
#[desc("foo")]
struct TestActMap2 {
    #[default(_code = r#"Act::new("IAM LEGEND",foox)"#)]
    act1: Act<TestCtx, Test>,
    #[default(_code = r#"Act::new("2",|_,_|{println!("act2")})"#)]
    act2: Act<TestCtx, Test>,
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
        format!("map1"),
        format!("act1"),
        // format!("--help"),
        format!("--id"),
        format!("10"),
        // format!("10a"),
        // format!("-nyom"),
    ];

    let ctx: TestCtx = serde_json::from_reader(File::open("config.json").expect("Failed to open"))
        .expect("Failed to parse");

    TestActMap::parse(&ctx, &args);
}
