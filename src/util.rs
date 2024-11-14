use std::path::Path;

use chrono::{DateTime, FixedOffset, TimeZone};
use regex::Regex;

use crate::com::*;

pub fn to_lines<const S: usize, I: AsRef<str>>(a: &[[I; S]]) -> Vec<String> {
    use unicode_width::*;
    let w = match (0..S)
        .map(|i| a.iter().map(|l| l[i].as_ref().width()).max().ok_or(()))
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(e) => e,
        Err(_) => {
            return vec![];
        }
    };
    a.iter()
        .map(|v| {
            v.iter()
                .enumerate()
                .map(|(i, s)| format!("{}{: <2$}", s.as_ref(), "", w[i] - s.as_ref().width()))
                .join(" ")
        })
        .collect()
}
pub fn to_table<const S: usize, I: AsRef<str>>(a: &[[I; S]]) -> String {
    to_lines(a).join("\n")
}

fn to_option_lines<const S: usize, I: AsRef<str>, T>(
    t: &[T],
    f: fn(&T) -> [I; S],
) -> Vec<ListOption<String>> {
    to_lines(&t.iter().map(f).collect::<Vec<_>>())
        .into_iter()
        .enumerate()
        .map(|(i, e)| ListOption::new(i, e))
        .collect()
}

pub fn select_line<'a, const S: usize, I: AsRef<str>, T>(
    prompt: &'a str,
    t: &[T],
    f: fn(&T) -> [I; S],
) -> Select<'a, ListOption<String>> {
    Select::new(prompt, to_option_lines(t, f))
}
pub fn select_multiple_line<'a, const S: usize, I: AsRef<str>, T>(
    prompt: &'a str,
    t: &[T],
    f: fn(&T) -> [I; S],
) -> MultiSelect<'a, ListOption<String>> {
    MultiSelect::new(prompt, to_option_lines(t, f))
}

pub fn input_path<'a>(prompt: &'a str) -> Text {
    Text::new(prompt).with_autocomplete(filepath::Comp::default())
}

mod filepath {
    use crate::com::*;

    #[derive(Clone, Default)]
    pub struct Comp {
        input: String,
        paths: Vec<String>,
    }

    impl Comp {
        fn update_input(&mut self, input: &str) -> Result<(), CustomUserError> {
            if input == self.input {
                return Ok(());
            }

            self.input = input.to_owned();
            self.paths.clear();

            let input_path = PathBuf::from(input);

            let fb = input_path
                .parent()
                .map(|p| {
                    if p.to_string_lossy() == "" {
                        PathBuf::from(".")
                    } else {
                        p.to_owned()
                    }
                })
                .unwrap_or_else(|| PathBuf::from("."));

            let scan_dir = if input.ends_with('/') {
                input_path
            } else {
                fb.clone()
            };

            let entries = match std::fs::read_dir(scan_dir) {
                Ok(r) => r.filter_map(|e| e.ok()).collect(),
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    match std::fs::read_dir(fb) {
                        Ok(r) => r.filter_map(|e| e.ok()).collect(),
                        Err(_) => vec![],
                    }
                }
                Err(_) => vec![],
            };

            for entry in entries {
                let path = entry.path();
                let path_str = if path.is_dir() {
                    format!("{}/", path.to_string_lossy())
                } else {
                    path.to_string_lossy().to_string()
                };

                self.paths.push(path_str);
            }

            Ok(())
        }

        fn fuzzy_sort(&self, input: &str) -> Vec<(String, i64)> {
            let mut matches: Vec<(String, i64)> = self
                .paths
                .iter()
                .filter_map(|path| {
                    SkimMatcherV2::default()
                        .smart_case()
                        .fuzzy_match(path, input)
                        .map(|score| (path.clone(), score))
                })
                .collect();

            matches.sort_by(|a, b| b.1.cmp(&a.1));
            matches
        }
    }

    fn expand(s: &str) -> String {
        match shellexpand::full(s) {
            Ok(e) => e.to_string(),
            Err(_) => s.to_owned(),
        }
    }

    impl Autocomplete for Comp {
        fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
            let input = &expand(input);
            self.update_input(input)?;

            let matches = self.fuzzy_sort(input);
            Ok(matches.into_iter().take(15).map(|(path, _)| path).collect())
        }

        fn get_completion(
            &mut self,
            input: &str,
            highlighted_suggestion: Option<String>,
        ) -> Result<Replacement, CustomUserError> {
            let input = &expand(input);
            self.update_input(input)?;

            Ok(match highlighted_suggestion {
                Some(e) => Replacement::Some(e),
                None => {
                    let matches = self.fuzzy_sort(input);
                    matches
                        .first()
                        .map(|(path, _)| Replacement::Some(path.clone()))
                        .unwrap_or(Replacement::None)
                }
            })
        }
    }
}

#[derive(Clone)]
pub struct MyDateTime<C: TimeZone> {
    v: DateTime<C>,
}
impl<C: TimeZone> Into<DateTime<C>> for MyDateTime<C>
where
    DateTime<C>: From<DateTime<FixedOffset>>,
{
    fn into(self) -> DateTime<C> {
        self.v.into()
    }
}

impl<C: TimeZone> FromStr for MyDateTime<C>
where
    DateTime<C>: From<DateTime<FixedOffset>>,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            v: DateTime::parse_from_rfc3339(s)
                .map_err(|e| format!("{e}"))?
                .into(),
        })
    }
}
impl<C: TimeZone> Display for MyDateTime<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.v.to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
        )
    }
}

impl<C: TimeZone> CustomTypeValidator<String> for MyDateTime<C> {
    fn validate(
        &self,
        i: &String,
    ) -> Result<inquire::validator::Validation, inquire::CustomUserError> {
        use inquire::validator::Validation::*;
        match DateTime::parse_from_rfc3339(i) {
            Ok(_) => Ok(Valid),
            Err(e) => Ok(Invalid(ErrorMessage::Custom(format!("{e}")))),
        }
    }
}

pub fn input_date<'a, C: TimeZone>(prompt: &'a str) -> CustomType<MyDateTime<C>>
where
    DateTime<C>: From<DateTime<FixedOffset>>,
{
    CustomType::<MyDateTime<C>>::new(prompt)
}

#[derive(Debug)]
pub enum MyErr {
    Inquire(inquire::InquireError),
    Generic(String),
}
impl From<String> for MyErr {
    fn from(v: String) -> Self {
        Self::Generic(v)
    }
}
impl From<inquire::InquireError> for MyErr {
    fn from(v: inquire::InquireError) -> Self {
        Self::Inquire(v)
    }
}
impl From<MyErr> for String {
    fn from(v: MyErr) -> Self {
        use MyErr::*;
        match v {
            Inquire(e) => format!("{e}"),
            Generic(e) => format!("{e}"),
        }
    }
}

pub trait Actions: Sized + Clone {
    fn get(prompt: &str, starting_input: Option<&str>) -> Result<Self, MyErr>;
}

#[derive(Clone)]
pub struct TextWithAutocomplete<I: Clone, const S: usize> {
    i: Vec<I>,

    input: String,
    matches: Vec<String>,
    print: fn(&I) -> [String; S],
}
impl<I: Clone, const S: usize> TextWithAutocomplete<I, S> {
    fn update_input(&mut self, input: &str) -> Result<(), CustomUserError> {
        if input == self.input {
            return Ok(());
        }

        self.input = input.to_owned();
        let mut m: Vec<_> = self
            .i
            .iter()
            .map(|c| {
                let s = (self.print)(c);
                let v = SkimMatcherV2::default()
                    .smart_case()
                    .fuzzy_match(&s.join(" "), input);
                (s, v)
            })
            .collect();

        m.sort_by(|a, b| b.1.cmp(&a.1));
        self.matches = to_lines(&m.into_iter().map(|e| e.0).collect_vec());
        Ok(())
    }

    pub fn new(i: Vec<I>, print: fn(&I) -> [String; S]) -> Self {
        Self {
            i,
            print,
            input: String::new(),
            matches: Vec::new(),
        }
    }
}

impl<I: Clone, const S: usize> Autocomplete for TextWithAutocomplete<I, S> {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        self.update_input(input)?;
        Ok(self.matches.to_owned())
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        self.update_input(input)?;

        Ok(match highlighted_suggestion {
            Some(e) => Replacement::Some(e),
            None => self
                .matches
                .first()
                .map(|e| Replacement::Some(e.to_owned()))
                .unwrap_or(Replacement::None),
        })
    }
}

type Res<T> = Result<T, MyErr>;
pub fn env_var(s: &str) -> Res<String> {
    Ok(std::env::var(s).map_err(|e| format!("Failed to get env '{s}' because '{e}'"))?)
}
pub fn reg(s: &str) -> Res<Regex> {
    Ok(Regex::new(s).map_err(|e| format!("Failed to compile regex '{s}' because '{e}'"))?)
}
pub fn fs_write<P: AsRef<Path>, C: AsRef<[u8]>>(p: P, c: C) -> Res<()> {
    Ok(std::fs::write(p.as_ref(), c).map_err(|e| {
        format!(
            "Failed to write to '{}' because '{e}'",
            p.as_ref().to_string_lossy()
        )
    })?)
}
pub fn fs_read<P: AsRef<Path>>(p: P) -> Res<Vec<u8>> {
    Ok(std::fs::read(p.as_ref()).map_err(|e| {
        format!(
            "Failed to read to '{}' because '{e}'",
            p.as_ref().to_string_lossy()
        )
    })?)
}
