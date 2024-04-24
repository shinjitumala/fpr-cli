use regex::Regex;
use std::{collections::HashMap, process, str::FromStr};

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

fn add_prefix(name: &'static str) -> String {
    format!("{}{}", PFX, name)
}

pub fn to_argmap(args: &[String]) -> ArgMap {
    let remove_prefix = Regex::new(&format!("^{}", PFX)).unwrap();

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
            remove_prefix.replace_all(x, "").to_string(),
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
        match am.get_mut(name) {
            Some(v) => {
                v.consume(name);
                Option::Some(<Require<T> as SArg>::parse(name, am))
            }
            None => Option::None,
        }
    }
}

impl<T: Parse> SArg for Vec<T> {
    type R = Vec<T>;
    fn parse(name: &'static str, tkns: &mut ArgMap) -> Self::R {
        match tkns.get_mut(name) {
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
    type R = T;
    fn parse(name: &'static str, am: &mut ArgMap) -> Self::R {
        match am.get_mut(name) {
            Some(v) => {
                v.consume(name);
                if v.tkns.len() != 1 {
                    panic!("Expected single vlaue for '{}': {:?}", name, v.tkns)
                }
                <T as Parse>::parse(name, &v.tkns[0])
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
    fn desc(&self, name: &'static str, c: &Ctx) -> Vec<(String, String)> {
        let d = match self.desc {
            Desc::Static(s) => s.to_owned(),
            Desc::Dyn(ref d) => d(c).to_owned(),
        };
        vec![(name.to_string(), d)]
    }
}
impl<Ctx: Sized, R: SArg> SArg for Arg<Ctx, R> {
    type R = <R as SArg>::R;

    fn parse(name: &'static str, tkns: &mut ArgMap) -> Self::R {
        <R as SArg>::parse(name, tkns)
    }
}

trait DArg<Ctx: Sized> {
    fn desc(&self, name: &'static str, c: &Ctx) -> Vec<(String, String)>;
}

impl<Ctx: Sized, S: SArg> DArg<Ctx> for Arg<Ctx, S> {
    fn desc(&self, name: &'static str, c: &Ctx) -> Vec<(String, String)> {
        let d = match self.desc {
            Desc::Static(ref a) => a.to_string(),
            Desc::Dyn(ref f) => f(c),
        };
        vec![(name.to_string(), d)]
    }
}

pub fn parse<A: SArg>(args: &[String]) -> <A as SArg>::R {
    let mut am = to_argmap(&args);

    match am.get("help") {
        Some(_) => {
            process::exit(0);
        }
        None => (),
    }

    let r = <A as SArg>::parse("", &mut am);

    let errs: Vec<String> = am
        .iter()
        .filter(|a| a.1.consumed == false)
        .map(|a| format!("Unknown argument: {}", a.0))
        .collect();

    if !errs.is_empty() {
        panic!("Parse error:\n{}", errs.join("\n"))
    }

    r
}
