mod i;
mod parse;
mod util;

mod com {
    pub use crate::*;
    pub use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
    pub use inquire::{
        autocompletion::Replacement, list_option::ListOption, validator::CustomTypeValidator,
        validator::ErrorMessage, Autocomplete, CustomType, CustomUserError, MultiSelect, Select,
        Text,
    };
    pub use itertools::Itertools;
    pub use std::{env::args, fmt::Display, path::PathBuf, str::FromStr};
    pub use thiserror::Error;
}

pub use util::*;

pub use fpr_cli_derives::*;
pub use i::*;
pub use parse::*;

use com::*;

#[derive(Clone, Debug)]
pub struct ParseCtx<'a> {
    pub exec: Arg<'a>,
    pub pfx: Vec<Arg<'a>>,
}

#[derive(Error, Debug)]
pub enum ErrActs {
    #[error("Needs help")]
    Help,
    #[error("Unknown act '{act}'")]
    Unknown { act: String },
    #[error("Expected act.")]
    Missing,
    #[error("{e}")]
    Args {
        #[from]
        e: ErrArgs,
    },
}
pub type Arg<'a> = &'a str;
pub trait Acts<C>: Sized {
    fn run(c: &C) -> Result<(), ErrActs> {
        let args = args().collect_vec();
        let a: Vec<_> = args.iter().map(|e| e.as_str()).collect();
        let mut s = ParseCtx {
            exec: a[0],
            pfx: vec![],
        };
        let r = Self::next(c, &mut s, &a[1..]);
        if r.is_err() {
            println!(
                "Use this command for help:\n{} --help",
                [s.exec].iter().chain(&s.pfx).join(" ")
            );
        }
        r
    }

    fn next<'a>(c: &C, s: &mut ParseCtx<'a>, args: &[Arg<'a>]) -> Result<(), ErrActs> {
        if args.is_empty() {
            return Err(ErrActs::Missing)?;
        };
        let a = &args[0];
        let args = &args[1..];
        match Self::next_impl(c, s, a, args) {
            Ok(_) => Ok(()),
            Err(e) => match e {
                ErrActs::Help => {
                    println!("{}", Self::full_usage(s.exec, &s.pfx));
                    Ok(())
                }
                e => Err(e),
            },
        }
    }
    fn list<'a>() -> Vec<Vec<Arg<'a>>> {
        let pfx = vec![];
        let mut res: Vec<Vec<Arg<'a>>> = vec![];
        Self::add_paths(&pfx, &mut res);
        return res;
    }
    fn usage() -> String {
        to_table(&Self::usage_v())
    }
    fn full_usage(exec: &str, pfx: &[&str]) -> String {
        format!(
            "Usage: {}\nActs:\n{}",
            [exec].iter().chain(pfx).chain(&["<act>"]).join(" "),
            Self::usage()
        )
    }

    fn opts() -> Vec<&'static str>;
    fn next_impl<'a>(
        c: &C,
        s: &mut ParseCtx<'a>,
        a: &Arg<'a>,
        args: &[Arg<'a>],
    ) -> Result<(), ErrActs>;
    fn desc_act() -> &'static str;
    fn usage_v() -> Vec<[&'static str; 2]>;
    fn add_paths<'a>(pfx: &Vec<Arg<'a>>, p: &mut Vec<Vec<Arg<'a>>>);
}

#[derive(Error, Debug)]
pub enum ErrArgs {
    #[error("Needs help")]
    Help,
    #[error("Unknown args '{}'", arg.join(", "))]
    Unknown { arg: Vec<String> },
    #[error("Failed while parsing arg '{arg}': '{e}'")]
    Arg { arg: String, e: ErrArg },
    #[error("{e}")]
    Run { e: String },
}
pub trait Args<C>: Sized {
    fn next_impl<'a>(c: &C, args: &[Arg<'a>]) -> Result<(), ErrArgs> {
        let mut args = ParsedArgs::new(args);
        if args.consume(&format!("{PFX}help")).is_some() {
            return Err(ErrArgs::Help);
        }

        let a = Self::new(c, &mut args)?;

        let u = args
            .k
            .iter()
            .filter(|k| !k.used && !k.k.is_none_or(|x| x == PFX))
            .map(|k| {
                match k.k {
                    Some(e) => vec![e],
                    None => vec![],
                }
                .into_iter()
                .chain(k.v.iter().map(|e| *e))
            })
            .flatten()
            .map(|e| e.to_owned())
            .collect_vec();
        if !u.is_empty() {
            return Err(ErrArgs::Unknown { arg: u });
        }

        let r = args
            .k
            .iter()
            .filter(|k| k.k.is_none_or(|x| x == PFX))
            .map(|k| k.v.iter())
            .flatten()
            .map(|e| *e)
            .collect();

        let _ = a.run(c, r).map_err(|e| ErrArgs::Run { e })?;
        Ok(())
    }
    fn next<'a>(c: &C, s: &mut ParseCtx<'a>, args: &[Arg<'a>]) -> Result<(), ErrArgs> {
        match Self::next_impl(c, args) {
            Err(e) => match e {
                ErrArgs::Help => {
                    println!("{}", Self::full_usage(c, s.exec, &s.pfx));
                    Ok(())
                }
                e => Err(e),
            },
            Ok(_) => Ok(()),
        }
    }
    fn usage(c: &C) -> String {
        let mut r: Vec<[String; 4]> = vec![];
        Self::add_usage(c, &mut r);
        to_table(&r)
    }
    fn full_usage(c: &C, exec: &str, pfx: &[&str]) -> String {
        let u = Self::usage(c);
        let has_positional = u.split("\n").any(|e| !e.starts_with(PFX));
        let v = if !has_positional {
            [exec].iter().chain(pfx).chain(&["<opts...>"]).join(" ")
        } else {
            [exec]
                .iter()
                .chain(pfx)
                .chain(&["<positional...>", "<opts...>", "--", "<positional...>"])
                .join(" ")
        };
        format!("Usage: {v}\nOptions:\n{u}")
    }
    fn new<'a, 'b>(c: &C, args: &mut ParsedArgs<'b>) -> Result<Self, ErrArgs>;
    fn desc_act() -> &'static str;
    fn add_paths<'a>(pfx: &Vec<Arg<'a>>, p: &mut Vec<Vec<Arg<'a>>>);
    fn add_usage(c: &C, r: &mut Vec<[String; 4]>);
    fn default(c: &C) -> Self;

    fn run(self, c: &C, r: Vec<&str>) -> Result<(), String>;
}

#[derive(Error, Debug)]
pub enum ErrVal {
    #[error("Failed to parse '{input}' as type '{typename}': {error}")]
    Err {
        input: String,
        typename: String,
        error: String,
    },
}
pub trait Parse<'a>
where
    Self: Sized + Display,
{
    fn parse(i: Arg<'a>) -> Result<Self, ErrVal>;
    fn desc() -> &'static str;
}
#[derive(Debug)]
pub struct ParseErr<'a> {
    pub ty: &'static str,
    pub i: Arg<'a>,
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
