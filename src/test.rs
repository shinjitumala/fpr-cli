use crate::cl2::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Ctx {
    account_id: i32,
}

#[derive(Args, SmartDefault)]
#[ctx(Ctx)]
struct Test2 {
    #[default(_code = "Arg::s(\"foo\")")]
    next: Arg<Ctx, Opt<One<String>>>,
}

#[derive(Args, SmartDefault)]
#[ctx(Ctx)]
struct Test {
    #[default(_code = "Arg::new(Desc::Dyn(|c| format!(\"FOO{}\",c.account_id)),Init::Const(1))")]
    name: Arg<Ctx, Req<One<i32>>>,
    #[default(_code = "Arg::s(\"foo\")")]
    id: Arg<Ctx, Opt<One<A>>>,
    #[default(_code = "Arg::s(\"focwo\")")]
    ids: Arg<Ctx, Opt<Vec<i32>>>,
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

#[test]
fn test() {
    use std::fs::File;

    let args = vec![
        // format!("-id"),
        // format!("foo"),
        format!("--help"),
        // format!("-name"),
        // format!("10"),
        // format!("10a"),
        // format!("-nyom"),
    ];

    let ctx: Ctx = serde_json::from_reader(File::open("config.json").expect("Failed to open"))
        .expect("Failed to parse");

    let x = parse::<Ctx, Test>(&ctx, &args).expect("Parse failed");
    println!("{:?}", x);
}
