use crate::cl::*;
use crate::cl2::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct TestCtx {
    account_id: i32,
}

type V<V> = Arg<TestCtx, V>;

#[derive(Args)]
#[args(ctx = TestCtx)]
struct Test2 {
    #[arg(desc = (""), i = (""))]
    next: V<Opt<One<String>>>,
}

#[derive(Args)]
#[args(ctx = TestCtx)]
struct Test {
    #[arg(desc = (""), i = 1)]
    name: V<Req<One<i32>>>,
    #[arg(desc = (""), i = Init::None)]
    id: V<Opt<One<B>>>,
    #[arg(desc = (""), i = Init::None)]
    ids: V<Opt<Vec<i32>>>,
    test: Test2,
}

#[derive(Debug, Clone)]
struct B {
    _name: String,
}
impl Parse for B {
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

type A<Args> = Act<TestCtx, Args>;

#[derive(Acts)]
#[acts(ctx = TestCtx, desc = "foo")]
struct TestActMap3 {
    #[act(desc = "IAM LEGEND",act = foox)]
    act9: A<Test>,
    #[act(desc = "2",act = |_,_|{println!("act2")})]
    act10: A<Test>,
}

#[derive(Acts)]
#[acts(ctx = TestCtx, desc = "foo")]
struct TestActMap2 {
    #[act(desc = "IAM LEGEND",act = foox)]
    act1: A<Test>,
    #[act(desc = "2",act = |_,_|{println!("act2")})]
    act2: A<Test>,
    act3: TestActMap3,
}

#[derive(Acts)]
#[acts(ctx = TestCtx, desc="foo")]
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
        format!("--nyom"),
    ];

    let ctx: TestCtx = serde_json::from_reader(File::open("config.json").expect("Failed to open"))
        .expect("Failed to parse");

    crate::cl::parse::<_, TestActMap>(&ctx, &args);
    let y = crate::cl::list::<_, TestActMap>();
    println!("{:?}", y);
}
