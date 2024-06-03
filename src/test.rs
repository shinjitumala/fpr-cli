use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct TestCtx {
    account_id: i32,
}

#[derive(Args, Debug)]
#[args(ctx = TestCtx)]
#[allow(dead_code)]
struct TestArgs2 {
    #[arg(desc = ("2"), i = Init::None)]
    names: Vec<String>,
}

#[derive(Args, Debug)]
#[args(ctx = TestCtx)]
#[allow(dead_code)]
struct TestArgs {
    #[arg(desc = ("1"), i = Init::None)]
    name: String,
    a: TestArgs2,
}

fn noop<T>(_: &TestCtx, _: T) -> Res<()> {
    Ok(())
}

#[derive(Acts)]
#[acts(ctx = TestCtx, desc = "foo")]
#[allow(dead_code)]
struct TestActs2 {
    #[act(desc = "IAM LEGEND", act = noop)]
    act1: TestArgs,
}

#[derive(Acts)]
#[acts(ctx = TestCtx, desc = "foo")]
#[allow(dead_code)]
struct TestActs {
    map1: TestActs2,
    #[act(desc = "IAM LEGEND", act = noop)]
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
