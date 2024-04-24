use super::*;

struct Ctx {}

#[derive(SArg)]
struct Test2 {
    next: Arg<Ctx, Option<String>>,
}

#[derive(SArg)]
struct Test {
    name: Arg<Ctx, Require<i32>>,
    id: Arg<Ctx, Option<A>>,
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
        // format!("c"),
        format!("-name"),
        // format!("10"),
    ];

    let x = parse::<Test>(&args);
    println!("{:?}", x);
}
