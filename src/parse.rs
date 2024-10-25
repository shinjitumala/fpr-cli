use crate::com::*;

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
    UnexpectedToken(Arg, String),
    Help(String),
    UnknownArgs(Vec<Arg>, String),
    Arg(String, ArgParseErr, String),
}
impl Display for ArgsParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ArgsParseErr::*;
        match self {
            UnexpectedToken(ref a, _) => write!(f, "Unexpected token '{a}'")?,
            Help(_) => (),
            UnknownArgs(ref a, _) => write!(
                f,
                "Unknown options '{}'",
                a.into_iter().map(|a| format!(r#""{a}""#)).join(", ")
            )?,
            Arg(ref a, ref e, _) => write!(f, "Error parsing option '{a}.'\n{e}")?,
        };
        Ok(())
    }
}
impl Display for ArgParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ArgParseErr::*;
        match self {
            ParseErr(e) => write!(f, "{e}"),
            Parse2Err(e) => write!(f, "{e}"),
        }
    }
}
impl Display for ParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to parse '{}' as '{}' because '{}'",
            self.i, self.ty, self.e
        )
    }
}
impl Display for Parse2Err {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Parse2Err::*;
        match self {
            ExpectedOne => write!(f, "Expected one value."),
            Rquired => write!(f, "Required."),
            ExpectedAtLeastOne => write!(f, "Expected one value minimum."),
        }
    }
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

pub const PFX: &'static str = "--";
#[derive(Debug)]
pub struct Key {
    pub i: usize,
    pub used: bool,
}
#[derive(Debug)]
pub struct ParsedArgs<'b> {
    pub args: &'b [String],
    pub keys: Vec<Key>,
}

impl<'b> ParsedArgs<'b> {
    pub fn consume(&mut self, name: &str) -> Option<&[String]> {
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
        } else if self.args[self.keys[i + 1].i] == PFX {
            &self.args[self.keys[i + 1].i + 1..]
        } else {
            &self.args[idx..self.keys[i + 1].i]
        })
    }
}

pub enum ParsedArgsErr {
    UnexpectedToken(Arg),
}
impl<'b> ParsedArgs<'b> {
    pub fn new(args: &[String]) -> Result<ParsedArgs, ParsedArgsErr> {
        let mut end = false;
        let r = ParsedArgs {
            args,
            keys: args
                .iter()
                .enumerate()
                .filter(|&(_, a)| {
                    if end {
                        false
                    } else {
                        let pfx = a.starts_with(PFX);
                        if pfx && a.len() == PFX.len() {
                            end = true;
                            true
                        } else {
                            pfx
                        }
                    }
                })
                .map(|(i, _)| Key { i, used: false })
                .collect(),
        };

        if !r.keys.is_empty() && r.keys[0].i != 0 {
            Err(ParsedArgsErr::UnexpectedToken(args[0].to_owned()))
        } else {
            Ok(r)
        }
    }
}

pub enum Init<C, T: Display> {
    None,
    Const(T),
    Dyn(fn(&C) -> T),
}

impl<C, T: Display> Init<C, T> {
    pub fn get(self, c: &C) -> Option<T> {
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
    Self::I: Display,
{
    type I;
    fn parse2(
        i: Init<C, Self::I>,
        k: &'static str,
        c: &C,
        p: &mut ParsedArgs,
    ) -> Result<Self, ArgParseErr>;
    fn desc2(i: Init<C, Self::I>, d: &'static str, k: &'static str, c: &C) -> [String; 4];
    fn default2(c: &C, i: Init<C, Self::I>) -> Self;
}

impl<C, T: Parse + Default> Parse2<C> for T {
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
    fn default2(c: &C, i: Init<C, Self::I>) -> Self {
        i.get(c).unwrap_or(Self::default())
    }
}

impl<Ctx, T: Parse> Parse2<Ctx> for Option<T> {
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
    fn default2(c: &Ctx, i: Init<Ctx, Self::I>) -> Self {
        match i.get(c) {
            Some(i) => Some(i),
            None => None,
        }
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
    fn default2(c: &Ctx, i: Init<Ctx, Self::I>) -> Self {
        match i.get(c) {
            Some(v) => v.0,
            None => Self::default(),
        }
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
impl<Ctx, T: Parse> Parse2<Ctx> for OptVec<T> {
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
    fn default2(c: &Ctx, i: Init<Ctx, Self::I>) -> Self {
        Self::from(match i.get(c) {
            Some(v) => v.0,
            None => <Vec<T>>::default(),
        })
    }
}
impl Display for DirExist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.s)
    }
}
impl Display for FileExist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.s)
    }
}
