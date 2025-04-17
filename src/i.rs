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

impl<'a> Parse<'a> for i32 {
    fn parse(i: Arg<'a>) -> Result<Self, ParseErr> {
        i32::from_str(i).map_err(|e| ParseErr {
            i,
            ty: Self::desc(),
            e: format!("{e}"),
        })
    }

    fn desc() -> &'static str {
        stringify!(i32)
    }
}
impl<'a> Parse<'a> for i64 {
    fn parse(i: Arg<'a>) -> Result<Self, ParseErr> {
        i64::from_str(i).map_err(|e| ParseErr {
            i,
            ty: Self::desc(),
            e: format!("{e}"),
        })
    }

    fn desc() -> &'static str {
        stringify!(i64)
    }
}
impl<'a> Parse<'a> for String {
    fn parse(i: Arg<'a>) -> Result<Self, ParseErr> {
        String::from_str(i).map_err(|e| ParseErr {
            i,
            ty: Self::desc(),
            e: format!("{e}"),
        })
    }

    fn desc() -> &'static str {
        stringify!(String)
    }
}

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
    fn parse(i: Arg<'a>) -> Result<Self, ParseErr> {
        match file_exist(i) {
            Ok(p) => Ok(FileExist { p, s: i.to_owned() }),
            Err(e) => Err(ParseErr {
                i,
                ty: Self::desc(),
                e,
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
    fn parse(i: Arg<'a>) -> Result<Self, ParseErr> {
        match dir_exist(i) {
            Ok(p) => Ok(DirExist { p, s: i.to_owned() }),
            Err(e) => Err(ParseErr {
                i,
                ty: Self::desc(),
                e,
            }),
        }
    }

    fn desc() -> &'static str {
        stringify!(DirExist)
    }
}
