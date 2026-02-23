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

### Basic tree view

Run inside any git repository:

```bash
gitcat
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

### Show README.md

```bash
gitcat --readme
```

Displays the tree view followed by the contents of `README.md` (if tracked), so you can preview what GitHub will render on your repository's front page.

## Options

```
Usage: gitcat [OPTIONS]

Options:
      --readme  Show README.md content
  -h, --help    Print help
```

## Development

### Prerequisites

This project uses [Nix flakes](https://nixos.wiki/wiki/Flakes) for reproducible development environments.

```bash
# Enter the development shell (provides cargo, rustc, etc.)
nix develop

# Build
cargo build

# Run
cargo run

# Test
cargo test
```

On non-NixOS systems, a standard Rust toolchain works fine:

```bash
cargo build
cargo run
```

## Links

- **crates.io:** https://crates.io/crates/conao3-gitcat
- **GitHub:** https://github.com/conao3/rust-gitcat

## License

MIT
