//! Writes a standalone HTML review sheet that draws every model from its figure, in the light theme on white,
//! and prints the item counts of each model per detail level.
//!
//! Usage: `cargo run -p acan-iso --example sheet -- <out.html>`

use std::fmt::Write as _;
use std::process::ExitCode;

use acan_iso::{Detail, Figure, Fill, Ink, Item, Kind, footprint, model};

/// Pixels per plan unit in the sheet.
const SCALE: f32 = 3.0;
/// Margin around each figure, in plan units.
const PAD: f32 = 4.0;

const KINDS: [Kind; 7] = [
    Kind::Assembler,
    Kind::Crate,
    Kind::Intake,
    Kind::Gate,
    Kind::Splitter,
    Kind::Tank,
    Kind::Valve,
];

fn main() -> ExitCode {
    let Some(out) = std::env::args().nth(1) else {
        eprintln!("usage: sheet <out.html>");
        return ExitCode::FAILURE;
    };
    let mut cards = String::new();
    println!("kind        fills  lines   dots  dotpts   always     near    close");
    for kind in KINDS {
        let figure = model(kind);
        let counts = Counts::of(&figure);
        println!(
            "{:<10} {:>6} {:>6} {:>6} {:>7} {:>8} {:>8} {:>8}",
            format!("{kind:?}"),
            counts.fills,
            counts.lines,
            counts.dots,
            counts.dot_points,
            counts.visible(Detail::Always),
            counts.visible(Detail::Near),
            counts.visible(Detail::Close),
        );
        let fp = footprint(kind);
        let note = if matches!(kind, Kind::Tank | Kind::Valve) {
            " &middot; no drawing yet, draws as the crate"
        } else {
            ""
        };
        let _ = writeln!(
            cards,
            "<figure><svg xmlns=\"http://www.w3.org/2000/svg\" {}>{}</svg><figcaption>{:?} &middot; footprint {} x {} x {} &middot; close items {}{note}</figcaption></figure>",
            size_attrs(&figure),
            svg_body(&figure),
            kind,
            fp.w,
            fp.d,
            fp.top,
            counts.visible(Detail::Close),
        );
    }
    let html = page(&cards);
    if let Err(error) = std::fs::write(&out, html) {
        eprintln!("cannot write {out}: {error}");
        return ExitCode::FAILURE;
    }
    println!("wrote {out}");
    ExitCode::SUCCESS
}

#[derive(Debug, Default)]
struct Counts {
    fills: usize,
    lines: usize,
    dots: usize,
    dot_points: usize,
    always: usize,
    near: usize,
    close: usize,
    lines_by_detail: [usize; 3],
    dots_by_detail: [usize; 3],
}

impl Counts {
    fn of(figure: &Figure) -> Self {
        let mut counts = Self::default();
        for item in &figure.items {
            match item {
                Item::Fill { .. } => counts.fills += 1,
                Item::Line { detail, .. } => {
                    counts.lines += 1;
                    counts.lines_by_detail[detail_index(*detail)] += 1;
                }
                Item::Dots { points, detail, .. } => {
                    counts.dots += 1;
                    counts.dot_points += points.len();
                    counts.dots_by_detail[detail_index(*detail)] += 1;
                }
            }
        }
        counts.always = counts.fills + counts.lines_by_detail[0] + counts.dots_by_detail[0];
        counts.near = counts.always + counts.lines_by_detail[1] + counts.dots_by_detail[1];
        counts.close = counts.near + counts.lines_by_detail[2] + counts.dots_by_detail[2];
        counts
    }

    /// Items drawn at `level`: fills always, and lines and dots whose detail is at or below it.
    fn visible(&self, level: Detail) -> usize {
        match level {
            Detail::Always => self.always,
            Detail::Near => self.near,
            Detail::Close => self.close,
        }
    }
}

fn detail_index(detail: Detail) -> usize {
    match detail {
        Detail::Always => 0,
        Detail::Near => 1,
        Detail::Close => 2,
    }
}

fn size_attrs(figure: &Figure) -> String {
    let [x0, y0, x1, y1] = figure.bounds;
    let (w, h) = (x1 - x0 + 2.0 * PAD, y1 - y0 + 2.0 * PAD);
    format!(
        "viewBox=\"{:.3} {:.3} {:.3} {:.3}\" width=\"{:.0}\" height=\"{:.0}\"",
        x0 - PAD,
        y0 - PAD,
        w,
        h,
        w * SCALE,
        h * SCALE
    )
}

fn svg_body(figure: &Figure) -> String {
    let mut out = String::new();
    for item in &figure.items {
        match item {
            Item::Fill { ring, tone } => {
                let (colour, opacity) = fill_paint(*tone);
                let _ = writeln!(
                    out,
                    "<path d=\"{}Z\" fill=\"{colour}\" fill-opacity=\"{opacity}\"/>",
                    path(ring)
                );
            }
            Item::Line {
                points,
                closed,
                tone,
                width,
                ..
            } => {
                let close = if *closed { "Z" } else { "" };
                let _ = writeln!(
                    out,
                    "<path d=\"{}{close}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{width}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",
                    path(points),
                    ink_colour(*tone),
                );
            }
            Item::Dots {
                points,
                radius,
                tone,
                ..
            } => {
                for [x, y] in points {
                    let _ = writeln!(
                        out,
                        "<circle cx=\"{x:.3}\" cy=\"{y:.3}\" r=\"{radius}\" fill=\"{}\"/>",
                        ink_colour(*tone)
                    );
                }
            }
        }
    }
    out
}

fn path(points: &[[f32; 2]]) -> String {
    let mut out = String::new();
    for (index, [x, y]) in points.iter().enumerate() {
        let command = if index == 0 { 'M' } else { 'L' };
        let _ = write!(out, "{command}{x:.3} {y:.3}");
    }
    out
}

/// Light theme, from the kit's iso.css. The renderer owns the real palette; this sheet only checks the drawing.
fn fill_paint(tone: Fill) -> (&'static str, f32) {
    match tone {
        Fill::Paper => ("#fbfbfa", 1.0),
        Fill::Face | Fill::Top | Fill::LitTop => ("#ffffff", 1.0),
        Fill::Shade0 => ("#dcdcd9", 1.0),
        Fill::Shade1 => ("#e5e5e2", 1.0),
        Fill::Shade2 => ("#ededea", 1.0),
        Fill::Shade3 => ("#f4f4f1", 1.0),
        Fill::LitShade => ("#eeeeeb", 1.0),
        Fill::Halo => ("#8a8a80", 0.32),
    }
}

/// Dots take the colour of their ink. The models use no agent slot, so `Agent` is drawn grey here.
fn ink_colour(tone: Ink) -> &'static str {
    match tone {
        Ink::Hi => "#26262a",
        Ink::Mid | Ink::Agent(_) => "#8a8a8f",
        Ink::Lo => "#b4b4b8",
        Ink::Faint => "#d4d4d6",
        Ink::Lit => "#000000",
        Ink::Red => "#e5484d",
        Ink::Green => "#30a46c",
        Ink::Blue => "#3e63dd",
    }
}

fn page(cards: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>acan-iso models</title>\n\
         <style>\n\
         html, body {{ background: #ffffff; color: #26262a; margin: 0; }}\n\
         body {{ font-family: system-ui, sans-serif; padding: 24px; }}\n\
         h1 {{ font-size: 18px; font-weight: 600; margin: 0 0 16px; }}\n\
         .grid {{ display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 24px; }}\n\
         figure {{ margin: 0; border: 1px solid #ededea; border-radius: 6px; padding: 12px; }}\n\
         svg {{ width: 100%; height: auto; display: block; }}\n\
         figcaption {{ font-size: 13px; color: #8a8a8f; margin-top: 8px; }}\n\
         </style>\n</head>\n<body>\n<h1>acan-iso models, drawn from their figures</h1>\n\
         <div class=\"grid\">\n{cards}</div>\n</body>\n</html>\n"
    )
}
