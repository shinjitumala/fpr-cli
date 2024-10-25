use chrono::{DateTime, FixedOffset, TimeZone};

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
    t: &Vec<T>,
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
    t: &Vec<T>,
    f: fn(&T) -> [I; S],
) -> Select<'a, ListOption<String>> {
    Select::new(prompt, to_option_lines(t, f))
}
pub fn select_multiple_line<'a, const S: usize, I: AsRef<str>, T>(
    prompt: &'a str,
    t: &Vec<T>,
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
