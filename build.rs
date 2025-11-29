use clap::CommandFactory;
use clap::ValueEnum;
use clap_mangen::Man;
use std::fs;
use std::io::Result;
use std::path::PathBuf;

// Include just the CLI module
#[path = "src/cli.rs"]
#[allow(dead_code)]
mod cli;

fn main() -> Result<()> {
    generate_man_page()?;
    generate_images()?;
    embed_icon_file()?;
    generate_completions()?;
    Ok(())
}

fn generate_man_page() -> Result<()> {
    // Create man directory if it doesn't exist
    let out_dir = PathBuf::from("target/man");
    fs::create_dir_all(&out_dir)?;

    // Generate main man page and all subcommand pages (like git/cargo do)
    let cmd = cli::Cli::command();

    // Generate main page
    let man = Man::new(cmd.clone());
    let mut buffer = Vec::new();
    man.render(&mut buffer)?;
    fs::write(out_dir.join("tomat.1"), buffer)?;

    // Generate pages for each top-level subcommand
    for subcommand in cmd.get_subcommands() {
        let subcommand_name = subcommand.get_name();
        if subcommand_name == "help" {
            continue; // Skip help command
        }

        let name = format!("tomat-{}", subcommand_name);
        let man = Man::new(subcommand.clone()).title(&name);
        let mut buffer = Vec::new();
        man.render(&mut buffer)?;

        // Post-process to fix nested subcommand references (daemon\-start -> tomat\-daemon\-start)
        let content = String::from_utf8_lossy(&buffer);
        let fixed_content = content.replace(
            &format!("{}\\-", subcommand_name),
            &format!("tomat\\-{}\\-", subcommand_name),
        );

        fs::write(
            out_dir.join(format!("{}.1", name)),
            fixed_content.as_bytes(),
        )?;

        // Generate pages for nested subcommands (e.g., daemon start -> tomat-daemon-start)
        for nested in subcommand.get_subcommands() {
            let nested_name = nested.get_name();
            if nested_name == "help" {
                continue;
            }

            let full_name = format!("tomat-{}-{}", subcommand_name, nested_name);
            let man = Man::new(nested.clone()).title(&full_name);
            let mut buffer = Vec::new();
            man.render(&mut buffer)?;

            // Post-process nested pages: fix NAME and SYNOPSIS sections
            let content = String::from_utf8_lossy(&buffer);
            let fixed_content = content
                .replace(
                    &format!("{} \\-", nested_name),
                    &format!("{} \\-", full_name),
                )
                .replace(
                    &format!("\\fB{}\\fR", nested_name),
                    &format!("\\fBtomat {} {}\\fR", subcommand_name, nested_name),
                );

            fs::write(
                out_dir.join(format!("{}.1", full_name)),
                fixed_content.as_bytes(),
            )?;
        }
    }

    println!("cargo:rerun-if-changed=src/cli.rs");
    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}

fn generate_images() -> Result<()> {
    let svg_path = "images/logo.svg";

    // Only generate if SVG exists
    if !std::path::Path::new(svg_path).exists() {
        return Ok(());
    }

    println!("cargo:rerun-if-changed={}", svg_path);
    println!("cargo:rerun-if-changed=images/logo-text.svg");

    // Read SVG content
    let svg_data = fs::read_to_string(svg_path)?;

    // Create resvg tree
    let tree = match resvg::usvg::Tree::from_str(&svg_data, &resvg::usvg::Options::default()) {
        Ok(tree) => tree,
        Err(_e) => {
            return Ok(());
        }
    };

    // Generate notification icon (48x48)
    generate_icon(&tree, "assets/icon.png", 48)?;

    // Generate documentation logo (256x256)
    generate_icon(&tree, "images/logo.png", 256)?;

    // Generate social media image (1280x640) with text
    generate_og_image(&tree, "images/og.png")?;

    Ok(())
}

fn generate_icon(tree: &resvg::usvg::Tree, output_path: &str, size: u32) -> Result<()> {
    // Create output directory if it doesn't exist
    if let Some(parent) = std::path::Path::new(output_path).parent() {
        fs::create_dir_all(parent)?;
    }

    // Create transform to scale to target size
    let svg_size = tree.size();
    let scale = size as f32 / svg_size.width().max(svg_size.height());

    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);

    // Create pixmap
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size)
        .ok_or_else(|| std::io::Error::other("Failed to create pixmap"))?;

    // Clear with transparent background
    pixmap.fill(resvg::tiny_skia::Color::TRANSPARENT);

    // Center the image
    let x_offset = (size as f32 - svg_size.width() * scale) / 2.0;
    let y_offset = (size as f32 - svg_size.height() * scale) / 2.0;
    let center_transform = resvg::tiny_skia::Transform::from_translate(x_offset, y_offset);
    let final_transform = center_transform.pre_concat(transform);

    // Render
    resvg::render(tree, final_transform, &mut pixmap.as_mut());

    // Save PNG
    pixmap
        .save_png(output_path)
        .map_err(|e| std::io::Error::other(format!("Failed to save PNG: {}", e)))?;

    println!("Generated: {}", output_path);
    Ok(())
}

fn generate_og_image(tree: &resvg::usvg::Tree, output_path: &str) -> Result<()> {
    // Create output directory if it doesn't exist
    if let Some(parent) = std::path::Path::new(output_path).parent() {
        fs::create_dir_all(parent)?;
    }

    let text_svg_path = "images/logo-text.svg";
    let text_tree = if std::path::Path::new(text_svg_path).exists() {
        match fs::read_to_string(text_svg_path) {
            Ok(svg_data) => {
                resvg::usvg::Tree::from_str(&svg_data, &resvg::usvg::Options::default()).ok()
            }
            Err(_e) => None,
        }
    } else {
        None
    };

    const WIDTH: u32 = 1280;
    const HEIGHT: u32 = 640;

    // Create pixmap
    let mut pixmap = resvg::tiny_skia::Pixmap::new(WIDTH, HEIGHT)
        .ok_or_else(|| std::io::Error::other("Failed to create pixmap"))?;

    // Fill with gradient background
    let gradient = resvg::tiny_skia::LinearGradient::new(
        resvg::tiny_skia::Point::from_xy(0.0, 0.0),
        resvg::tiny_skia::Point::from_xy(WIDTH as f32, HEIGHT as f32),
        vec![
            resvg::tiny_skia::GradientStop::new(
                0.0,
                resvg::tiny_skia::Color::from_rgba8(150, 150, 150, 255),
            ),
            resvg::tiny_skia::GradientStop::new(
                1.0,
                resvg::tiny_skia::Color::from_rgba8(200, 200, 200, 255),
            ),
        ],
        resvg::tiny_skia::SpreadMode::Pad,
        resvg::tiny_skia::Transform::identity(),
    )
    .ok_or_else(|| std::io::Error::other("Failed to create gradient"))?;

    let paint = resvg::tiny_skia::Paint {
        shader: gradient,
        ..Default::default()
    };

    pixmap.fill_rect(
        resvg::tiny_skia::Rect::from_xywh(0.0, 0.0, WIDTH as f32, HEIGHT as f32).unwrap(),
        &paint,
        resvg::tiny_skia::Transform::identity(),
        None,
    );

    // Scale and position main logo (left side)
    let logo_size = 410.0;
    let svg_size = tree.size();
    let logo_scale = logo_size / svg_size.width().max(svg_size.height());

    let logo_x = 95.0;
    let logo_y = (HEIGHT as f32 - logo_size) / 2.0;

    let logo_transform = resvg::tiny_skia::Transform::from_translate(logo_x, logo_y).pre_concat(
        resvg::tiny_skia::Transform::from_scale(logo_scale, logo_scale),
    );

    // Render main logo
    resvg::render(tree, logo_transform, &mut pixmap.as_mut());

    // Add text if available (right side)
    if let Some(text_tree) = text_tree {
        // Scale and position text
        let text_scale = 2.4;
        let text_x = 560.0;
        let text_y = 260.0;

        let text_transform = resvg::tiny_skia::Transform::from_translate(text_x, text_y)
            .pre_concat(resvg::tiny_skia::Transform::from_scale(
                text_scale, text_scale,
            ));

        // Render text as-is
        resvg::render(&text_tree, text_transform, &mut pixmap.as_mut());

        println!("Generated: {} (with logo and text from SVG)", output_path);
    }

    // Save PNG
    pixmap
        .save_png(output_path)
        .map_err(|e| std::io::Error::other(format!("Failed to save PNG: {}", e)))?;

    Ok(())
}

fn embed_icon_file() -> Result<()> {
    // Tell Cargo to embed the icon file and rebuild if it changes
    println!("cargo:rerun-if-changed=assets/icon.png");
    Ok(())
}

fn generate_completions() -> Result<()> {
    // Create man directory if it doesn't exist
    let out_dir = PathBuf::from("target/completions");
    fs::create_dir_all(&out_dir)?;

    // Generate main man page and all subcommand pages (like git/cargo do)
    let mut cmd = cli::Cli::command();

    for &shell in clap_complete::Shell::value_variants() {
        clap_complete::generate_to(shell, &mut cmd, "tomat", &out_dir)?;
    }

    println!("cargo:rerun-if-changed=src/cli.rs");
    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
