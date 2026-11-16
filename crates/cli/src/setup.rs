use anyhow::{Context, Result};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub fn run() -> Result<()> {
    let home = std::env::var_os("HOME").context("HOME is not set")?;
    let directory = std::env::var_os("ZDOTDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(home));
    let file = directory.join(".zshrc");
    let existing = match fs::read_to_string(&file) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error).context("Could not read zsh configuration"),
    };
    let exe = std::env::current_exe()?;
    let on_path = std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths).any(|dir| same_file(&dir.join("figfig"), &exe))
    });
    let command = if on_path {
        "figfig".into()
    } else {
        figfig_core::parser::shell_quote(&exe.to_string_lossy())
    };
    let line = format!("eval \"$({command} init zsh)\"");
    if existing
        .lines()
        .any(|entry| entry.trim() == "# >>> figfig >>>" || entry.trim() == line)
    {
        println!("zsh integration is already installed in {}", file.display());
    } else {
        fs::create_dir_all(&directory)?;
        let backup = directory.join(".zshrc.figfig-backup");
        if file.exists() && !backup.exists() {
            fs::copy(&file, &backup)?;
        }
        let mut output = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file)?;
        if !existing.is_empty() && !existing.ends_with('\n') {
            writeln!(output)?;
        }
        writeln!(output, "\n# >>> figfig >>>\n{line}\n# <<< figfig <<<")?;
        println!("Installed zsh integration in {}", file.display());
    }
    println!("Open a new zsh session. The popup helper starts automatically.\nRun `figfig permissions` once to allow terminal cursor positioning.\nUse `figfig config` to inspect preferences.");
    Ok(())
}

fn same_file(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}
