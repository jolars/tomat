use clap::CommandFactory;
use clap_mangen::Man;
use std::fs;
use std::io::Result;
use std::path::PathBuf;

// Include just the CLI module
#[path = "src/cli.rs"]
#[allow(dead_code)]
mod cli;

fn main() -> Result<()> {
    // Generate man page
    generate_man_page()?;

    // Embed icon file
    embed_icon_file()?;

    Ok(())
}

fn generate_man_page() -> Result<()> {
    // Create man directory if it doesn't exist
    let out_dir = PathBuf::from("target/man");
    fs::create_dir_all(&out_dir)?;

    // Generate main man page from the Cli struct
    let cmd = cli::Cli::command();
    let man = Man::new(cmd);
    let mut buffer = Vec::new();
    man.render(&mut buffer)?;
    fs::write(out_dir.join("tomat.1"), buffer)?;

    println!("cargo:rerun-if-changed=src/cli.rs");
    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}

fn embed_icon_file() -> Result<()> {
    // Tell Cargo to embed the icon file and rebuild if it changes
    println!("cargo:rerun-if-changed=assets/icon.png");
    Ok(())
}
