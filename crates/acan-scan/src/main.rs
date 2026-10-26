use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

const USAGE: &str = "usage: acan-scan [--summary] <repo-path>";

struct Args {
    root: PathBuf,
    summary: bool,
}

impl Args {
    fn parse(args: impl Iterator<Item = String>) -> Result<Self> {
        let mut root = None;
        let mut summary = false;
        for arg in args {
            if arg == "--summary" {
                summary = true;
            } else if arg.starts_with('-') {
                bail!("unknown flag {arg}\n{USAGE}");
            } else if root.replace(PathBuf::from(arg)).is_some() {
                bail!("expected one repo path\n{USAGE}");
            }
        }
        let root = root.with_context(|| USAGE)?;
        Ok(Self { root, summary })
    }
}

fn main() -> Result<()> {
    let args = Args::parse(std::env::args().skip(1))?;
    let started = Instant::now();
    let plant =
        acan_scan::scan(&args.root).with_context(|| format!("scanning {}", args.root.display()))?;
    let elapsed = started.elapsed();

    let mut out = io::stdout().lock();
    if args.summary {
        write_summary(&mut out, &plant, elapsed)?;
    } else {
        serde_json::to_writer_pretty(&mut out, &plant)?;
        writeln!(out)?;
    }
    Ok(())
}

fn write_summary(out: &mut impl Write, plant: &acan_scan::Plant, elapsed: Duration) -> Result<()> {
    let symbols: usize = plant.files.iter().map(|file| file.symbols.len()).sum();
    let exported: usize = plant
        .files
        .iter()
        .flat_map(|file| &file.symbols)
        .filter(|symbol| symbol.exported)
        .count();
    let imports: usize = plant.files.iter().map(|file| file.imports.len()).sum();
    writeln!(out, "root: {}", plant.root.display())?;
    writeln!(out, "files: {}", plant.files.len())?;
    writeln!(out, "machines: {}", plant.machines.len())?;
    writeln!(out, "belts: {}", plant.belts.len())?;
    writeln!(out, "symbols: {symbols} ({exported} exported)")?;
    writeln!(out, "imports: {imports}")?;
    writeln!(out, "time: {:.1} ms", elapsed.as_secs_f64() * 1000.0)?;
    Ok(())
}
