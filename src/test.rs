use super::*;

struct Ctx {}
use smart_default::SmartDefault;

// type Req<T: Parse> = Arg<Ctx, super::Req<One<T>>>;

#[derive(Args, SmartDefault)]
// #[derive(Args)]
#[ctx(Ctx)]
struct Test2 {
    #[default(_code = "Arg::s(\"foo\")")]
    next: Arg<Ctx, Opt<One<String>>>,
}

#[derive(Args, SmartDefault)]
// #[derive(Args)]
#[ctx(Ctx)]
struct Test {
    // #[default(_code = "Arg::d(|c| format!(\"FOO\"))")]
    #[default(_code = "Arg::new(Desc::Dyn(|c| format!(\"FOO\")),Init::Const(1))")]
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
        format!("-id"),
        format!("foo"),
        // format!("-help"),
        // format!("-name"),
        // format!("10"),
    ];

    let ctx = Ctx {};

    let x = parse::<Ctx, Test>(&ctx, &args).expect("Parse failed.");
    println!("{:?}", x);
}
