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
impl Parse for i64 {
    fn parse(i: &Arg) -> Result<Self, ParseErr> {
        i64::from_str(i).map_err(|e| ParseErr {
            i: i.to_owned(),
            ty: Self::desc(),
            e: format!("{e}"),
        })
    }

    fn desc() -> &'static str {
        stringify!(i64)
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
