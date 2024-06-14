use std::path::PathBuf;

use super::*;

impl Parse for i32 {
    fn parse(i: &Arg) -> Result<Self, ParseErr> {
        i32::from_str(i).map_err(|e| ParseErr {
            i: i.to_owned(),
            ty: Self::desc(),
            e: format!("{e}"),
        })
    }

    fn desc() -> &'static str {
        stringify!(i32)
    }
}
impl Parse for String {
    fn parse(i: &Arg) -> Result<Self, ParseErr> {
        String::from_str(i).map_err(|e| ParseErr {
            i: i.to_owned(),
            ty: Self::desc(),
            e: format!("{e}"),
        })
    }

    fn desc() -> &'static str {
        stringify!(String)
    }
}
#[derive(Clone, Debug)]
pub struct FileExist {
    pub p: PathBuf,
    pub s: String,
}

pub enum Parse2Err {
    ExpectedOne,
    Rquired,
    ExpectedAtLeastOne,
}
pub enum ArgParseErr {
    ParseErr(ParseErr),
    Parse2Err(Parse2Err),
}
impl From<ParseErr> for ArgParseErr {
    fn from(v: ParseErr) -> Self {
        ArgParseErr::ParseErr(v)
    }
}
impl From<Parse2Err> for ArgParseErr {
    fn from(v: Parse2Err) -> Self {
        ArgParseErr::Parse2Err(v)
    }
}
pub enum ArgsParseErr {
    UnexpectedToken(Arg),
    Help,
    UnknownArgs(Vec<Arg>),
    Arg(String, ArgParseErr),
}
pub enum ArgsErr {
    Run(String),
    Parse(ArgsParseErr),
}
impl From<ArgsParseErr> for ArgsErr {
    fn from(v: ArgsParseErr) -> Self {
        Self::Parse(v)
    }
}

fn file_exist(i: &String) -> Result<PathBuf, String> {
    let p = PathBuf::from_str(i).map_err(|e| e.to_string())?;
    if !p.exists() {
        return Err(format!("Does not exist"));
    };
    if !p.is_file() {
        return Err(format!("Not a file"));
    };
    Ok(p)
}

impl Parse for FileExist {
    fn parse(i: &String) -> Result<Self, ParseErr> {
        match file_exist(i) {
            Ok(p) => Ok(FileExist { p, s: i.to_owned() }),
            Err(e) => Err(ParseErr {
                i: i.to_owned(),
                ty: Self::desc(),
                e,
            }),
        }
    }

    fn desc() -> &'static str {
        stringify!(FileExist)
    }
}

#[derive(Clone, Debug)]
pub struct DirExist {
    pub p: PathBuf,
    pub s: String,
}

fn dir_exist(i: &String) -> Result<PathBuf, String> {
    let p = PathBuf::from_str(i).map_err(|e| e.to_string())?;
    if !p.exists() {
        return Err(format!("Does not exist"));
    };
    if !p.is_dir() {
        return Err(format!("Not a dir"));
    };
    Ok(p)
}

impl Parse for DirExist {
    fn parse(i: &String) -> Result<Self, ParseErr> {
        match dir_exist(i) {
            Ok(p) => Ok(DirExist { p, s: i.to_owned() }),
            Err(e) => Err(ParseErr {
                i: i.to_owned(),
                ty: Self::desc(),
                e,
            }),
        }
    }

    fn desc() -> &'static str {
        stringify!(DirExist)
    }
}

pub const PFX: &'static str = "--";
pub struct Key {
    pub i: usize,
    pub used: bool,
}
pub struct ParsedArgs<'b> {
    pub args: &'b [String],
    pub keys: Vec<Key>,
}

impl<'b> ParsedArgs<'b> {
    pub fn consume(&mut self, name: &'static str) -> Option<&[String]> {
        let (i, k) = self
            .keys
            .iter_mut()
            .enumerate()
            .find(|(_, k)| self.args[k.i] == name)?;
        assert!(
            !k.used,
            "A token should not ever be consumed twice. Probably a duplicate argument: {name}"
        );
        k.used = true;
        let idx = k.i + 1;
        Some(if i == self.keys.len() - 1 {
            &self.args[idx..]
        } else {
            &self.args[idx..self.keys[i + 1].i]
        })
    }
}

impl<'b> ParsedArgs<'b> {
    pub fn new(args: &[String]) -> Result<ParsedArgs, ArgsParseErr> {
        let mut end = false;
        let r = ParsedArgs {
            args,
            keys: args
                .iter()
                .enumerate()
                .filter(|&(_, a)| {
                    if !end {
                        true
                    } else {
                        let pfx = a.starts_with(PFX);
                        if pfx && a.len() == PFX.len() {
                            end = true;
                            false
                        } else {
                            true
                        }
                    }
                })
                .map(|(i, _)| Key { i, used: false })
                .collect(),
        };

        if !r.keys.is_empty() && r.keys[0].i != 0 {
            Err(ArgsParseErr::UnexpectedToken(args[0].to_owned()))
        } else {
            Ok(r)
        }
    }
}

pub enum Init<Ctx, T: Display> {
    None,
    Const(T),
    Dyn(fn(&Ctx) -> T),
}

impl<C, T: Display> Init<C, T> {
    fn get(self, c: &C) -> Option<T> {
        match self {
            Init::None => None,
            Init::Const(v) => Some(v),
            Init::Dyn(f) => Some(f(&c)),
        }
    }
    fn to_string(self, c: &C) -> String {
        match self.get(c) {
            Some(s) => format!(" (default: {s})"),
            None => format!(""),
        }
    }
}

pub trait Parse2<C>
where
    Self: Sized,
{
    type I;
    fn parse2(
        i: Init<C, Self::I>,
        k: &'static str,
        c: &C,
        p: &mut ParsedArgs,
    ) -> Result<Self, ArgParseErr>
    where
        Self::I: Display;
    fn desc2(i: Init<C, Self::I>, d: &'static str, k: &'static str, c: &C) -> [String; 4]
    where
        Self::I: Display;
}

impl<C, T: Parse + Display> Parse2<C> for T {
    type I = T;
    fn parse2(
        i: Init<C, Self::I>,
        k: &'static str,
        c: &C,
        p: &mut ParsedArgs,
    ) -> Result<Self, ArgParseErr> {
        match p.consume(k) {
            Some(args) => {
                if args.len() != 1 {
                    Err(Parse2Err::ExpectedOne)?
                } else {
                    Ok(T::parse(&args[0])?)
                }
            }
            None => Ok(i.get(c).ok_or(Parse2Err::Rquired)?),
        }
    }

    fn desc2(i: Init<C, Self>, d: &'static str, k: &'static str, c: &C) -> [String; 4] {
        [
            k.into(),
            format!("Req<{}>", T::desc()),
            d.into(),
            i.to_string(c),
        ]
    }
}

impl<Ctx, T: Parse + Display> Parse2<Ctx> for Option<T> {
    type I = T;
    fn parse2(
        i: Init<Ctx, T>,
        k: &'static str,
        c: &Ctx,
        p: &mut ParsedArgs,
    ) -> Result<Self, ArgParseErr> {
        match p.consume(k) {
            Some(args) => {
                if args.len() != 1 {
                    Err(Parse2Err::ExpectedOne)?
                } else {
                    Ok(Some(T::parse(&args[0])?))
                }
            }
            None => Ok(match i.get(c) {
                Some(e) => Some(e),
                None => None,
            }),
        }
    }
    fn desc2(i: Init<Ctx, T>, d: &'static str, k: &'static str, c: &Ctx) -> [String; 4] {
        [
            k.into(),
            format!("Opt<{}>", T::desc()),
            d.into(),
            i.to_string(c),
        ]
    }
}

impl<Ctx, T: Parse + Display> Parse2<Ctx> for Vec<T> {
    type I = DisplayVec<T>;
    fn parse2(
        i: Init<Ctx, Self::I>,
        k: &'static str,
        c: &Ctx,
        p: &mut ParsedArgs,
    ) -> Result<Self, ArgParseErr> {
        match p.consume(k) {
            Some(args) => {
                let args = args
                    .iter()
                    .map(|a| T::parse(a))
                    .collect::<Result<Vec<_>, _>>()?;
                if args.is_empty() {
                    Err(Parse2Err::ExpectedAtLeastOne)?
                }
                Ok(args)
            }
            None => Ok(match i.get(c) {
                Some(e) => e.into(),
                None => vec![],
            }),
        }
    }

    fn desc2(i: Init<Ctx, Self::I>, d: &'static str, k: &'static str, c: &Ctx) -> [String; 4] {
        [
            k.into(),
            format!("Vec<{}>", T::desc()),
            d.into(),
            i.to_string(c),
        ]
    }
}

pub struct OptVec<T: Display>(pub Vec<T>);
impl<T: Display> From<Vec<T>> for OptVec<T> {
    fn from(v: Vec<T>) -> Self {
        Self(v)
    }
}
impl<T: Display> From<DisplayVec<T>> for OptVec<T> {
    fn from(v: DisplayVec<T>) -> Self {
        Self(v.0)
    }
}
impl<Ctx, T: Parse + Display> Parse2<Ctx> for OptVec<T> {
    type I = DisplayVec<T>;
    fn parse2(
        i: Init<Ctx, Self::I>,
        k: &'static str,
        c: &Ctx,
        p: &mut ParsedArgs,
    ) -> Result<Self, ArgParseErr> {
        match p.consume(k) {
            Some(args) => {
                let args = args
                    .iter()
                    .map(|a| T::parse(a))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(args.into())
            }
            None => Ok(match i.get(c) {
                Some(e) => e.into(),
                None => vec![].into(),
            }),
        }
    }

    fn desc2(i: Init<Ctx, Self::I>, d: &'static str, k: &'static str, c: &Ctx) -> [String; 4] {
        [
            k.into(),
            format!("Vec<{}>", T::desc()),
            d.into(),
            i.to_string(c),
        ]
    }
}