// Reads the sfnt table directory of a TrueType/OpenType font and prints
// the metrics that matter for laying out text: units per em, the three
// competing ascent/descent/line-gap triples (hhea, OS/2 typo, OS/2 win),
// and cap-height/x-height when the font bothers to report them.

use std::env;
use std::fs;
use std::io::{self, Read};
use std::process;

struct Table {
    tag: [u8; 4],
    offset: u32,
    length: u32,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        return;
    }
    if args.len() > 1 {
        print_usage();
        process::exit(2);
    }

    let data = match args.first().map(|s| s.as_str()) {
        None | Some("-") => read_stdin(),
        Some(path) => fs::read(path).map_err(|e| format!("{path}: {e}")),
    };

    let data = match data {
        Ok(d) => d,
        Err(e) => {
            eprintln!("fontmetrics: {e}");
            process::exit(1);
        }
    };

    match report(&data) {
        Ok(text) => print!("{text}"),
        Err(e) => {
            eprintln!("fontmetrics: {e}");
            process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("usage: fontmetrics [FILE]");
    eprintln!();
    eprintln!("Print font metrics from a TrueType or OpenType file.");
    eprintln!("With no FILE, or FILE is -, read from standard input.");
}

fn read_stdin() -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    io::stdin()
        .read_to_end(&mut buf)
        .map_err(|e| format!("stdin: {e}"))?;
    Ok(buf)
}

fn report(data: &[u8]) -> Result<String, String> {
    let tag = u32_at(data, 0)?;
    if tag == 0x74746366 {
        // 'ttcf'
        return Err("TrueType collections (.ttc) are not supported yet".to_string());
    }

    let tables = read_table_directory(data)?;
    let mut out = String::new();

    let units_per_em = if let Some(head) = find(&tables, b"head") {
        let upm = u16_at(data, head.offset as usize + 18)?;
        out.push_str(&format!("units per em     : {upm}\n"));
        upm
    } else {
        return Err("no 'head' table found; not a valid sfnt font".to_string());
    };

    if let Some(hhea) = find(&tables, b"hhea") {
        let off = hhea.offset as usize;
        let ascender = i16_at(data, off + 4)?;
        let descender = i16_at(data, off + 6)?;
        let line_gap = i16_at(data, off + 8)?;
        out.push_str(&format!(
            "hhea ascent/descent/gap : {ascender} / {descender} / {line_gap}\n"
        ));
    }

    if let Some(os2) = find(&tables, b"OS/2") {
        let off = os2.offset as usize;
        let version = u16_at(data, off)?;
        let typo_ascender = i16_at(data, off + 68)?;
        let typo_descender = i16_at(data, off + 70)?;
        let typo_line_gap = i16_at(data, off + 72)?;
        let win_ascent = u16_at(data, off + 74)?;
        let win_descent = u16_at(data, off + 76)?;
        out.push_str(&format!(
            "OS/2 typo asc/desc/gap  : {typo_ascender} / {typo_descender} / {typo_line_gap}\n"
        ));
        out.push_str(&format!(
            "OS/2 win ascent/descent : {win_ascent} / {win_descent}\n"
        ));

        if version >= 2 && os2.length >= 96 {
            let x_height = i16_at(data, off + 86)?;
            let cap_height = i16_at(data, off + 88)?;
            out.push_str(&format!("x-height / cap-height   : {x_height} / {cap_height}\n"));
        }
    }

    if let Some(post) = find(&tables, b"post") {
        let off = post.offset as usize;
        let italic_angle = fixed_at(data, off + 4)?;
        out.push_str(&format!("italic angle     : {italic_angle:.2} degrees\n"));
    }

    let _ = units_per_em; // reserved for the em-to-pixel conversion planned next
    Ok(out)
}

fn read_table_directory(data: &[u8]) -> Result<Vec<Table>, String> {
    let num_tables = u16_at(data, 4)? as usize;
    let mut tables = Vec::with_capacity(num_tables);
    for i in 0..num_tables {
        let record_off = 12 + i * 16;
        let mut tag = [0u8; 4];
        tag.copy_from_slice(bytes_at(data, record_off, 4)?);
        let offset = u32_at(data, record_off + 8)?;
        let length = u32_at(data, record_off + 12)?;
        tables.push(Table { tag, offset, length });
    }
    Ok(tables)
}

fn find<'a>(tables: &'a [Table], tag: &[u8; 4]) -> Option<&'a Table> {
    tables.iter().find(|t| &t.tag == tag)
}

fn bytes_at(data: &[u8], off: usize, len: usize) -> Result<&[u8], String> {
    data.get(off..off + len)
        .ok_or_else(|| "unexpected end of font data".to_string())
}

fn u16_at(data: &[u8], off: usize) -> Result<u16, String> {
    let b = bytes_at(data, off, 2)?;
    Ok(u16::from_be_bytes([b[0], b[1]]))
}

fn i16_at(data: &[u8], off: usize) -> Result<i16, String> {
    Ok(u16_at(data, off)? as i16)
}

fn u32_at(data: &[u8], off: usize) -> Result<u32, String> {
    let b = bytes_at(data, off, 4)?;
    Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
}

fn fixed_at(data: &[u8], off: usize) -> Result<f64, String> {
    let raw = u32_at(data, off)? as i32;
    Ok(raw as f64 / 65536.0)
}
