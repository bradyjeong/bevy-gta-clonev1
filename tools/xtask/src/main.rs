//! Development automation tool for the Amp game engine

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::process::Command;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the full CI pipeline locally
    Ci,
    /// Format all code
    Fmt,
    /// Run linting
    Lint,
    /// Run tests
    Test,
    /// Generate documentation
    Doc,
    /// Validate documentation
    DocValidate,
    /// Check all crates
    Check,
    /// Run coverage analysis
    Coverage,
    /// Bump version
    BumpVersion {
        /// Version type to bump
        #[arg(value_enum)]
        version_type: VersionType,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum VersionType {
    Patch,
    Minor,
    Major,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ci => run_ci(),
        Commands::Fmt => run_fmt(),
        Commands::Lint => run_lint(),
        Commands::Test => run_test(),
        Commands::Doc => run_doc(),
        Commands::DocValidate => run_doc_validate(),
        Commands::Check => run_check(),
        Commands::Coverage => run_coverage(),
        Commands::BumpVersion { version_type } => bump_version(version_type),
    }
}

fn run_ci() -> Result<()> {
    println!("Running full CI pipeline...");

    run_fmt()?;
    run_lint()?;
    run_test()?;
    run_coverage()?;
    run_doc()?;
    run_doc_validate()?;

    println!("✅ CI pipeline completed successfully!");
    Ok(())
}

fn run_fmt() -> Result<()> {
    println!("Formatting code...");

    let output = Command::new("cargo")
        .args(["fmt", "--all", "--", "--check"])
        .output()?;

    if !output.status.success() {
        println!("Running cargo fmt to fix formatting issues...");
        Command::new("cargo").args(["fmt", "--all"]).status()?;
    }

    println!("✅ Code formatted");
    Ok(())
}

fn run_lint() -> Result<()> {
    println!("Running clippy...");

    let status = Command::new("cargo")
        .args([
            "clippy",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ])
        .status()?;

    if !status.success() {
        anyhow::bail!("Clippy found issues");
    }

    println!("✅ Linting passed");
    Ok(())
}

fn run_test() -> Result<()> {
    println!("Running tests...");

    let status = Command::new("cargo")
        .args(["test", "--workspace", "--all-features"])
        .status()?;

    if !status.success() {
        anyhow::bail!("Tests failed");
    }

    println!("✅ Tests passed");
    Ok(())
}

fn run_doc() -> Result<()> {
    println!("Generating documentation...");

    let status = Command::new("cargo")
        .args(["doc", "--workspace", "--no-deps", "--all-features"])
        .status()?;

    if !status.success() {
        anyhow::bail!("Documentation generation failed");
    }

    println!("✅ Documentation generated");
    Ok(())
}

fn run_doc_validate() -> Result<()> {
    println!("Validating documentation...");

    // Check for missing documentation
    let status = Command::new("cargo")
        .args(["doc", "--workspace", "--no-deps", "--all-features"])
        .env("RUSTDOCFLAGS", "-D missing_docs")
        .status()?;

    if !status.success() {
        anyhow::bail!("Documentation validation failed - missing docs");
    }

    // Check markdown files exist and are not empty
    let markdown_files = [
        "README.md",
        "CONTRIBUTING.md",
        "AGENT.md",
        "docs/README.md",
        "docs/architecture/README.md",
        "docs/guides/development.md",
        "docs/adr/README.md",
    ];

    for file in &markdown_files {
        let path = std::path::Path::new(file);
        if !path.exists() {
            anyhow::bail!("Required markdown file missing: {}", file);
        }

        let content = std::fs::read_to_string(path)?;
        if content.trim().is_empty() {
            anyhow::bail!("Markdown file is empty: {}", file);
        }
    }

    println!("✅ Documentation validation passed");
    Ok(())
}

fn run_check() -> Result<()> {
    println!("Checking all crates...");

    let status = Command::new("cargo")
        .args(["check", "--workspace", "--all-targets", "--all-features"])
        .status()?;

    if !status.success() {
        anyhow::bail!("Check failed");
    }

    println!("✅ All crates check passed");
    Ok(())
}

fn run_coverage() -> Result<()> {
    println!("Running coverage analysis...");

    // Install cargo-llvm-cov if not available
    let install_status = Command::new("cargo")
        .args(["install", "cargo-llvm-cov"])
        .status()?;

    if !install_status.success() {
        println!("cargo-llvm-cov already installed or installation failed");
    }

    // Run coverage with 70% threshold
    let status = Command::new("cargo")
        .args([
            "llvm-cov",
            "--workspace",
            "--all-features",
            "--lcov",
            "--output-path",
            "lcov.info",
            "--fail-under-lines",
            "70",
        ])
        .status()?;

    if !status.success() {
        anyhow::bail!("Coverage below 70% threshold");
    }

    println!("✅ Coverage analysis passed (≥70%)");
    Ok(())
}

fn bump_version(version_type: VersionType) -> Result<()> {
    let version_arg = match version_type {
        VersionType::Patch => "patch",
        VersionType::Minor => "minor",
        VersionType::Major => "major",
    };

    println!("Bumping {version_arg} version...");

    // This is a stub - in a real implementation you'd use cargo-edit or similar
    println!("Version bump for {version_arg} - implementation needed");

    Ok(())
}
