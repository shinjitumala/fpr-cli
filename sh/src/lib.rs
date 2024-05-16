use std::{
    env::{consts, var},
    fs::{self, canonicalize, File},
    io::{BufRead, BufReader},
    path::Path,
    str::FromStr,
};

pub use cli::*;
use itertools::Itertools;
use regex::Regex;
pub use std::process::Command;

struct Pats {
    start: Regex,
    ty: Regex,
    arg: Regex,
    sh_var: Regex,
    end: Regex,
}

struct Config {
    shared: bool,
}

enum Type {
    Text,
    Interactive,
}

fn gen(fp: &Path, p: &Pats, cfg: &Config) -> String {
    let f = BufReader::new(File::open(&fp).expect("Failed to open file"));

    let name = fp.file_stem().expect("Failed to get filename.");
    let name = name
        .to_str()
        .expect(&format!("Filename is not valid: {:?}", name));

    let filename = fp.file_name().expect("Failed to get filename.");
    let filename = filename
        .to_str()
        .expect(&format!("Filename is not valid: {:?}", filename));

    let absl = canonicalize(&fp).expect(&format!("Failed to get absolute path: {:?}", &fp));
    let plat_absl = || {
        let m = format!("Failed to get parent path: {:?}", absl);
        let y = absl
            .parent()
            .expect(&m)
            .parent()
            .expect(&m)
            .to_str()
            .expect(&m);
        let plat = match consts::OS {
            "linux" => "win",
            "macos" => "mac",
            e => panic!("Unknown platform: {e}"),
        };
        format!("{y}/{plat}/{filename}")
    };
    let absl = absl
        .to_str()
        .expect(&format!("Failed to get absolute path: {:?}", &fp));

    let lines = (|| {
        let mut inner_lines = Vec::<String>::new();
        let mut b = false;

        for l in f.lines() {
            let l = l.unwrap();
            if p.start.find(&l).is_some() {
                b = true;
                continue;
            }
            if b && p.end.find(&l).is_some() {
                break;
            }

            if b {
                inner_lines.push(l);
            }
        }

        if !b {
            panic!(
                "Not all tags present for '{:?}': {:?}, {:?}",
                name, p.start, p.start
            )
        }

        inner_lines
    })();

    if lines.is_empty() {
        panic!("Expected type at first line.")
    };

    let ty =
        p.ty.captures(&lines[0])
            .expect(&format!("Expected one type tag for '{}': {:?}", name, p.ty));
    let ty = match &ty[1] {
        "text" => Type::Text,
        "interactive" => Type::Interactive,
        e => panic!("Unexpected type for '{name}': {e}"),
    };

    struct Arg {
        name: String,
        num: usize,
        varidic: bool,
        desc: String,
    }

    let args =
        lines
            .iter()
            .skip(1)
            .map(|l| -> Result<_, _> { p.arg.captures(&l).ok_or(format!("Malformed line '{l}'")) })
            .map(|m| -> Result<_, String> {
                let m = m?;
                let v = m[2].to_owned();
                let v_caps = p
                    .sh_var
                    .captures(&v)
                    .ok_or(format!("Malformed variable '{v}'"))?;
                let (num, varidic) = if v_caps.get(3).is_some()
                    && v_caps[1].to_string() == r#"("${@:"# {
                    (v_caps[2].to_owned(), true)
                } else if v_caps.get(3).is_none() && v_caps[1].to_string() == r#"$"# {
                    (v_caps[2].to_owned(), false)
                } else {
                    return Err(format!("Malformed variable '{v}' '{:?}'", v_caps));
                };
                let num = usize::from_str(&num).expect(&format!("Not a digit '{num}'"));
                Ok(Arg {
                    name: m[1].to_owned(),
                    num,
                    varidic,
                    desc: m[3].to_owned(),
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .expect(&format!("Failed to parse file {absl}"));

    args.iter().enumerate().for_each(|(i, a)| {
        if i + 1 != a.num {
            panic!("Argument not ordered at {i} for '{:?}'", &fp);
        }
        if a.varidic && a.num != args.len() {
            panic!(
                "Only the last argument can be varidic in '{:?}' '{}'",
                &fp, a.name
            );
        }
    });

    let doc = args
        .iter()
        .filter(|a| !a.desc.is_empty())
        .map(|a| format!("/// {}{}", a.name, a.desc))
        .join("\n");
    let fn_args = args
        .iter()
        .map(|a| {
            if !a.varidic {
                format!("{}: String", a.name)
            } else {
                format!("{}: &[String]", a.name)
            }
        })
        .join(", ");
    let ret_ty = match ty {
        Type::Text => "String",
        Type::Interactive => "()",
    };
    let vec = if 0 < args.iter().filter(|a| a.varidic).count() {
        "use velcro::vec;"
    } else {
        ""
    };
    let res = match ty {
        Type::Text => "let r = ",
        Type::Interactive => "let _ = ",
    };
    let cmd = if cfg.shared {
        absl.to_owned()
    } else {
        plat_absl()
    };
    let cmd_args = if args.is_empty() {
        format!("")
    } else {
        let a = args
            .iter()
            .map(|a| {
                if !a.varidic {
                    format!("{}", a.name)
                } else {
                    format!("..{}.to_owned()", a.name)
                }
            })
            .join(", ");
        format!(".args(vec![{a}])")
    };
    let exec = match ty {
        Type::Text => "output",
        Type::Interactive => "status",
    };
    let ret = match ty {
        Type::Text => format!(
            r#"Ok(String::from_utf8(r.stdout).map_err(|e| format!("Output of '{cmd}' not valid UTF-8. '{{e}}'"))?)"#
        ),
        Type::Interactive => format!("Ok(())"),
    };

    format!(
        r#"{doc}
#[allow(dead_code)]
pub fn {name}({fn_args}) -> Res<{ret_ty}> {{
    {vec}
    {res}std::process::Command::new("{cmd}"){cmd_args}.{exec}().map_err(|e| format!("Command '{cmd}' error '{{e}}'"))?;
    {ret}
}}
"#
    )
}

fn gen2(d: &String, p: &Pats, cfg: Config) -> String {
    fs::read_dir(&d)
        .expect(&format!("Failed to read dir '{}'", d))
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .map(|f| -> Result<String, String> {
            let f = f.path();
            Ok(gen(&f, &p, &cfg))
        })
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to generate code")
        .join("\n")
}

pub fn run(src: &'static str, main_plat: &'static str, dst_file: &'static str) {
    let src = format!("{}/{}", var("CARGO_MANIFEST_DIR").unwrap(), src);
    let out = format!("{}/{}", var("OUT_DIR").unwrap(), dst_file);

    let p = Pats {
        start: Regex::new("^# start metadata$").unwrap(),
        end: Regex::new("^# end metadata$").unwrap(),
        ty: Regex::new("^# type ([^ ]+)$").unwrap(),
        arg: Regex::new(r#"^([^=]+)=([()"1-9${}:@]+)(.*)$"#).unwrap(),
        sh_var: Regex::new(r#"([(${"@:]+)([0-9]+)(\}"\))?"#).unwrap(),
    };

    let src_main_plat = format!("{}/{}/", src, main_plat);
    let src_all = format!("{}/all/", src);

    let r_shared = gen2(&src_all, &p, Config { shared: true });
    let r_plat = gen2(&src_main_plat, &p, Config { shared: false });

    fs::write(&out, format!("{r_shared}\n{r_plat}\n"))
        .expect(&format!("Failed to write to '{}'", &out));
}

pub fn run_sh<Ctx, A: Acts<Ctx>>(pfx: &'static str, dst: &'static str) {
    let pfx = [format!("{pfx}")];
    let cmds = list::<Ctx, A>()
        .iter()
        .map(|a| {
            let mut a = a.to_owned();
            a.splice(0..0, pfx.iter().cloned());
            a
        })
        .collect::<Vec<_>>();
    let out = format!("{}/{}", var("OUT_DIR").unwrap(), dst);

    let body = cmds
        .iter()
        .map(|c| {
            let cmd = c.join("_");
            let cmd2 = c.join(" ");
            format!(r#"function {cmd} () {{ {cmd2} "$@"; }} "#)
        })
        .join("\n");

    const HEAD: &'static str = r"#!/bin/bash
# Generated script";

    let content = format!("{HEAD}\n{body}\n");

    fs::write(&out, content).expect(&format!("Failed to write to {}", &out));
}
