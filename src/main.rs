use std::collections::BTreeMap;
use std::process::Command;

use anyhow::{Context, Result, bail};
use clap::Parser;
use colored::Colorize;

#[derive(Parser)]
#[command(name = "gitcat", about = "Preview your repository as GitHub would see it")]
struct Cli {
    #[arg(long, help = "Show README.md content")]
    readme: bool,
}

enum TreeEntry {
    File,
    Dir(BTreeMap<String, TreeEntry>),
}

fn git_command(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .context("failed to run git")?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn check_git_repo() -> Result<()> {
    git_command(&["rev-parse", "--is-inside-work-tree"])?;
    Ok(())
}

fn build_tree(files: &[&str]) -> BTreeMap<String, TreeEntry> {
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

fn print_tree(tree: &BTreeMap<String, TreeEntry>, prefix: &str) {
    let entries: Vec<_> = tree.iter().collect();
    for (i, (name, entry)) in entries.iter().enumerate() {
        let is_last = i == entries.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        match entry {
            TreeEntry::Dir(_) => {
                println!("{prefix}{connector}{}", name.bold().cyan());
            }
            TreeEntry::File => {
                println!("{prefix}{connector}{name}");
            }
        }
        if let TreeEntry::Dir(children) = entry {
            let child_prefix = if is_last {
                format!("{prefix}    ")
            } else {
                format!("{prefix}│   ")
            };
            print_tree(children, &child_prefix);
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Err(_) = check_git_repo() {
        eprintln!(
            "{} Not a git repository. Run this command inside a git repo.",
            "error:".red().bold()
        );
        std::process::exit(1);
    }

    let tracked = git_command(&["ls-files"])?;
    let tracked_files: Vec<&str> = tracked.lines().filter(|l| !l.is_empty()).collect();

    let untracked = git_command(&["ls-files", "--others", "--exclude-standard"])?;
    let untracked_count = untracked.lines().filter(|l| !l.is_empty()).count();

    let ignored = git_command(&["ls-files", "--others", "-i", "--exclude-standard"])?;
    let ignored_count = ignored.lines().filter(|l| !l.is_empty()).count();

    let repo_root = git_command(&["rev-parse", "--show-toplevel"])?;
    let repo_name = std::path::Path::new(repo_root.trim())
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| ".".to_string());

    println!("{}", repo_name.bold().cyan());
    let tree = build_tree(&tracked_files);
    print_tree(&tree, "");

    println!();
    println!(
        "{} {} tracked file(s)",
        "●".green(),
        tracked_files.len()
    );
    println!(
        "{} {} untracked file(s)",
        "●".yellow(),
        untracked_count
    );
    println!(
        "{} {} gitignored file(s)",
        "●".bright_black(),
        ignored_count
    );

    if cli.readme {
        if tracked_files.iter().any(|f| *f == "README.md") {
            let content = std::fs::read_to_string("README.md")
                .context("failed to read README.md")?;
            println!();
            println!("{}", "─── README.md (as rendered on GitHub) ───".dimmed());
            println!("{content}");
        } else {
            println!();
            println!("{}", "No README.md found in tracked files.".dimmed());
        }
    }

    Ok(())
}
