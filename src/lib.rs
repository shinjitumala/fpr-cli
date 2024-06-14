mod parse;
mod test;

use constcat::concat;
use inquire::{InquireError, Select};
use itertools::Itertools;
use std::{env::args, fmt::Display, str::FromStr};

pub use derives::*;
use parse::*;

pub enum ActsErr {
    Run(ParseCtx, String),
    Inquire(String),
    ExpectedAct(ParseCtx),
    UnknownAct(ParseCtx, String),
    Args(ParseCtx, ArgsParseErr),
}

#[derive(Clone, Debug)]
pub struct ParseCtx {
    pfx: Vec<Arg>,
}

impl Display for ActsErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ActsErr::*;
        write!(
            f,
            "{}",
            match &self {
                Run(_, e) => format!("Run error '{e}'"),
                Inquire(e) => format!("{e}"),
                ExpectedAct(_) => format!("Expected an act"),
                UnknownAct(_, e) => format!("Unknown act '{e}'"),
                Args(_, _) => todo!(),
            }
        )?;
        Ok(())
    }
}
impl From<InquireError> for ActsErr {
    fn from(v: InquireError) -> Self {
        Self::Inquire(format!("{v}"))
    }
}

pub type Arg = String;
pub trait Acts<C>: Sized {
    fn run(c: &C) -> Result<(), String> {
        let a: Vec<_> = args().collect();
        let mut s = ParseCtx { pfx: vec![] };
        Self::next(c, &mut s, &a[1..]).map_err(|e| format!("{e}"))
    }

    fn next(c: &C, s: &mut ParseCtx, args: &[Arg]) -> Result<(), ActsErr> {
        if args.is_empty() {
            return Err(ActsErr::ExpectedAct(s.to_owned()));
        };
        let a = &args[0];
        let args = &args[1..];

        use ActsErr::*;
        match Self::next_impl(c, s, a, args) {
            Err(e) => match e {
                UnknownAct(_, ref a) => {
                    println!("{e}");
                    let a = Select::new("Choose an action.", Self::opts())
                        .with_starting_filter_input(&a)
                        .prompt()?;
                    Self::next_impl(c, s, &a.into(), args)
                }
                _ => return Err(e),
            },
            e => return e,
        }
    }

    fn list() -> Vec<Vec<String>> {
        let mut pfx = vec![];
        let mut res: Vec<Vec<String>> = vec![];
        Self::add_paths(&mut pfx, &mut res);
        return res;
    }

    fn opts() -> Vec<&'static str>;
    fn next_impl(c: &C, s: &mut ParseCtx, a: &Arg, args: &[Arg]) -> Result<(), ActsErr>;
    fn desc_act() -> &'static str;
    fn usage() -> Vec<[&'static str; 2]>;
    fn add_paths(pfx: &mut Vec<String>, p: &mut Vec<Vec<String>>);
}
pub trait Args<C>: Run<C> + Sized {
    fn next_impl(c: &C, args: &[Arg]) -> Result<(), ArgsErr> {
        let mut args = ParsedArgs::new(args)?;

        if args.consume(concat!(PFX, "help")).is_some() {
            return Err(ArgsParseErr::Help.into());
        }

        let u = args
            .keys
            .iter()
            .filter(|k| !k.used)
            .map(|k| args.args[k.i].to_owned())
            .collect::<Vec<_>>();
        if !u.is_empty() {
            return Err(ArgsParseErr::UnknownArgs(u).into());
        }

        let _ = Self::run(c, Self::new(c, &mut args)?).map_err(|s| ArgsErr::Run(s))?;
        Ok(())
    }
    fn next(c: &C, s: &mut ParseCtx, args: &[Arg]) -> Result<(), ActsErr> {
        match Self::next_impl(c, args) {
            Err(e) => match e {
                ArgsErr::Run(r) => Err(ActsErr::Run(s.to_owned(), r)),
                ArgsErr::Parse(e) => Err(ActsErr::Args(s.to_owned(), e)),
            },
            Ok(o) => Ok(o),
        }
    }
    fn usage(c: &C) -> Vec<[String; 4]> {
        let mut r: Vec<[String; 4]> = vec![];
        Self::add_usage(c, &mut r);
        r
    }
    fn new(c: &C, args: &mut ParsedArgs) -> Result<Self, ArgsParseErr>;
    fn desc_act() -> &'static str;
    fn add_paths(pfx: &mut Vec<String>, p: &mut Vec<Vec<String>>);
    fn add_usage(c: &C, r: &mut Vec<[String; 4]>);
}
pub trait Run<C> {
    type R;
    fn run(c: &C, a: Self) -> Result<Self::R, String>;
}

pub trait Parse
where
    Self: Sized,
{
    fn parse(i: &Arg) -> Result<Self, ParseErr>;
    fn desc() -> &'static str;
}
pub struct ParseErr {
    ty: &'static str,
    i: Arg,
    e: String,
}

pub struct DisplayVec<T: Display>(Vec<T>);
impl<T: Display> Display for DisplayVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().map(|t| format!("{t}")).join(", "))
    }
}
impl<T: Display> From<Vec<T>> for DisplayVec<T> {
    fn from(v: Vec<T>) -> Self {
        Self(v)
    }
}
impl<T: Display> Into<Vec<T>> for DisplayVec<T> {
    fn into(self) -> Vec<T> {
        self.0
    }
}

// pub fn print_table(d: &Vec<Vec<String>>) -> String {
//     if d.is_empty() {
//         return format!("");
//     };
//     let w = d[0]
//         .iter()
//         .enumerate()
//         .map(|(i, _)| {
//             d.iter()
//                 .map(|l| l[i].width())
//                 .max()
//                 .expect("Data should not be empty")
//         })
//         .collect::<Vec<_>>();
//
//     d.iter()
//         .map(|v| {
//             v.iter()
//                 .enumerate()
//                 .map(|(i, s)| format!("{}{: <2$}", s, "", w[i] - s.width()))
//                 .join(" ")
//         })
//         .join("\n")
// }
//
// pub const LIST_SEP: &'static str = "_";
//
// pub fn parse<Ctx, A: Acts<Ctx>>(c: &Ctx, args: &[String]) -> Res<()> {
//     A::parse(c, Option::None, args)
// }
//
// pub fn list<Ctx, A: Acts<Ctx>>() -> Vec<Vec<String>> {
//     A::list(&vec![])
// }
