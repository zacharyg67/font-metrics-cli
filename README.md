# fontmetrics

A small command line tool that reads a TrueType or OpenType font and prints
the metrics you need to lay out text correctly: units per em, the ascent/
descent/line-gap numbers from the `hhea` and `OS/2` tables (which routinely
disagree with each other), and cap-height/x-height when the font reports
them.

## Why

Every font carries at least two, sometimes three, different opinions about
its own line height: the `hhea` table's ascender/descender, the `OS/2`
table's "typo" values, and the `OS/2` table's "win" values that some
Windows apps use for line height instead. Browsers and design tools don't
all pick the same one, which is why the same font can look tightly packed
in one app and loose in another. Rather than open a font editor or write a
one-off parser every time this comes up, this tool just dumps the numbers.

## Usage

From a file:

```
$ fontmetrics /System/Library/Fonts/Supplemental/Georgia.ttf
units per em     : 2048
hhea ascent/descent/gap : 1878 / -449 / 0
OS/2 typo asc/desc/gap  : 1419 / -397 / 200
OS/2 win ascent/descent : 1878 / 449
x-height / cap-height   : 1000 / 1466
italic angle     : 0.00 degrees
```

From standard input, which is the point of it existing:

```
$ curl -s https://example.com/fonts/Inter.ttf | fontmetrics
```

or piped from anything else that can produce a font on stdout:

```
$ cat MyFont.otf | fontmetrics -
```

With no argument, or `-` as the argument, input is read from stdin. With a
path argument, that file is read instead.

Add `--json` to get the same metrics as a single JSON object instead, for
piping into `jq` or another script:

```
$ fontmetrics --json Georgia.ttf
{"units_per_em":2048,"hhea":{"ascent":1878,"descent":-449,"line_gap":0},"os2_typo":{"ascent":1419,"descent":-397,"line_gap":200},"os2_win":{"ascent":1878,"descent":449},"cap_height":1466,"x_height":1000,"italic_angle":0.00}
```

Fields the font doesn't provide come back as `null` rather than being
omitted, so the shape is the same for every font.

For a TrueType collection (`.ttc`), the tool reports on one font from the
collection at a time. It picks font 0 by default; use `--font-index` to
pick another:

```
$ fontmetrics --font-index 1 NotoSansCJK.ttc
```

## Building

Standard library only, no dependencies:

```
cargo build --release
```

## Scope

This reads the table directory and the `head`, `hhea`, `OS/2`, and `post`
tables directly; it does not parse glyph outlines, so it works the same on
TrueType-outline and CFF-outline (OpenType/CFF) fonts.
