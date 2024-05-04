use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct TestCtx {
    account_id: i32,
}

// type V<V> = Arg<TestCtx, V>;
//
// #[derive(Args)]
// #[args(ctx = TestCtx)]
// struct Test2 {
//     #[arg(desc = (""), i = (""))]
//     next: V<Opt<One<String>>>,
// }
//
// #[derive(Args)]
// #[args(ctx = TestCtx)]
// struct Test {
//     #[arg(desc = (""), i = 1)]
//     name: V<Req<One<i32>>>,
//     #[arg(desc = (""), i = Init::None)]
//     id: V<Opt<One<B>>>,
//     #[arg(desc = (""), i = Init::None)]
//     ids: V<Opt<Vec<i32>>>,
//     test: Test2,
// }
//
// #[derive(Debug, Clone)]
// struct B {
//     _name: String,
// }
// impl Parse for B {
//     fn parse(_name: &'static str, tkn: &String) -> ParseResult<Self> {
//         Ok(Self {
//             _name: tkn.to_owned(),
//         })
//     }
// }
//
// fn foox(c: &TestCtx, x: Ret<TestCtx, Test>) -> Result<(), String> {
//     println!("x: {:?}", x);
//     println!("c: {:?}", c);
//     Ok(())
// }
//
// type A<Args> = Act<TestCtx, Args>;
//
// #[derive(Acts)]
// #[acts(ctx = TestCtx, desc = "foo")]
// struct TestActMap3 {
//     #[act(desc = "IAM LEGEND",act = foox)]
//     act9: A<Test>,
//     #[act(desc = "2",act = |_,_|{println!("act2"); Ok(())})]
//     act10: A<Test>,
// }
//
// #[derive(Acts)]
// #[acts(ctx = TestCtx, desc = "foo")]
// struct TestActMap2 {
//     #[act(desc = "IAM LEGEND",act = foox)]
//     act1: A<Test>,
//     #[act(desc = "2",act = |_,_|{println!("act2"); Ok(())})]
//     act2: A<Test>,
//     act3: TestActMap3,
// }
//
// #[derive(Acts)]
// #[acts(ctx = TestCtx, desc="foo")]
// struct TestActMap {
//     map1: TestActMap2,
// }
//

#[derive(Args, Debug)]
#[args(ctx = TestCtx)]
struct TestArgs2 {
    #[arg(desc = ("2"), i = Init::None)]
    names: Vec<String>,
}

#[derive(Args, Debug)]
#[args(ctx = TestCtx)]
struct TestArgs {
    #[arg(desc = ("1"), i = Init::None)]
    name: String,
    a: TestArgs2,
}

#[derive(Acts)]
#[acts(ctx = TestCtx, desc = "foo")]
struct TestActs2 {
    #[act(desc = "IAM LEGEND", act = |a,b|{Ok(())})]
    act1: TestArgs,
}

#[derive(Acts)]
#[acts(ctx = TestCtx, desc = "foo")]
struct TestActs {
    map1: TestActs2,
    #[act(desc = "IAM LEGEND", act = |a,b|{Ok(())})]
    act2: TestArgs2,
}

#[test]
fn test() -> Res<()> {
    use std::fs::File;

    let args = vec![
        format!("foo"),
        // format!("map1"),
        // format!("act1"),
        // format!("--help"),
        // format!("--id"),
        // format!("10"),
        // format!("10a"),
        // format!("--nyom"),
        format!("--name"),
        format!("foo"),
        format!("--names"),
        // format!("foo"),
        // format!("foo"),
        format!("--help"),
    ];

    let ctx: TestCtx = serde_json::from_reader(File::open("config.json").expect("Failed to open"))
        .expect("Failed to parse");
    //
    let y = parse::<_, TestActs>(&ctx, &args);
    match y {
        Ok(r) => println!("{:?}", r),
        Err(r) => println!("{}", r),
    }

    let r = parse2::<_, TestArgs>(&ctx, &args);
    match r {
        Ok(r) => println!("{:?}", r),
        Err(r) => println!("{}", r),
    }

    let r = list::<_, TestActs>();
    println!("{:?}", r);

    Ok(())
}
