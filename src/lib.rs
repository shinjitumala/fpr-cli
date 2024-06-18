mod parse;

use inquire::{list_option::ListOption, InquireError, Select};
use itertools::Itertools;
use std::{env::args, fmt::Display, path::PathBuf, str::FromStr};

pub use parse::*;

pub enum ActsErr {
    Run(ParseCtx, String),
    Inquire(String),
    ExpectedAct(ParseCtx, String),
    UnknownAct(ParseCtx, String),
    Args(ParseCtx, ArgsParseErr, String),
}

#[derive(Clone, Debug)]
pub struct ParseCtx {
    pub pfx: Vec<Arg>,
}

impl ActsErr {
    fn display<'a>(self, arg0: &'a Arg) -> DActsErr<'a> {
        DActsErr { e: self, arg0 }
    }
}
struct DActsErr<'a> {
    e: ActsErr,
    arg0: &'a Arg,
}

impl Display for ActsErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ActsErr::*;
        match self {
            Run(_, ref e) => write!(f, "Failed to run:\n{e}"),
            Inquire(ref e) => write!(f, "{e}"),
            ExpectedAct(_, _) => write!(f, "Expected an act.\n"),
            UnknownAct(_, ref e) => write!(f, "Unknown act '{e}.'\n"),
            Args(_, ref e, _) => match e {
                ArgsParseErr::Help(_) => write!(f, "{e}"),
                _ => write!(f, "Failed to parse opts.\n{e}\n"),
            },
        }
    }
}
impl<'a> Display for DActsErr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ActsErr::*;
        write!(f, "{}", self.e)?;
        match self.e {
            ExpectedAct(ref c, ref u) | UnknownAct(ref c, ref u) => {
                write!(f, "Usage: {} {} <action>\n{u}", self.arg0, c.pfx.join(" "))?;
            }
            Args(ref c, _, ref u) => {
                write!(
                    f,
                    "Usage: {} {} <opts...>\nOpts:\n{u}",
                    self.arg0,
                    c.pfx.join(" ")
                )?;
            }
            _ => (),
        };
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
        Self::next(c, &mut s, &a[1..]).map_err(|e| format!("{}", e.display(&a[0])))
    }

    fn next(c: &C, s: &mut ParseCtx, args: &[Arg]) -> Result<(), ActsErr> {
        if args.is_empty() {
            print!("{}", ActsErr::ExpectedAct(s.to_owned(), Self::usage()));
            return Self::next(c, s, &[Self::select_act()?.to_owned()]);
        };
        let a = &args[0];
        let args = &args[1..];

        use ActsErr::*;
        match Self::next_impl(c, s, a, args) {
            Err(e) => match e {
                UnknownAct(_, _) => {
                    print!("{e}");
                    Self::next(c, s, &[Self::select_act()?.to_owned()])
                }
                _ => return Err(e),
            },
            e => return e,
        }
    }
    fn select_act() -> Result<&'static str, ActsErr> {
        let opts: Vec<_> = to_lines(&Self::usage_v())
            .into_iter()
            .enumerate()
            .map(|(i, o)| ListOption::new(i, o))
            .collect();
        Ok(Self::opts()[Select::new("Choose an action.", opts)
            .with_page_size(50)
            .prompt()?
            .index])
    }

    fn list() -> Vec<Vec<String>> {
        let pfx = vec![];
        let mut res: Vec<Vec<String>> = vec![];
        Self::add_paths(&pfx, &mut res);
        return res;
    }
    fn usage() -> String {
        to_table(&Self::usage_v())
    }

    fn opts() -> Vec<&'static str>;
    fn next_impl(c: &C, s: &mut ParseCtx, a: &Arg, args: &[Arg]) -> Result<(), ActsErr>;
    fn desc_act() -> &'static str;
    fn usage_v() -> Vec<[&'static str; 2]>;
    fn add_paths(pfx: &Vec<String>, p: &mut Vec<Vec<String>>);
}
pub trait Args<C>: Run<C> + Sized {
    fn next_impl(c: &C, args: &[Arg]) -> Result<(), ArgsErr> {
        let mut args = ParsedArgs::new(args).map_err(|e| {
            use ParsedArgsErr::*;
            match e {
                UnexpectedToken(a) => ArgsParseErr::UnexpectedToken(a, Self::usage(c)),
            }
        })?;

        if args.consume(&format!("{PFX}help")).is_some() {
            return Err(ArgsParseErr::Help(Self::usage(c)).into());
        }

        let a = Self::new(c, &mut args)?;

        let u = args
            .keys
            .iter()
            .filter(|k| !k.used)
            .map(|k| args.args[k.i].to_owned())
            .collect::<Vec<_>>();
        if !u.is_empty() {
            return Err(ArgsParseErr::UnknownArgs(u, Self::usage(c)).into());
        }

        let _ = Self::run(c, a).map_err(|s| ArgsErr::Run(s))?;
        Ok(())
    }
    fn next(c: &C, s: &mut ParseCtx, args: &[Arg]) -> Result<(), ActsErr> {
        match Self::next_impl(c, args) {
            Err(e) => match e {
                ArgsErr::Run(r) => Err(ActsErr::Run(s.to_owned(), r)),
                ArgsErr::Parse(e) => Err(ActsErr::Args(s.to_owned(), e, Self::usage(c))),
            },
            Ok(o) => Ok(o),
        }
    }
    fn usage(c: &C) -> String {
        let mut r: Vec<[String; 4]> = vec![];
        Self::add_usage(c, &mut r);
        to_table(&r)
    }
    fn new(c: &C, args: &mut ParsedArgs) -> Result<Self, ArgsParseErr>;
    fn desc_act() -> &'static str;
    fn add_paths(pfx: &Vec<String>, p: &mut Vec<Vec<String>>);
    fn add_usage(c: &C, r: &mut Vec<[String; 4]>);
    fn default(c: &C) -> Self;
}
pub trait Run<C> {
    type R;
    fn run(c: &C, a: Self) -> Result<Self::R, String>;
}

pub trait Parse
where
    Self: Sized + Display,
{
    fn parse(i: &Arg) -> Result<Self, ParseErr>;
    fn desc() -> &'static str;
}
#[derive(Debug)]
pub struct ParseErr {
    pub ty: &'static str,
    pub i: Arg,
    pub e: String,
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

pub fn to_lines<const S: usize, I: AsRef<str>>(d: &Vec<[I; S]>) -> Vec<String> {
    use unicode_width::*;
    let w: [usize; S] =
        std::array::from_fn(|i| i).map(|i| d.iter().map(|l| l[i].as_ref().width()).max().unwrap());
    d.into_iter()
        .map(|v| {
            v.iter()
                .enumerate()
                .map(|(i, s)| format!("{}{: <2$}", s.as_ref(), "", w[i] - s.as_ref().width()))
                .join(" ")
        })
        .collect()
}
pub fn to_table<const S: usize, I: AsRef<str>>(d: &Vec<[I; S]>) -> String {
    to_lines(d).join("\n")
}

#[derive(Clone, Debug, Default)]
pub struct FileExist {
    pub p: PathBuf,
    pub s: String,
}
#[derive(Clone, Debug, Default)]
pub struct DirExist {
    pub p: PathBuf,
    pub s: String,
}
