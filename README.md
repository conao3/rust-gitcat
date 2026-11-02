# gitcat

> Preview your repository as GitHub would see it — before you push.

`gitcat` displays the output of `git ls-files` as a GitHub-style directory tree, along with a summary of tracked, untracked, and ignored files. Use it to catch `.gitignore` misconfigurations or accidentally committed secrets before publishing your work.

## Why?

After vibe-coding sessions, AI agents can create many files. Before pushing to GitHub, you want to confirm:

- No secrets accidentally tracked
- No test/temp files committed
- Your `.gitignore` is actually working

Instead of pushing just to see the GitHub file tree, `gitcat` shows you exactly what GitHub would display.

## Installation

```bash
cargo install conao3-gitcat
```

This installs the `gitcat` binary.

## Usage

The CLI takes input in the AWS CLI grammar: `gitcat [global options] <command> <subcommand> [--parameter value ...]`.

### Basic tree view

Run inside any git repository:

```bash
gitcat repository preview-repository --output text
```

**Example output:**

```
rust-gitcat
├── .gitignore
├── Cargo.lock
├── Cargo.toml
├── Makefile
├── flake.lock
├── flake.nix
└── src
    └── main.rs

● 7 tracked file(s)
● 8 untracked file(s)
● 562 gitignored file(s)
```

The summary tells you at a glance:
- 🟢 **tracked** — files Git knows about (what GitHub will show)
- 🟡 **untracked** — files not yet added to Git (not on GitHub)
- ⚫ **gitignored** — files excluded by `.gitignore`

Without `--output text`, gitcat prints the same result as JSON:

```json
{
  "RepositoryName": "rust-gitcat",
  "TrackedFiles": [".gitignore", "Cargo.lock", "Cargo.toml", "Makefile", "flake.lock", "flake.nix", "src/main.rs"],
  "TrackedFileCount": 7,
  "UntrackedFileCount": 8,
  "IgnoredFileCount": 562,
  "Readme": null
}
```

### Show README.md

```bash
gitcat repository preview-repository --readme --output text
```

Displays the tree view followed by the contents of `README.md` (if tracked), so you can preview what GitHub will render on your repository's front page. In JSON the content is in `Readme`.

### Preview another directory

```bash
gitcat repository preview-repository --repository-path ~/src/my-project --output text
```

## Parameters

`gitcat repository preview-repository`:

- `--repository-path <DIR>`: directory inside the git repository to preview (default `.`).
- `--readme` / `--no-readme`: include the content of the tracked `README.md` (default `--no-readme`).

Common to every subcommand:

- A value starting with `file://` is replaced by the content of that file. Read stdin with `file:///dev/stdin`.
- `--cli-input-json file://input.json` passes all parameters as a JSON object keyed by the PascalCase parameter names (`RepositoryPath`, `Readme`). Parameters given on the command line take precedence.
- `--generate-cli-skeleton input|output` prints the input or output skeleton.
- Global options: `--output json|text|table` (default `json`; `text` prints the tree), `--query <JMESPath>`, `--debug`, `--version`.
- `help` works at every level: `gitcat help`, `gitcat repository help`, `gitcat repository preview-repository help`.
- Exit codes: `0` success, `252` syntax or parameter error, `254` the path is not inside a git repository or git failed on it, `255` other failures.

## Development

### Prerequisites

This project uses [Nix flakes](https://nixos.wiki/wiki/Flakes) for reproducible development environments.

```bash
# Enter the development shell (provides cargo, rustc, etc.)
nix develop

# Build
cargo build

# Run (prints the tree of this repository)
cargo run -- repository preview-repository --output text

# Test
cargo test
```

On non-NixOS systems, a standard Rust toolchain works fine:

```bash
cargo build
cargo run -- repository preview-repository --output text
```

## Links

- **crates.io:** https://crates.io/crates/conao3-gitcat
- **GitHub:** https://github.com/conao3/rust-gitcat

## License

MIT
