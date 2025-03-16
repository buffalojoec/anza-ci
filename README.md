# Anza CI CLI

Anza engineers maintain a large collection of on-chain Solana programs, most of
which follow the same repository pattern.

Currently, we copy-paste a folder of scripts across each repository, and
changes in one repository may not cascade to all repositories without annoying
manual procedures.

At the most fundamental level, we simply require a way to call `cargo` commands
and provide repository-scoped defaults for toolchain, `cargo` arguments, and
command arguments.

We simply require a way to call `cargo` commands and provide repository-scoped
defaults for toolchain, `cargo` arguments, and command arguments.

## About This Tool

This tool allows developers to configure command aliases and default values (
such as toolchains, `cargo` arguments, and command arguments) in a
configuration file named `Anza.toml`.

```toml
[[template]]
name = "format"
extra-args = ["--check"]
subcommand = "fmt"
toolchain = "nightly-2024-11-22"

[[template]]
name = "lint"
args = [
    "-Zunstable-options",
    "--all-features",
    "--all-targets",
]
extra-args = [
    "--deny=warnings",
    "--deny=clippy::arithmetic_side_effects",
]
subcommand = "clippy"
toolchain = "nightly-2024-11-22"


[[alias]]
name = "fmt-program"
manifest = "program/Cargo.toml"
template = "format"

[[alias]]
name = "lint-program"
manifest = "program/Cargo.toml"
template = "lint"
```

The CLI will vacuum up this TOML file and use it to vend aliases, which apply
all of the configured values in the file.

```
anza-ci fmt-program
```

In the situation where you need to avoid using an alias, you can just use
`cargo` directly, or you can invoke it through `anza-ci`.

```
anza-ci cargo [+<toolchain>] <command> [<cargo-args>] -- [<command-args>]
```
