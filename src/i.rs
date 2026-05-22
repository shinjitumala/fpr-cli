use crate::com::*;

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

macro_rules! from_str {
    ($a:ty) => {
        impl<'a> Parse<'a> for $a {
            fn parse(i: Arg) -> Result<Self, ErrVal> {
                <$a>::from_str(i).map_err(|e| ErrVal::Err {
                    input: i.into(),
                    typename: Self::desc().into(),
                    error: format!("{e}"),
                })
            }

            fn desc() -> &'static str {
                stringify!($a)
            }
        }
    };
}

from_str!(i8);
from_str!(i16);
from_str!(i32);
from_str!(i64);
from_str!(i128);
from_str!(u8);
from_str!(u16);
from_str!(u32);
from_str!(u64);
from_str!(u128);
from_str!(f32);
from_str!(f64);
from_str!(String);
from_str!(bool);
from_str!(char);
from_str!(usize);
from_str!(isize);

fn file_exist(i: &str) -> Result<PathBuf, String> {
    let p = PathBuf::from_str(i).map_err(|e| e.to_string())?;
    if !p.exists() {
        return Err(format!("Does not exist"));
    };
    if !p.is_file() {
        return Err(format!("Not a file"));
    };
    Ok(p)
}

impl<'a> Parse<'a> for FileExist {
    fn parse(i: Arg) -> Result<Self, ErrVal> {
        match file_exist(i) {
            Ok(p) => Ok(FileExist { p, s: i.to_owned() }),
            Err(e) => Err(ErrVal::Err {
                input: i.into(),
                typename: Self::desc().into(),
                error: e,
            }),
        }
    }

    fn desc() -> &'static str {
        stringify!(FileExist)
    }
}

fn dir_exist(i: &str) -> Result<PathBuf, String> {
    let p = PathBuf::from_str(i).map_err(|e| e.to_string())?;
    if !p.exists() {
        return Err(format!("Does not exist"));
    };
    if !p.is_dir() {
        return Err(format!("Not a dir"));
    };
    Ok(p)
}

impl<'a> Parse<'a> for DirExist {
    fn parse(i: Arg) -> Result<Self, ErrVal> {
        match dir_exist(i) {
            Ok(p) => Ok(DirExist { p, s: i.to_owned() }),
            Err(e) => Err(ErrVal::Err {
                input: i.into(),
                typename: Self::desc().into(),
                error: e,
            }),
        }
    }

    fn desc() -> &'static str {
        stringify!(DirExist)
    }
}
