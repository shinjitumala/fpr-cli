use crate::cl2::*;
use smart_default::SmartDefault;

struct Ctx {}

#[derive(Args, SmartDefault)]
#[ctx(Ctx)]
struct Test2 {
    #[default(_code = "Arg::s(\"foo\")")]
    next: Arg<Ctx, Opt<One<String>>>,
}

#[derive(Args, SmartDefault)]
#[ctx(Ctx)]
struct Test {
    #[default(_code = "Arg::new(Desc::Dyn(|_| format!(\"FOO\")),Init::Const(1))")]
    name: Arg<Ctx, Req<One<i32>>>,
    #[default(_code = "Arg::s(\"foo\")")]
    id: Arg<Ctx, Opt<One<A>>>,
    #[default(_code = "Arg::s(\"focwo\")")]
    ids: Arg<Ctx, Opt<Vec<i32>>>,
    test: Test2,
}

#[derive(Debug, Clone)]
struct A {
    name: String,
}
impl Parse for A {
    fn parse(_name: &'static str, tkn: &String) -> ParseResult<Self> {
        Ok(Self {
            name: tkn.to_owned(),
        })
    }
}

#[test]
fn test() {
    let args = vec![
        // format!("-id"),
        // format!("foo"),
        // format!("-help"),
        // format!("-name"),
        // format!("10"),
        // format!("10a"),
        format!("-nyom"),
    ];

    let ctx = Ctx {};

    let x = parse::<Ctx, Test>(&ctx, &args).expect("Parse failed");
    println!("{:?}", x);
}
