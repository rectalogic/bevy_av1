use std::{
    env::{self, Args},
    path::{Path, PathBuf},
    process::Command,
};

type DynError = Box<dyn std::error::Error>;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn try_main() -> Result<(), DynError> {
    let mut args = env::args();
    let task = args.nth(1);
    match task.as_deref() {
        Some("doc") => doc(args)?,
        _ => print_help(),
    }
    Ok(())
}

fn print_help() {
    eprintln!(
        "Tasks:

doc            builds documentation
"
    )
}

fn doc(args: Args) -> Result<(), DynError> {
    let args: Vec<String> = args.collect();
    let status = Command::new("rustup")
        .current_dir(project_root())
        .args(
            [
                "run",
                "nightly",
                "cargo",
                "doc",
                "-Zunstable-options",
                "-Zrustdoc-scrape-examples",
            ]
            .iter()
            .copied()
            .chain(args.iter().map(|s| s.as_str())),
        )
        .status()?;

    if !status.success() {
        Err("cargo doc failed")?;
    }

    Ok(())
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}
