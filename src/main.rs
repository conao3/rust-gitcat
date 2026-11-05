mod error;
mod grammar;
mod output;

use std::collections::BTreeMap;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::process;

use clap::{ArgAction, Args, CommandFactory, Parser, Subcommand};
use colored::Colorize;
use serde::Serialize;
use serde_json::Value;

use error::{Result, other, target};
use output::{OutputFormat, Response};

#[derive(Parser)]
#[command(
    name = "gitcat",
    version,
    about = "Preview your repository as GitHub would see it"
)]
struct Cli {
    /// Output format (text prints the GitHub-style tree)
    #[arg(long, global = true, help_heading = "Global Options", value_enum, default_value_t = OutputFormat::Json)]
    output: OutputFormat,
    /// Print debug logs to stderr
    #[arg(long, global = true, help_heading = "Global Options")]
    debug: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Preview a git repository
    #[command(subcommand)]
    Repository(RepositoryCommand),
}

#[derive(Subcommand)]
enum RepositoryCommand {
    /// List the tracked files as GitHub shows them and count the untracked and gitignored files
    #[command(name = "preview-repository")]
    Preview(PreviewRepositoryArgs),
}

#[derive(Args)]
struct PreviewRepositoryArgs {
    /// Directory inside the git repository to preview
    #[arg(long, default_value = ".")]
    repository_path: PathBuf,
    /// Include the content of the tracked README.md
    #[arg(long, action = ArgAction::SetTrue, overrides_with = "no_readme")]
    readme: bool,
    /// Leave out README.md (default)
    #[arg(long, action = ArgAction::SetTrue, overrides_with = "readme")]
    no_readme: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct PreviewRepositoryReport {
    repository_name: String,
    tracked_files: Vec<String>,
    tracked_file_count: usize,
    untracked_file_count: usize,
    ignored_file_count: usize,
    readme: Option<String>,
}

enum TreeEntry {
    File,
    Dir(BTreeMap<String, TreeEntry>),
}

fn git_command(repository_path: &Path, args: &[&str]) -> Result<String> {
    let output = process::Command::new("git")
        .arg("-C")
        .arg(repository_path)
        .args(args)
        .output()
        .map_err(|error| other(format!("failed to run git: {error}")))?;
    if !output.status.success() {
        return Err(target(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn check_git_repo(repository_path: &Path) -> Result<()> {
    git_command(repository_path, &["rev-parse", "--is-inside-work-tree"])
        .map(|_| ())
        .map_err(|_| {
            target(format!(
                "{} is not inside a git repository",
                repository_path.display()
            ))
        })
}

fn lines(output: &str) -> Vec<String> {
    output
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

fn build_tree(files: &[String]) -> BTreeMap<String, TreeEntry> {
    let mut root: BTreeMap<String, TreeEntry> = BTreeMap::new();
    for file in files {
        let parts: Vec<&str> = file.split('/').collect();
        let mut current = &mut root;
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                current.entry(part.to_string()).or_insert(TreeEntry::File);
            } else {
                current = match current
                    .entry(part.to_string())
                    .or_insert_with(|| TreeEntry::Dir(BTreeMap::new()))
                {
                    TreeEntry::Dir(map) => map,
                    TreeEntry::File => unreachable!(),
                };
            }
        }
    }
    root
}

fn write_tree(tree: &BTreeMap<String, TreeEntry>, prefix: &str, out: &mut String) {
    let entries: Vec<_> = tree.iter().collect();
    for (i, (name, entry)) in entries.iter().enumerate() {
        let is_last = i == entries.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        match entry {
            TreeEntry::Dir(_) => {
                let _ = writeln!(out, "{prefix}{connector}{}", name.bold().cyan());
            }
            TreeEntry::File => {
                let _ = writeln!(out, "{prefix}{connector}{name}");
            }
        }
        if let TreeEntry::Dir(children) = entry {
            let child_prefix = if is_last {
                format!("{prefix}    ")
            } else {
                format!("{prefix}│   ")
            };
            write_tree(children, &child_prefix, out);
        }
    }
}

fn preview(repository_path: &Path, readme: bool) -> Result<PreviewRepositoryReport> {
    check_git_repo(repository_path)?;

    let tracked_files = lines(&git_command(repository_path, &["ls-files"])?);
    let untracked_file_count = lines(&git_command(
        repository_path,
        &["ls-files", "--others", "--exclude-standard"],
    )?)
    .len();
    let ignored_file_count = lines(&git_command(
        repository_path,
        &["ls-files", "--others", "-i", "--exclude-standard"],
    )?)
    .len();

    let repo_root = git_command(repository_path, &["rev-parse", "--show-toplevel"])?;
    let repository_name = Path::new(repo_root.trim())
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| ".".to_string());

    let readme = if readme && tracked_files.iter().any(|f| f == "README.md") {
        Some(
            std::fs::read_to_string(repository_path.join("README.md"))
                .map_err(|error| target(format!("failed to read README.md: {error}")))?,
        )
    } else {
        None
    };

    Ok(PreviewRepositoryReport {
        repository_name,
        tracked_file_count: tracked_files.len(),
        tracked_files,
        untracked_file_count,
        ignored_file_count,
        readme,
    })
}

fn render_text(report: &PreviewRepositoryReport, readme: bool) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{}", report.repository_name.bold().cyan());
    write_tree(&build_tree(&report.tracked_files), "", &mut out);

    out.push('\n');
    let _ = writeln!(
        out,
        "{} {} tracked file(s)",
        "●".green(),
        report.tracked_file_count
    );
    let _ = writeln!(
        out,
        "{} {} untracked file(s)",
        "●".yellow(),
        report.untracked_file_count
    );
    let _ = writeln!(
        out,
        "{} {} gitignored file(s)",
        "●".bright_black(),
        report.ignored_file_count
    );

    if readme {
        out.push('\n');
        match &report.readme {
            Some(content) => {
                let _ = writeln!(
                    out,
                    "{}",
                    "─── README.md (as rendered on GitHub) ───".dimmed()
                );
                let _ = writeln!(out, "{content}");
            }
            None => {
                let _ = writeln!(out, "{}", "No README.md found in tracked files.".dimmed());
            }
        }
    }
    out
}

fn main() {
    let code = match run() {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("error: {error}");
            error.exit_code()
        }
    };
    process::exit(code);
}

fn to_value<T: Serialize>(value: &T) -> Result<Value> {
    serde_json::to_value(value).map_err(|error| other(error.to_string()))
}

fn run() -> Result<()> {
    let mut root = Cli::command();
    root.build();
    let argv = grammar::prepare(&root, std::env::args_os().collect())?;
    let debug = argv.iter().any(|token| token == "--debug");
    if debug {
        eprintln!("debug: argv {:?}", &argv[1..]);
    }

    let cli = match Cli::try_parse_from(&argv) {
        Ok(cli) => cli,
        Err(error) => {
            let code = if error.use_stderr() { 252 } else { 0 };
            let _ = error.print();
            process::exit(code);
        }
    };

    let Command::Repository(RepositoryCommand::Preview(args)) = cli.command;
    let readme = args.readme && !args.no_readme;
    let report = preview(&args.repository_path, readme)?;
    let text = render_text(&report, readme);
    print!(
        "{}",
        output::render(
            &grammar::pascal_case("preview-repository"),
            Response {
                value: to_value(&report)?,
                text: Some(text),
            },
            cli.output
        )?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_nests_directories_under_their_parent() {
        let files = vec![
            "Cargo.toml".to_owned(),
            "src/main.rs".to_owned(),
            "src/output.rs".to_owned(),
        ];
        colored::control::set_override(false);
        let mut out = String::new();
        write_tree(&build_tree(&files), "", &mut out);
        assert_eq!(
            out,
            "├── Cargo.toml\n└── src\n    ├── main.rs\n    └── output.rs\n"
        );
    }
}
