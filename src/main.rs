mod cli;
mod cov;
mod tui;
use anyhow::{anyhow, Result};
use cli::parse_cli;
use cov::{DefaultReadFilter, DepthProcessor};
use std::path::PathBuf;

fn main() -> Result<()> {
    // parse cli
    let cli = parse_cli()?;
    let bam = cli.bam;
    let region = cli.region;
    let color = cli.color;
    let step_size = cli.step_size;
    let include_flags = cli.include_flags;
    let exclude_flags = cli.exclude_flags;
    let min_mapq = cli.min_mapq;

    // parse region
    let (chrom, start, end) = parse_region(region)?;

    // create read filter and depth processor
    let read_filter = DefaultReadFilter::new(include_flags, exclude_flags, min_mapq);
    let bam_path = PathBuf::from(bam); // check it
    let depth_processer = DepthProcessor::new(bam_path, read_filter);
    let res = depth_processer.process_region(&chrom, start, end)?;

    // get the depth data
    let data: Vec<u64> = res.iter().map(|x| x.depth as u64).collect();
    let legend = format!("{}:{}-{}", chrom, start, end);

    // run tui
    tui::tview(data, start, legend, step_size, color)
}

/// Parse the region string into chrom, start, end; if invalid, return an error
fn parse_region(region: String) -> Result<(String, u32, u32)> {
    let parts: Vec<&str> = region.split(':').collect();
    if parts.len() != 2 {
        return Err(anyhow!("Invalid region format"));
    }
    let chrom = parts[0].to_string();
    let start_end: Vec<&str> = parts[1].split('-').collect();
    if start_end.len() != 2 {
        return Err(anyhow!("Invalid region format"));
    }
    let start = start_end[0].parse::<u32>()?;
    let end = start_end[1].parse::<u32>()?;
    Ok((chrom, start, end))
}
