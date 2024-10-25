# What is it?
A tool to help you build CLI programs quickly.

# Motivating Example
```rs
#[derive(Acts)]
#[acts(desc = "salt")]
#[allow(dead_code)]
pub struct Main(
    Printscreen,
    Copy,
    Paste,
    Headset,
    # ...
);

#[derive(Args)]
#[args(desc = "Take a screenshot.")]
struct Printscreen {
    # Static defaults
    #[arg(desc = "The saved image.", s = ("/tmp/image.png"))]
    pub path: String,
}
impl Run<C> for Printscreen {
    type R = ();
    fn run(_: &C, a: Self) -> Result<Self::R, String> {
        sh::printscreen(&a.path)
    }
}

#[derive(Args)]
#[args(desc = "Copy to clipboard.")]
struct Copy {}
impl Run<C> for Copy {
    type R = ();
    fn run(_: &C, _: Self) -> Result<Self::R, String> {
        sh::copy()
    }
}

#[derive(Args)]
#[args(desc = "Paste from clipboard.")]
struct Paste {
    #[arg(desc = "Path to paste the file to.")]
    pub path: Option<String>
}
impl Run<C> for Paste {
    type R = String;
    fn run(_: &C, a: Self) -> Result<Self::R, String> {
        let r = sh::paste()?;
        println!("{r}");
        if let Some(p) => a.path {
            # ...
        }
        Ok(r)
    }
}

# Submenus
#[derive(Acts)]
#[acts(desc = "Headset controls.")]
#[allow(dead_code)]
pub struct Headset(Con, Dis);

#[derive(Args)]
#[args(desc = "Disconnect.")]
struct Dis {
    # Dynamic defaults
    #[arg(desc = "Headphone identifier.", d = |c| c.get_headphone())]
    pub path: String,
}
impl Run<C> for Dis {
    type R = ();
    fn run(c: &C, _: Self) -> Result<Self::R, String> {
        # ...
    }
}
```

```bash
$ salt
Expected an act.
? Choose an action.
> printscreen   Take a screenshot.
  headset       Headset controls.
  copy          Copy to clipboard.
  paste         Paste from clipboard.
[↑↓ to move, enter to select, type to filter]
# Submenus
$ salt
Expected an act.
> Choose an action. headset       Headset controls.
Expected an act.
? Choose an action.
  con Connect.
> dis Disconnect.
[↑↓ to move, enter to select, type to filter]
Operation was interrupted by the user
# Print help when needed.
$ salt gopro
Failed to parse opts.
Error parsing option '--dev.'
Required.
Usage: salt gopro <opts...>
Opts:
--dev Req<String> The device name for the GoPro. ex) /dev/sde1
```

# Example projects
- [gym by shinjitumala](https://github.com/shinjitumala/gym)

More to be published soon.
