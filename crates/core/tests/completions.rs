use figfig_core::{CompletionRequest, Engine};
use std::fs;

fn request(cwd: &std::path::Path, buffer: &str) -> CompletionRequest {
    CompletionRequest {
        session_id: "integration".into(),
        revision: 1,
        buffer: buffer.into(),
        cursor: buffer.chars().count(),
        cwd: cwd.to_string_lossy().into(),
        terminal_pid: 0,
        terminal_size: None,
    }
}

#[test]
fn filesystem_candidates_are_literal_and_directories_stay_open() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("日本 語.txt"), "").unwrap();
    fs::create_dir(dir.path().join("日本 folder")).unwrap();
    let response = Engine::new().complete(&request(dir.path(), "cat 日本"), 10);
    let file = response
        .candidates
        .iter()
        .find(|c| c.label == "日本 語.txt")
        .unwrap();
    assert_eq!(file.insert_text, "'日本 語.txt'");
    assert!(file.append_space);
    let cd = Engine::new().complete(&request(dir.path(), "cd 日本"), 10);
    assert_eq!(cd.candidates.len(), 1);
    assert!(!cd.candidates[0].append_space);
}

#[test]
fn package_scripts_are_read_without_running_them() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"scripts":{"dev":"touch SHOULD_NOT_EXIST","test:unit":"vitest"}}"#,
    )
    .unwrap();
    let response = Engine::new().complete(&request(dir.path(), "pnpm run te"), 10);
    assert_eq!(response.candidates[0].label, "test:unit");
    assert!(!dir.path().join("SHOULD_NOT_EXIST").exists());
}

#[test]
fn local_git_branches_are_suggested() {
    let dir = tempfile::tempdir().unwrap();
    let git = |args: &[&str]| {
        assert!(std::process::Command::new("/usr/bin/git")
            .args(args)
            .current_dir(dir.path())
            .status()
            .unwrap()
            .success());
    };
    git(&["init", "-q"]);
    git(&[
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.invalid",
        "-c",
        "commit.gpgsign=false",
        "commit",
        "--allow-empty",
        "-qm",
        "fixture",
    ]);
    git(&["branch", "feature/figfig"]);
    let response = Engine::new().complete(&request(dir.path(), "git switch fe"), 10);
    assert_eq!(response.candidates[0].label, "feature/figfig");
}
