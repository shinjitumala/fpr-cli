use super::*;

struct Ctx {}
use smart_default::SmartDefault;

#[derive(SArg, SmartDefault)]
#[ctx(Ctx)]
struct Test2 {
    #[default(_code = "Arg::new(Desc::Static(\"foo\"))")]
    next: Arg<Ctx, Option<String>>,
}

#[derive(SArg, SmartDefault)]
#[ctx(Ctx)]
struct Test {
    #[default(_code = "Arg::new(Desc::Static(\"foo\"))")]
    name: Arg<Ctx, Require<i32>>,
    #[default(_code = "Arg::new(Desc::Static(\"foo\"))")]
    id: Arg<Ctx, Option<A>>,
    #[default(_code = "Arg::new(Desc::Static(\"focwo\"))")]
    ids: Arg<Ctx, Vec<i32>>,
    test: Test2,
}

#[derive(Debug)]
struct A {
    name: String,
}
impl Parse for A {
    fn parse(name: &'static str, tkn: &String) -> Self {
        todo!()
    }
}

#[test]
fn test() {
    let args = vec![
        // format!("-aaa"),
        // format!("a"),
        // format!("b"),
        // format!("c"),
        // format!("-foo"),
        // format!("bar"),
        // format!("-baz"),
        // format!("-nyom"),
        // format!("-baa"),
        // format!("a"),
        // format!("b"),
        format!("-help"),
        format!("-name"),
        format!("10"),
    ];

    let ctx = Ctx {};

    let x = parse::<Ctx, Test>(&ctx, &args);
    println!("{:?}", x);

    let y = Test2 {
        next: Arg::<Ctx, Option<String>>::new(Desc::Static("DESCRIPTION")),
    };

    for a in y.desc("", &Ctx {}) {
        println!("{} {}", a.0, a.1);
    }

    // x.desc(&Ctx{})
}
