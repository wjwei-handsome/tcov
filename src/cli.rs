use anyhow::Result;
use clap::{command, Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "tcov")]
#[command(about = "View coverage data in terminal")]
#[command(long_about = "long_about todo!!!")]
#[command(author, version)]
#[command(
    help_template = "{name} -- {about}\n\nVersion: {version}\n\nAuthors: {author}\
    \n\n{usage-heading} {usage}\n\n{all-args}"
)]
pub struct Cli {
    /// Input bam file with index
    #[arg(short, long, help_heading = Some("Input Options"))]
    pub bam: String,
    /// input region, format: chr:start-end
    #[arg(short, long, help_heading = Some("Input Options"))]
    pub region: String,

    /// Display color for coverage
    #[arg(default_value = "blue", short, long, help_heading = Some("Display Options"))]
    pub color: Color,
    /// Step size for moving the view
    #[arg(default_value = "10", short, long, help_heading = Some("Display Options"))]
    pub step_size: u8,

    /// Included flags
    #[arg(default_value = "0", short, long, help_heading = Some("Filter Options"))]
    pub include_flags: u16,
    /// Excluded flags
    #[arg(default_value = "0", short, long, help_heading = Some("Filter Options"))]
    pub exclude_flags: u16,
    /// Minimum mapping quality
    #[arg(default_value = "0", short, long, help_heading = Some("Filter Options"))]
    pub min_mapq: u8,
}

pub fn parse_cli() -> Result<Cli> {
    let cli = Cli::parse();
    Ok(cli)
}

#[derive(ValueEnum, Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum Color {
    black,
    red,
    green,
    yellow,
    blue,
    magenta,
    cyan,
    gray,
    darkgray,
    lightred,
    lightgreen,
    lightyellow,
    lightblue,
    lightmagenta,
    lightcyan,
    white,
}

// impl to_string for Color
// what a dummy impl HAHAHA
impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Color::black => write!(f, "black"),
            Color::red => write!(f, "red"),
            Color::green => write!(f, "green"),
            Color::yellow => write!(f, "yellow"),
            Color::blue => write!(f, "blue"),
            Color::magenta => write!(f, "magenta"),
            Color::cyan => write!(f, "cyan"),
            Color::gray => write!(f, "gray"),
            Color::darkgray => write!(f, "darkgray"),
            Color::lightred => write!(f, "lightred"),
            Color::lightgreen => write!(f, "lightgreen"),
            Color::lightyellow => write!(f, "lightyellow"),
            Color::lightblue => write!(f, "lightblue"),
            Color::lightmagenta => write!(f, "lightmagenta"),
            Color::lightcyan => write!(f, "lightcyan"),
            Color::white => write!(f, "white"),
        }
    }
}
