use std::str::FromStr;

use itertools::Itertools;

mod test;

const PFX: &'static str = "-";

trait Parse {
    fn parse(name: &'static str, tkn: &String) -> Self;
}

impl<T: FromStr> Parse for T {
    fn parse(name: &'static str, tkn: &String) -> Self {
        match Self::from_str(tkn) {
            Ok(v) => v,
            Err(_) => panic!("Failed to parse '{}': {}", name, tkn),
        }
    }
}

struct Require<T: Parse> {
    v: T,
}

trait SArg<Res> {
    fn parse(name: &'static str, tkns: Option<&[String]>) -> Res;
}

impl<T: Parse> SArg<Option<T>> for Option<T> {
    fn parse(name: &'static str, tkns: Option<&[String]>) -> Self {
        todo!()
    }
}

impl<T: Parse> SArg<Vec<T>> for Vec<T> {
    fn parse(name: &'static str, tkns: Option<&[String]>) -> Vec<T> {
        todo!()
    }
}

impl<T: Parse> SArg<Require<T>> for Require<T> {
    fn parse(name: &'static str, tkns: Option<&[String]>) -> Require<T> {
        todo!()
    }
}

enum Desc<Ctx: Sized> {
    Static(&'static str),
    Dyn(fn(&Ctx) -> String),
}

struct Arg<Ctx: Sized, S: SArg<S>> {
    desc: Desc<Ctx>,
    parse: fn(name: &'static str, tkns: Option<&[String]>) -> S,
}

impl<Ctx: Sized, Res: SArg<Res>> Arg<Ctx, Res> {
    fn new(desc: Desc<Ctx>) -> Self {
        Self {
            desc,
            parse: Self::parse,
        }
    }
    fn parse(name: &'static str, tkns: Option<&[String]>) -> Res {
        <Res as SArg<Res>>::parse(name, tkns)
    }
    fn desc(&self, c: &Ctx) -> String {
        match self.desc {
            Desc::Static(s) => s.to_owned(),
            Desc::Dyn(ref d) => d(c).to_owned(),
        }
    }
}

struct FakeCtx {}

struct Test2 {}

struct Test {
    name: Arg<FakeCtx, Require<i32>>,
    id: Arg<FakeCtx, Option<A>>,
}

struct A {
    name: String,
}
impl Parse for A {
    fn parse(name: &'static str, tkn: &String) -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub struct ParsedArg {
    name: String,
    values: Vec<String>,
}

pub fn to_tokens(args: &[String]) -> Vec<ParsedArg> {
    let mut res = Vec::<ParsedArg>::new();

    let mut idx_arg_names: Vec<usize> = args
        .iter()
        .enumerate()
        .filter(|&(_, a)| a.starts_with(PFX))
        .map(|(i, _)| i)
        .collect();
    idx_arg_names.push(args.len());

    for (i, next) in idx_arg_names.iter().tuple_windows() {
        println!("{:?}", i);
        let x = &args[*i];
        res.push(ParsedArg {
            name: x.to_owned(),
            values: args[*i + 1..*next].to_owned(),
        });
    }
    res
}
