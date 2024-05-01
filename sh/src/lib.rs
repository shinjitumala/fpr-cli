use std::{
    env::{consts, var},
    fs::{self, canonicalize, File},
    io::{BufRead, BufReader},
    path::Path,
};

use regex::Regex;

struct Pats {
    start: Regex,
    ty: Regex,
    arg: Regex,
    end: Regex,
}

const I: &'static str = "    ";

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

    let ty = lines
        .iter()
        .map(|l| p.ty.captures(&l))
        .filter(|m| m.is_some())
        .map(|m| m.unwrap())
        .map(|m| {
            m.iter()
                .skip(1)
                .map(|m| m.unwrap().as_str())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    if ty.len() != 1 {
        panic!("Expected one type tag for '{}': {:?}", name, p.ty);
    }
    let ty = match ty[0][0] {
        "text" => Type::Text,
        "interactive" => Type::Interactive,
        _ => panic!("Unexpected type for '{name}': {}", ty[0][0]),
    };

    let args = lines
        .iter()
        .map(|l| p.arg.captures(&l))
        .filter(|m| m.is_some())
        .map(|m| m.unwrap())
        .map(|m| {
            m.iter()
                .skip(1)
                .map(|m| m.unwrap().as_str())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let mut buf = String::new();

    let result_type_name = format!("R{}", name);

    buf.push_str("#[derive(Args)]\n");
    buf.push_str("#[args(ctx = Ctx)]\n");
    buf.push_str(&format!("pub struct {} ", result_type_name));
    buf.push_str("{\n");
    for a in args.iter() {
        let ty = match a[2] {
            "" => format!("Init::None"),
            e => format!("Init::Const(format!({}))", e),
        };
        buf.push_str(&format!("    #[arg(desc = (\"{}\"), i = {ty})]\n", a[3]));
        buf.push_str(&format!("    {}: Arg<Ctx, Req<One<String>>>,\n", a[0]));
    }
    buf.push_str("}\n");

    buf.push_str(&format!("pub fn {}_from_args(", name));
    buf.push_str("args: &[String]) -> ");
    buf.push_str(match ty {
        Type::Text => "Result<String, String>",
        Type::Interactive => "Result<(), String>",
    });
    buf.push_str(" {\n");
    buf.push_str(&format!("{}let c = Ctx {{}};\n", I));
    buf.push_str(&format!(
        "{}let a = parse2::<_, {}>(&c, args).map_err(|e| format!(\"Parse error: {{}}\\nUsage:\\n{{}}\", e, desc::<_, {}>(&c)))?;\n",
        I, result_type_name, result_type_name
    ));
    buf.push_str(&format!("{}_from_struct(a)\n", name));
    buf.push_str("}\n");

    buf.push_str(&format!(
        "pub fn {}_from_struct(a: Ret<Ctx, {}>) -> ",
        name, result_type_name
    ));
    buf.push_str(match ty {
        Type::Text => "Result<String, String>",
        Type::Interactive => "Result<(), String>",
    });
    buf.push_str(" {\n");
    let cmd = if cfg.shared {
        absl.to_owned()
    } else {
        plat_absl()
    };
    buf.push_str(&format!("{I}let cmd = \"{cmd}\";"));
    buf.push_str(&format!("{I}let r = Command::new(cmd)\n"));
    buf.push_str(&format!("{I}{I}.args(["));
    for a in args.iter() {
        buf.push_str(&format!("a.{}, ", a[0]));
    }
    buf.push_str("])\n");
    buf.push_str(&match ty {
        Type::Text => format!("{I}{I}.output();\n"),
        Type::Interactive => format!("{I}{I}.status();\n"),
    });
    buf.push_str(&format!("{}match r {{\n", I));
    buf.push_str(&match ty {
        Type::Text => format!("{}{}Ok(r) => if r.status.success() {{ Ok(String::from_utf8(r.stdout).map_err(|e| format!(\"Output not valid: {{}}\", e))?) }} else {{ Err(String::from_utf8(r.stderr).map_err(|e| format!(\"Output not valid: {{}}\", e))?)  }},\n", I, I),
        Type::Interactive => format!("{}{}Ok(r) => if r.success() {{ Ok(()) }} else {{ Err(format!(\"Command failed.\")) }},\n", I, I),
    });
    buf.push_str(&match ty {
        Type::Text => format!(
            "{}{}Err(r) => Err(format!(\"Command '{{}}' error: {{}}\", cmd, r)),\n",
            I, I
        ),
        Type::Interactive => format!(
            "{}{}Err(r) => Err(format!(\"Command '{{}}' error: {{}}\", cmd, r)),\n",
            I, I
        ),
    });
    buf.push_str(&format!("{}}}\n", I));
    buf.push_str("}\n");

    println!("{}", buf);
    buf
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
        arg: Regex::new(r"^([^=]+)=\$([^ ]+) # ([^:]*): (.*)$").unwrap(),
    };

    let src_main_plat = format!("{}/{}/", src, main_plat);

    let r_shared = gen2(&src, &p, Config { shared: true });
    let r_plat = gen2(&src_main_plat, &p, Config { shared: false });

    fs::write(&out, format!("{r_shared}\n{r_plat}\n"))
        .expect(&format!("Failed to write to '{}'", &out));
}
