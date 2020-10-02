//! Cargo xtask definitions for the wasm-language-server project.

#![deny(clippy::all)]
#![deny(missing_docs)]
#![deny(unsafe_code)]

type Fallible<T> = Result<T, Box<dyn std::error::Error>>;

fn rest(input: &str) -> Fallible<Vec<String>> {
    Ok(input
        .trim_start_matches('\'')
        .trim_end_matches('\'')
        .split_whitespace()
        .map(String::from)
        .collect())
}

fn main() -> Fallible<()> {
    let help = r#"
xtask

USAGE:
    xtask [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information

SUBCOMMANDS:
    build
    check
    clippy
    doc
    format
    help         Prints this message or the help of the given subcommand(s)
    install
    run
    test
"#
    .trim();

    let mut args = pico_args::Arguments::from_env();
    match args.subcommand()?.as_deref() {
        Some("build") => {
            subcommand::cargo::build(args)?;
            return Ok(());
        },
        Some("check") => {
            subcommand::cargo::check(args)?;
            return Ok(());
        },
        Some("clippy") => {
            subcommand::cargo::clippy(args)?;
            return Ok(());
        },
        Some("doc") => {
            subcommand::cargo::doc(args)?;
            return Ok(());
        },
        Some("format") => {
            subcommand::cargo::format(args)?;
            return Ok(());
        },
        Some("help") => {
            println!("{}\n", help);
            return Ok(());
        },
        Some("install") => {
            subcommand::cargo::install(args)?;
            return Ok(());
        },
        Some("run") => {
            subcommand::cargo::run(args)?;
            return Ok(());
        },
        Some("test") => {
            subcommand::cargo::test(args)?;
            return Ok(());
        },
        Some(subcommand) => {
            return Err(format!("unknown subcommand: {}", subcommand).into());
        },
        None => {
            if args.contains(["-h", "--help"]) {
                println!("{}\n", help);
                return Ok(());
            }
        },
    }

    if let Err(pico_args::Error::UnusedArgsLeft(args)) = args.finish() {
        return Err(format!("unrecognized arguments: {}", args.join(" ")).into());
    }

    Ok(())
}

mod metadata {
    use std::path::{Path, PathBuf};

    pub fn cargo() -> crate::Fallible<String> {
        // NOTE: we use the cargo wrapper rather than the binary reported through the "CARGO" environment
        // variable because we need to be able to invoke cargo with different toolchains (e.g., +nightly)
        Ok(String::from("cargo"))
    }

    pub fn project_root() -> PathBuf {
        Path::new(&env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(1)
            .unwrap()
            .to_path_buf()
    }
}

mod subcommand {
    pub mod cargo {
        use crate::metadata;
        use std::process::Command;

        // Run `cargo build` with custom options.
        pub fn build(mut args: pico_args::Arguments) -> crate::Fallible<()> {
            let help = r#"
xtask-build

USAGE:
    xtask build

FLAGS:
    -h, --help       Prints help information
    --rest '...'     Extra arguments to pass to the underlying cargo command
"#
            .trim();

            if args.contains(["-h", "--help"]) {
                println!("{}\n", help);
                return Ok(());
            }

            let cargo = metadata::cargo()?;
            let mut cmd = Command::new(cargo);
            cmd.current_dir(metadata::project_root());
            cmd.env("RUSTFLAGS", "-Dwarnings");
            cmd.args(&["build"]);
            if cfg!(target_os = "macos") {
                cmd.args(&["--features", "metal"]);
            } else {
                cmd.args(&["--features", "vulkan"]);
            }
            if let Some(values) = args.opt_value_from_fn("--rest", crate::rest)? {
                cmd.args(values);
            }
            cmd.status()?;
            Ok(())
        }

        // Run `cargo check` with custom options.
        pub fn check(mut args: pico_args::Arguments) -> crate::Fallible<()> {
            let help = r#"
xtask-check

USAGE:
    xtask check

FLAGS:
    -h, --help       Prints help information
    --rest '...'     Extra arguments to pass to the underlying cargo command
"#
            .trim();

            if args.contains(["-h", "--help"]) {
                println!("{}\n", help);
                return Ok(());
            }

            let cargo = metadata::cargo()?;
            let mut cmd = Command::new(cargo);
            cmd.current_dir(metadata::project_root());
            cmd.env("RUSTFLAGS", "-Dwarnings");
            cmd.args(&["check", "--all-targets", "--workspace"]);
            if cfg!(target_os = "macos") {
                cmd.args(&["--features", "metal"]);
            } else {
                cmd.args(&["--features", "vulkan"]);
            }
            if let Some(values) = args.opt_value_from_fn("--rest", crate::rest)? {
                cmd.args(values);
            }
            cmd.status()?;
            Ok(())
        }

        // Run `cargo clippy` with custom options.
        pub fn clippy(mut args: pico_args::Arguments) -> crate::Fallible<()> {
            let help = r#"
xtask-clippy

USAGE:
    xtask clippy

FLAGS:
    -h, --help       Prints help information
    --rest '...'     Extra arguments to pass to the underlying cargo command
"#
            .trim();

            if args.contains(["-h", "--help"]) {
                println!("{}\n", help);
                return Ok(());
            }

            let cargo = metadata::cargo()?;
            let mut cmd = Command::new(cargo);
            cmd.current_dir(metadata::project_root());
            cmd.args(&["clippy", "--all-targets", "--workspace"]);
            if cfg!(target_os = "macos") {
                cmd.args(&["--features", "metal"]);
            } else {
                cmd.args(&["--features", "vulkan"]);
            }
            if let Some(values) = args.opt_value_from_fn("--rest", crate::rest)? {
                cmd.args(values);
            }
            cmd.args(&["--", "-D", "warnings"]);
            cmd.status()?;
            Ok(())
        }

        // Run `cargo doc` with custom options.
        pub fn doc(mut args: pico_args::Arguments) -> crate::Fallible<()> {
            let help = r#"
xtask-doc

USAGE:
    xtask doc

FLAGS:
    -h, --help       Prints help information
    --rest '...'     Extra arguments to pass to the underlying cargo command
"#
            .trim();

            if args.contains(["-h", "--help"]) {
                println!("{}\n", help);
                return Ok(());
            }

            let cargo = metadata::cargo()?;
            let mut cmd = Command::new(cargo);
            cmd.current_dir(metadata::project_root());
            cmd.args(&["+nightly", "doc"]);
            if let Some(values) = args.opt_value_from_fn("--rest", crate::rest)? {
                cmd.args(values);
            }
            cmd.status()?;
            Ok(())
        }

        // Run `cargo format` with custom options.
        pub fn format(mut args: pico_args::Arguments) -> crate::Fallible<()> {
            let help = r#"
xtask-format

USAGE:
    xtask format

FLAGS:
    -h, --help       Prints help information
    --rest '...'     Extra arguments to pass to the underlying cargo command
"#
            .trim();

            if args.contains(["-h", "--help"]) {
                println!("{}\n", help);
                return Ok(());
            }

            let cargo = metadata::cargo()?;
            let mut cmd = Command::new(cargo);
            cmd.current_dir(metadata::project_root());
            cmd.args(&["+nightly", "fmt", "--all"]);
            if let Some(values) = args.opt_value_from_fn("--rest", crate::rest)? {
                cmd.args(values);
            }
            cmd.status()?;
            Ok(())
        }

        // Run `cargo install` with custom options.
        pub fn install(mut args: pico_args::Arguments) -> crate::Fallible<()> {
            let help = r#"
xtask-install

USAGE:
    xtask install

FLAGS:
    -h, --help       Prints help information
    --rest '...'     Extra arguments to pass to the underlying cargo command
"#
            .trim();

            if args.contains(["-h", "--help"]) {
                println!("{}\n", help);
                return Ok(());
            }

            let cargo = metadata::cargo()?;
            let mut cmd = Command::new(cargo);
            cmd.current_dir(metadata::project_root());
            cmd.args(&["install"]);
            if let Some(values) = args.opt_value_from_fn("--rest", crate::rest)? {
                cmd.args(values);
            }
            cmd.status()?;

            Ok(())
        }

        // Run `cargo run` with custom options.
        pub fn run(mut args: pico_args::Arguments) -> crate::Fallible<()> {
            let help = r#"
xtask-run

USAGE:
    xtask run

FLAGS:
    -h, --help       Prints help information
    --rest '...'     Extra arguments to pass to the underlying cargo command
"#
            .trim();

            if args.contains(["-h", "--help"]) {
                println!("{}\n", help);
                return Ok(());
            }

            let cargo = metadata::cargo()?;
            let mut cmd = Command::new(cargo);
            cmd.current_dir(metadata::project_root());
            cmd.env("RUSTFLAGS", "-Dwarnings");
            cmd.args(&["run"]);
            if cfg!(target_os = "macos") {
                cmd.args(&["--features", "metal"]);
            } else {
                cmd.args(&["--features", "vulkan"]);
            }
            if let Some(values) = args.opt_value_from_fn("--rest", crate::rest)? {
                cmd.args(values);
            }
            cmd.status()?;
            Ok(())
        }

        // Run `cargo test` with custom options.
        pub fn test(mut args: pico_args::Arguments) -> crate::Fallible<()> {
            let help = r#"
xtask-test

USAGE:
    xtask test

FLAGS:
    -h, --help       Prints help information
    --rest '...'     Extra arguments to pass to the underlying cargo command
"#
            .trim();

            if args.contains(["-h", "--help"]) {
                println!("{}\n", help);
                return Ok(());
            }

            let cargo = metadata::cargo()?;
            let mut cmd = Command::new(cargo);
            cmd.current_dir(metadata::project_root());
            cmd.env("RUSTFLAGS", "-Dwarnings");
            cmd.args(&["test", "--all-targets", "--workspace"]);
            if cfg!(target_os = "macos") {
                cmd.args(&["--features", "metal"]);
            } else {
                cmd.args(&["--features", "vulkan"]);
            }
            if let Some(values) = args.opt_value_from_fn("--rest", crate::rest)? {
                cmd.args(values);
            }
            cmd.status()?;

            Ok(())
        }
    }
}
