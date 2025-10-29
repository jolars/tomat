use clap::CommandFactory;
use clap_mangen::Man;
use std::fs;
use std::io::Result;
use std::path::{Path, PathBuf};

// Include just the CLI module
#[path = "src/cli.rs"]
#[allow(dead_code)]
mod cli;

fn main() -> Result<()> {
    // Generate man page
    generate_man_page()?;

    // Generate sound files
    generate_sound_files()?;

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

fn generate_sound_files() -> Result<()> {
    let sounds_dir = Path::new("assets/sounds");

    // Create sounds directory if it doesn't exist
    if !sounds_dir.exists() {
        fs::create_dir_all(sounds_dir)?;
    }

    // Check if sound files already exist
    let work_to_break = sounds_dir.join("work-to-break.wav");
    let break_to_work = sounds_dir.join("break-to-work.wav");
    let work_to_long_break = sounds_dir.join("work-to-long-break.wav");

    if work_to_break.exists() && break_to_work.exists() && work_to_long_break.exists() {
        println!("cargo:rerun-if-changed=assets/sounds/");
        return Ok(()); // Sound files already exist
    }

    // Generate placeholder WAV files to prevent compilation errors
    // Users can replace these with their own sound files

    if !work_to_break.exists() {
        create_placeholder_wav(&work_to_break)?;
    }
    if !break_to_work.exists() {
        create_placeholder_wav(&break_to_work)?;
    }
    if !work_to_long_break.exists() {
        create_placeholder_wav(&work_to_long_break)?;
    }

    println!("cargo:rerun-if-changed=assets/sounds/");
    Ok(())
}

fn create_placeholder_wav(path: &Path) -> Result<()> {
    // Create a minimal valid WAV file header (44 bytes) with no audio data
    // This prevents compilation errors when sound files don't exist
    let wav_header = [
        // RIFF header
        0x52, 0x49, 0x46, 0x46, // "RIFF"
        0x24, 0x00, 0x00, 0x00, // File size - 8 (36 bytes)
        0x57, 0x41, 0x56, 0x45, // "WAVE"
        // fmt subchunk
        0x66, 0x6D, 0x74, 0x20, // "fmt "
        0x10, 0x00, 0x00, 0x00, // Subchunk size (16)
        0x01, 0x00, // Audio format (PCM)
        0x01, 0x00, // Number of channels (1)
        0x44, 0xAC, 0x00, 0x00, // Sample rate (44100)
        0x44, 0xAC, 0x00, 0x00, // Byte rate
        0x01, 0x00, // Block align
        0x08, 0x00, // Bits per sample (8)
        // data subchunk
        0x64, 0x61, 0x74, 0x61, // "data"
        0x00, 0x00, 0x00, 0x00, // Data size (0 - silent)
    ];

    fs::write(path, wav_header)?;
    Ok(())
}
