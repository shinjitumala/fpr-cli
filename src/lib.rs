use std::{borrow::BorrowMut, collections::HashMap, str::FromStr};

use cli_derive::*;
use itertools::Itertools;

mod test;

const PFX: &'static str = "-";
#[derive(Debug)]
struct Tkns {
    tkns: Vec<String>,
    consumed: bool,
}

impl Tkns {
    fn consume(&mut self, name: &'static str) {
        if self.consumed {
            panic!("Multiple consumers for '{}' (To developer)", name)
        }
        self.consumed = true;
    }
}

type ArgMap = HashMap<String, Tkns>;

pub fn to_argmap(args: &[String]) -> ArgMap {
    let mut res = ArgMap::new();

    let mut idx_arg_names: Vec<usize> = args
        .iter()
        .enumerate()
        .filter(|&(_, a)| a.starts_with(PFX))
        .map(|(i, _)| i)
        .collect();
    idx_arg_names.push(args.len());

    for (i, next) in idx_arg_names.iter().tuple_windows() {
        let x = &args[*i];

        let r = res.insert(
            x.to_owned(),
            Tkns {
                tkns: args[*i + 1..*next].to_owned(),
                consumed: false,
            },
        );
        match r {
            Some(_) => panic!("Duplicate argument: {}", x),
            None => (),
        }
    }
    res
}

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

trait SArg {
    type R;
    fn parse(name: &'static str, am: &mut ArgMap) -> Self::R;
}

impl<T: Parse> SArg for Option<T> {
    type R = Option<T>;
    fn parse(name: &'static str, am: &mut ArgMap) -> Self::R {
        match am.get_mut(&format!("{}{}", PFX, name)) {
            Some(v) => {
                v.consume(name);
                Option::Some(<Require<T> as SArg>::parse(name, am).v)
            }
            None => Option::None,
        }
    }
}

impl<T: Parse> SArg for Vec<T> {
    type R = Vec<T>;
    fn parse(name: &'static str, tkns: &mut ArgMap) -> Self::R {
        match tkns.get_mut(&format!("{}{}", PFX, name)) {
            Some(v) => {
                v.consume(name);
                v.tkns
                    .iter()
                    .map(|t| <T as Parse>::parse(name, t))
                    .collect()
            }
            None => panic!("Expected at least one value for '{}'.", name),
        }
    }
}

impl<T: Parse> SArg for Require<T> {
    type R = Require<T>;
    fn parse(name: &'static str, tkns: &mut ArgMap) -> Self::R {
        match tkns.get_mut(&format!("{}{}", PFX, name)) {
            Some(v) => {
                v.consume(name);
                if v.tkns.len() != 1 {
                    panic!("Expected single vlaue for '{}': {:?}", name, tkns)
                }
                Require {
                    v: <T as Parse>::parse(name, &v.tkns[0]),
                }
            }
            None => panic!("Argument '{}' is required.", name),
        }
    }
}

enum Desc<Ctx: Sized> {
    Static(&'static str),
    Dyn(fn(&Ctx) -> String),
}

struct Arg<Ctx: Sized, S: SArg> {
    desc: Desc<Ctx>,
    parse: fn(name: &'static str, tkns: &mut ArgMap) -> S::R,
}

impl<Ctx: Sized, R: SArg> Arg<Ctx, R> {
    fn new(desc: Desc<Ctx>) -> Self {
        Self {
            desc,
            parse: Self::parse,
        }
    }
    fn parse(name: &'static str, tkns: &mut ArgMap) -> R::R {
        <R as SArg>::parse(name, tkns)
    }
    fn desc(&self, c: &Ctx) -> String {
        match self.desc {
            Desc::Static(s) => s.to_owned(),
            Desc::Dyn(ref d) => d(c).to_owned(),
        }
    }
}
impl<Ctx: Sized, R: SArg<R = R>> SArg for Arg<Ctx, R> {
    type R = R;

    fn parse(name: &'static str, tkns: &mut ArgMap) -> Self::R {
        <R as SArg>::parse(name, tkns)
    }
}

struct FakeCtx {}

#[derive(SArg)]
struct Test2 {
    name: Arg<FakeCtx, Require<String>>,
}

#[derive(SArg)]
struct Test {
    name: Arg<FakeCtx, Require<i32>>,
    id: Arg<FakeCtx, Option<A>>,
    test: Test2,
}

struct A {
    name: String,
}
impl Parse for A {
    fn parse(name: &'static str, tkn: &String) -> Self {
        todo!()
    }
}
