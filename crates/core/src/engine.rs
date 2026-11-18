use crate::{
    generators::{git_branches, package_scripts},
    parser, Candidate, CompletionRequest, CompletionResponse,
};
use serde::Deserialize;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OptionSpec {
    name: String,
    description: String,
    #[serde(default)]
    takes_value: bool,
}
#[derive(Debug, Deserialize)]
struct SubcommandSpec {
    name: String,
    description: String,
    options: Vec<OptionSpec>,
    generator: Option<String>,
}
#[derive(Debug, Deserialize)]
struct CommandSpec {
    name: String,
    description: String,
    options: Vec<OptionSpec>,
    subcommands: Vec<SubcommandSpec>,
}

struct CommandContext<'a> {
    subcommand: Option<&'a SubcommandSpec>,
    expecting_value: Option<&'a str>,
}

impl CommandSpec {
    fn resolve_arguments(&self, words: &[String]) -> CommandContext<'_> {
        let mut subcommand = None;
        let mut expecting_value = None;
        for word in words.iter().skip(1) {
            if expecting_value.take().is_some() {
                continue;
            }
            if word.starts_with('-') {
                let options = subcommand
                    .map(|s: &SubcommandSpec| s.options.as_slice())
                    .unwrap_or(&[]);
                expecting_value = options
                    .iter()
                    .chain(&self.options)
                    .find(|o| o.name == *word && o.takes_value)
                    .map(|o| o.name.as_str());
            } else if subcommand.is_none() {
                subcommand = self.subcommands.iter().find(|s| s.name == *word);
            }
        }
        CommandContext {
            subcommand,
            expecting_value,
        }
    }
}

pub struct Engine {
    specs: Vec<CommandSpec>,
    commands: Vec<String>,
    home: PathBuf,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        use std::os::unix::fs::PermissionsExt;
        let mut commands = HashSet::new();
        if let Some(path) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&path) {
                if !dir.is_absolute() {
                    continue;
                }
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten().take(10_000) {
                        if let Ok(meta) = entry.metadata() {
                            if meta.is_file() && meta.permissions().mode() & 0o111 != 0 {
                                if let Some(name) = entry.file_name().to_str() {
                                    commands.insert(name.to_owned());
                                }
                            }
                        }
                    }
                }
            }
        }
        let mut commands: Vec<_> = commands.into_iter().collect();
        commands.sort();
        Self {
            specs: serde_json::from_str(include_str!("../../../assets/specs.json"))
                .expect("built-in specs must be valid"),
            commands,
            home: std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_default(),
        }
    }

    pub fn complete(&self, request: &CompletionRequest, limit: usize) -> CompletionResponse {
        let mut response = CompletionResponse {
            session_id: request.session_id.clone(),
            revision: request.revision,
            ..Default::default()
        };
        if request.buffer.is_empty() {
            return response;
        }
        let Some(context) = parser::context(&request.buffer, request.cursor) else {
            return response;
        };
        response.replace_start = context.start;
        response.replace_end = context.end;
        let mut candidates = if context.words.is_empty() && !context.prefix.contains('/') {
            self.command_candidates()
        } else {
            self.argument_candidates(&context, Path::new(&request.cwd))
        };
        let mut seen = HashSet::new();
        candidates
            .retain(|c| rank(&c.label, &context.prefix).is_some() && seen.insert(c.label.clone()));
        candidates.sort_by(|a, b| {
            rank(&a.label, &context.prefix)
                .cmp(&rank(&b.label, &context.prefix))
                .then_with(|| kind_rank(&a.kind).cmp(&kind_rank(&b.kind)))
                .then_with(|| a.label.cmp(&b.label))
        });
        candidates.truncate(limit.clamp(1, 100));
        response.candidates = candidates;
        response
    }

    fn command_candidates(&self) -> Vec<Candidate> {
        let mut candidates = Vec::new();
        for spec in &self.specs {
            candidates.push(candidate(&spec.name, &spec.description, "command"));
        }
        for name in &self.commands {
            candidates.push(candidate(name, "Executable on PATH", "command"));
        }
        for name in [
            "cd", "ls", "cat", "pwd", "echo", "mkdir", "open", "source", "export",
        ] {
            candidates.push(candidate(name, "Shell command", "command"));
        }
        candidates
    }

    fn argument_candidates(&self, context: &parser::Context, cwd: &Path) -> Vec<Candidate> {
        let mut candidates = Vec::new();
        let command = context.words.first().map(|s| s.as_str()).unwrap_or("");
        let spec = self.specs.iter().find(|s| s.name == command);
        let mut expecting_value = None;
        if let Some(spec) = spec {
            let resolved = spec.resolve_arguments(&context.words);
            let subcommand = resolved.subcommand;
            expecting_value = resolved.expecting_value;
            if expecting_value.is_none() {
                if context.prefix.starts_with('-') {
                    for o in spec
                        .options
                        .iter()
                        .chain(subcommand.map(|s| s.options.as_slice()).unwrap_or(&[]))
                    {
                        candidates.push(candidate(&o.name, &o.description, "option"));
                    }
                } else if let Some(sub) = subcommand {
                    match sub.generator.as_deref() {
                        Some("gitBranches") => candidates.extend(
                            git_branches(cwd)
                                .into_iter()
                                .map(|s| candidate(&s, "Local Git branch", "branch")),
                        ),
                        Some("packageScripts") => candidates.extend(package_scripts(cwd)),
                        _ => (),
                    }
                } else {
                    candidates.extend(
                        spec.subcommands
                            .iter()
                            .map(|s| candidate(&s.name, &s.description, "subcommand")),
                    );
                }
            }
        }
        let values_are_paths =
            expecting_value.is_none() || matches!(expecting_value, Some("-C" | "-f"));
        if values_are_paths && !context.prefix.starts_with('-') {
            candidates.extend(self.paths(&context.prefix, cwd, command == "cd"));
        }
        candidates
    }

    fn paths(&self, prefix: &str, cwd: &Path, directories_only: bool) -> Vec<Candidate> {
        let (parent, needle) = prefix
            .rsplit_once('/')
            .map(|(a, b)| (format!("{a}/"), b))
            .unwrap_or((String::new(), prefix));
        let dir = if let Some(relative) = parent.strip_prefix("~/") {
            self.home.join(relative)
        } else if parent.is_empty() {
            cwd.to_path_buf()
        } else {
            cwd.join(&parent)
        };
        let Ok(entries) = fs::read_dir(dir) else {
            return Vec::new();
        };
        let mut result = Vec::new();
        for entry in entries.flatten().take(5000) {
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if !name.starts_with(needle)
                || (name.starts_with('.') && !needle.starts_with('.'))
                || name.chars().any(char::is_control)
            {
                continue;
            }
            let is_dir = entry.path().is_dir();
            if directories_only && !is_dir {
                continue;
            }
            let label = format!("{parent}{name}{}", if is_dir { "/" } else { "" });
            let insert = if let Some(relative) = label.strip_prefix("~/") {
                self.home.join(relative).to_string_lossy().into_owned()
            } else {
                label.clone()
            };
            result.push(Candidate {
                label,
                description: if is_dir { "Directory" } else { "File" }.into(),
                insert_text: parser::shell_quote(&insert),
                kind: if is_dir { "directory" } else { "file" }.into(),
                append_space: !is_dir,
            });
        }
        result
    }
}

pub(super) fn candidate(label: &str, description: &str, kind: &str) -> Candidate {
    Candidate {
        label: label.into(),
        description: description
            .chars()
            .filter(|c| !c.is_control())
            .take(180)
            .collect(),
        insert_text: parser::shell_quote(label),
        kind: kind.into(),
        append_space: true,
    }
}

fn rank(label: &str, prefix: &str) -> Option<u8> {
    if label == prefix {
        Some(0)
    } else if label.starts_with(prefix) {
        Some(1)
    } else if label.to_lowercase().starts_with(&prefix.to_lowercase()) {
        Some(2)
    } else {
        None
    }
}

fn kind_rank(kind: &str) -> u8 {
    match kind {
        "subcommand" | "option" | "script" | "branch" => 0,
        "command" => 1,
        "directory" => 2,
        _ => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn complete(s: &str) -> CompletionResponse {
        Engine {
            specs: serde_json::from_str(include_str!("../../../assets/specs.json")).unwrap(),
            commands: Vec::new(),
            home: PathBuf::new(),
        }
        .complete(
            &CompletionRequest {
                session_id: "a".into(),
                revision: 1,
                buffer: s.into(),
                cursor: s.chars().count(),
                cwd: "/nonexistent-figfig-unit-test".into(),
                terminal_pid: 0,
                terminal_size: None,
            },
            20,
        )
    }
    #[test]
    fn suggests_contextual_subcommands_and_flags() {
        assert!(complete("git ch")
            .candidates
            .iter()
            .any(|c| c.label == "checkout"));
        assert!(complete("git status --s")
            .candidates
            .iter()
            .any(|c| c.label == "--short"));
        assert!(!complete("git status ")
            .candidates
            .iter()
            .any(|c| c.label == "checkout"));
    }
    #[test]
    fn option_values_do_not_change_the_resolved_subcommand() {
        assert!(complete("git -C status ch")
            .candidates
            .iter()
            .any(|candidate| candidate.label == "checkout"));
        assert!(complete("git commit -m status --a")
            .candidates
            .iter()
            .any(|candidate| candidate.label == "--amend"));
        assert!(!complete("git commit -m status --s")
            .candidates
            .iter()
            .any(|candidate| candidate.label == "--short"));
    }

    #[test]
    fn does_not_offer_flags_for_option_values() {
        assert!(complete("git commit -m --a").candidates.is_empty());
    }
    #[test]
    fn blank_and_unsupported_input_have_no_candidates() {
        assert!(complete("").candidates.is_empty());
        assert!(complete("git $(date)").candidates.is_empty());
    }
}
