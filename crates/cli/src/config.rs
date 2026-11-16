use anyhow::{bail, Context, Result};
use figfig_core::runtime;

pub enum ConfigResult {
    Printed,
    Changed,
}

pub fn run(args: &[String]) -> Result<ConfigResult> {
    if matches!(args.first().map(String::as_str), Some("path")) && args.len() == 1 {
        println!("{}", runtime::settings_path().display());
        return Ok(ConfigResult::Printed);
    }
    let mut settings = runtime::load_settings()?;
    match args.first().map(String::as_str) {
        None | Some("list") if args.len() <= 1 => {
            println!("{}", serde_json::to_string_pretty(&settings)?)
        }
        Some("get") if args.len() == 2 => match args[1].as_str() {
            "popup" => println!("{}", settings.popup_enabled),
            "history" => println!("{}", settings.history_enabled),
            "max-candidates" => println!("{}", settings.max_candidates),
            _ => bail!("Unknown setting. Keys: popup, history, max-candidates"),
        },
        Some("set") if args.len() == 3 => {
            match args[1].as_str() {
                "popup" => {
                    settings.popup_enabled =
                        args[2].parse().context("popup must be true or false")?
                }
                "history" => {
                    settings.history_enabled =
                        args[2].parse().context("history must be true or false")?
                }
                "max-candidates" => {
                    let count = args[2]
                        .parse::<usize>()
                        .context("max-candidates must be an integer from 3 to 12")?;
                    if !(3..=12).contains(&count) {
                        bail!("max-candidates must be from 3 to 12");
                    }
                    settings.max_candidates = count;
                }
                _ => bail!("Unknown setting. Keys: popup, history, max-candidates"),
            }
            runtime::save_settings(settings)?;
            return Ok(ConfigResult::Changed);
        }
        _ => bail!("Usage: figfig config [list | path | get KEY | set KEY VALUE]"),
    }
    Ok(ConfigResult::Printed)
}
