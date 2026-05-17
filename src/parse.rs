use crate::com::*;

pub enum Parse2Err {
    ExpectedOne,
    Rquired,
    ExpectedAtLeastOne,
}
pub enum ArgParseErr<'a> {
    ParseErr(ParseErr<'a>),
    Parse2Err(Parse2Err),
}
impl<'a> From<ParseErr<'a>> for ArgParseErr<'a> {
    fn from(v: ParseErr<'a>) -> Self {
        ArgParseErr::ParseErr(v)
    }
}
impl<'a> From<Parse2Err> for ArgParseErr<'a> {
    fn from(v: Parse2Err) -> Self {
        ArgParseErr::Parse2Err(v)
    }
}
pub enum ArgsParseErr<'a> {
    UnexpectedToken(Arg<'a>, String),
    Help(String),
    UnknownArgs(Vec<Arg<'a>>, String),
    Arg(&'static str, ArgParseErr<'a>, String),
}
impl<'a> Display for ArgsParseErr<'a> {
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
impl<'a> Display for ArgParseErr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ArgParseErr::*;
        match self {
            ParseErr(e) => write!(f, "{e}"),
            Parse2Err(e) => write!(f, "{e}"),
        }
    }
}
impl<'a> Display for ParseErr<'a> {
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
pub enum ArgsErr<'a> {
    Run(String),
    Parse(ArgsParseErr<'a>),
}
impl<'a> From<ArgsParseErr<'a>> for ArgsErr<'a> {
    fn from(v: ArgsParseErr<'a>) -> Self {
        Self::Parse(v)
    }
}
impl<'a> Display for ArgsErr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ArgsErr::*;
        let e = match self {
            Run(e) => e.to_string(),
            Parse(e) => e.to_string(),
        };
        write!(f, "{e}")
    }
}

pub const PFX: &'static str = "--";
#[derive(Debug)]
pub struct Key<'a> {
    pub k: Option<&'a str>,
    pub v: Vec<&'a str>,
    pub used: bool,
}
#[derive(Debug)]
pub struct ParsedArgs<'c> {
    pub k: Vec<Key<'c>>,
}

impl<'c> ParsedArgs<'c> {
    pub fn consume(&mut self, name: &str) -> Option<&Vec<&'c str>> {
        let k = self
            .k
            .iter_mut()
            .find(|k| k.k.map(|k| k == name).unwrap_or(false))?;
        assert!(
            !k.used,
            "A token should not ever be consumed twice. Probably a duplicate argument: {name}"
        );
        k.used = true;
        Some(&k.v)
    }
}

pub enum ParsedArgsErr<'a> {
    UnexpectedToken(Arg<'a>),
}
impl<'c> ParsedArgs<'c> {
    pub fn new(args: &[&'c str]) -> Result<ParsedArgs<'c>, ParsedArgsErr<'c>> {
        let mut last: Option<&str> = None;
        let r = ParsedArgs {
            k: args
                .iter()
                .filter_map(|a| {
                    if a.starts_with(PFX) {
                        last = Some(a);
                        None
                    } else {
                        Some((last, *a))
                    }
                })
                .into_group_map()
                .into_iter()
                .map(|(k, v)| Key { k, v, used: false })
                .collect(),
        };

        Ok(r)
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

pub trait Parse2<'a, 'b, C>
where
    Self: Sized,
    Self::I: Display,
{
    type I;
    fn parse2(
        i: Init<C, Self::I>,
        k: &'static str,
        c: &C,
        p: &mut ParsedArgs<'b>,
    ) -> Result<Self, ArgParseErr<'b>>;
    fn desc2(i: Init<C, Self::I>, d: &'static str, k: &'static str, c: &C) -> [String; 4];
    fn default2(c: &C, i: Init<C, Self::I>) -> Self;
}

impl<'a, 'b, C, T: Parse<'a> + Default> Parse2<'b, 'a, C> for T {
    type I = T;
    fn parse2(
        i: Init<C, Self::I>,
        k: &'static str,
        c: &C,
        p: &mut ParsedArgs<'a>,
    ) -> Result<Self, ArgParseErr<'a>> {
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

impl<'a, 'b, Ctx, T: Parse<'a>> Parse2<'b, 'a, Ctx> for Option<T> {
    type I = T;
    fn parse2(
        i: Init<Ctx, T>,
        k: &'static str,
        c: &Ctx,
        p: &mut ParsedArgs<'a>,
    ) -> Result<Self, ArgParseErr<'a>> {
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

impl<'a, 'b, Ctx, T: Parse<'a> + Display> Parse2<'b, 'a, Ctx> for Vec<T> {
    type I = DisplayVec<T>;
    fn parse2(
        i: Init<Ctx, Self::I>,
        k: &'static str,
        c: &Ctx,
        p: &mut ParsedArgs<'a>,
    ) -> Result<Self, ArgParseErr<'a>> {
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

#[derive(Debug)]
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
impl<'a, 'b, Ctx, T: Parse<'a>> Parse2<'b, 'a, Ctx> for OptVec<T> {
    type I = DisplayVec<T>;
    fn parse2(
        i: Init<Ctx, Self::I>,
        k: &'static str,
        c: &Ctx,
        p: &mut ParsedArgs<'a>,
    ) -> Result<Self, ArgParseErr<'a>> {
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
