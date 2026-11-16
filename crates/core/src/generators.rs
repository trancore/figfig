use crate::{engine::candidate, Candidate};
use std::{
    fs,
    io::Read,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

pub(super) fn package_scripts(cwd: &Path) -> Vec<Candidate> {
    // Do not run package manager commands or lifecycle scripts for completion.
    let path = cwd.join("package.json");
    let Ok(file) = fs::File::open(path) else {
        return Vec::new();
    };
    let mut contents = String::new();
    if file
        .take(1024 * 1024)
        .read_to_string(&mut contents)
        .is_err()
    {
        return Vec::new();
    }
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&contents) else {
        return Vec::new();
    };
    value
        .get("scripts")
        .and_then(|v| v.as_object())
        .map(|scripts| {
            scripts
                .iter()
                .filter(|(name, _)| !name.chars().any(char::is_control))
                .map(|(name, body)| {
                    candidate(name, body.as_str().unwrap_or("Package script"), "script")
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn git_branches(cwd: &Path) -> Vec<String> {
    let Ok(mut child) = Command::new("/usr/bin/git")
        .args([
            "--no-optional-locks",
            "for-each-ref",
            "--count=200",
            "--format=%(refname:short)",
            "refs/heads/",
        ])
        .current_dir(cwd)
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        return Vec::new();
    };
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => break,
            Ok(Some(_)) | Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Vec::new();
            }
            Ok(None) if start.elapsed() > Duration::from_millis(150) => {
                let _ = child.kill();
                let _ = child.wait();
                return Vec::new();
            }
            _ => std::thread::sleep(Duration::from_millis(3)),
        }
    }
    let mut output = String::new();
    if let Some(pipe) = child.stdout.take() {
        let _ = pipe.take(64 * 1024).read_to_string(&mut output);
    }
    output
        .lines()
        .filter(|s| !s.chars().any(char::is_control))
        .map(str::to_owned)
        .collect()
}
