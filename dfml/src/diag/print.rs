// TODO: get num cols of terminal dynamically
// TODO: care about the given print options

use super::{Report, ReportKind, RemarkKind, Snippet};
use base::{FileMap, LineIdx, Span};
use term_painter::ToStyle;
use term_painter::Color::*;
use std::default::Default;

/// Options for printing on the terminal. By `default()` everything is enabled.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PrintOptions {
    /// Use of unicode allowed?
    pub unicode: bool,
    /// Use of colors allowed?
    pub color: bool,
    /// Is line wrapping allowed?
    pub line_wrap: bool,
}

impl Default for PrintOptions {
    fn default() -> Self {
        PrintOptions {
            unicode: true,
            color: true,
            line_wrap: true,
        }
    }
}

/// Pretty prints a report
///
/// **Note**: right now, the `PrintOptions` are ignored.
pub fn print(rep: &Report, src: &FileMap, _: PrintOptions) {
    trace!("Printing report: {:#?}", rep);
    trace!("Printing with filemap: {:#?}", src);

    // print header
    let title = match rep.kind {
        ReportKind::Error => White.bold().bg(Red).paint("ERROR"),
        ReportKind::Warning => White.bold().bg(Yellow).paint("WARNING"),
    };

    let (sep, line) = if let Some(span) = rep.span {
        (" : ", if span.is_dummy() {
            "<dummy-span>".into()
        } else {
            let start = src.get_loc(span.lo);
            let end = src.get_loc(span.hi);
            trace!("Span is from {:?} to {:?}", start, end);

            if start.line != end.line {
                format!("{}-{}", start.line, end.line)
            } else {
                start.line.to_string()
            }
        })
    } else {
        ("", "".into())
    };

    println!("+---- {} in {}{}{} ----+",
        title,
        src.filename(),
        sep,
        Magenta.bold().paint(line)
    );



    for rem in &rep.remarks {
        trace!("Handling Remark {:?}", rem);

        // print message
        let (title, title_len) = match rem.kind {
            RemarkKind::Error => (Red.paint("error:"), 6),
            RemarkKind::Warning => (Yellow.paint("warning:"), 8),
            RemarkKind::Note => (Green.paint("note:"), 5),
        };

        print!("      =====>  {} ", title);
        // spaces + big arrow + spaces + title + space
        let indent = 6 + 6 + 2 + title_len + 1;
        let block_width = 80 - indent;

        let mut col = 0;
        for word in rem.desc.split_whitespace() {
            let word_len = word.chars().count();
            if col + word_len >= block_width && col != 0 {
                println!("");
                print!("           >  {0:>1$} ", " ", title_len);
                col = 0;
            }
            print!("{} ", White.bold().paint(word));
            col += word_len + 1;
        }
        println!("");

        // print code snippet
        if let Some(span) = rem.snippet.span() {
            print_snippet(src, span, &rem.snippet);
            println!("");
        }
    }
    println!("");
}

fn print_snippet(src: &FileMap, span: Span, snippet: &Snippet) {
    let start = src.get_loc(span.lo);
    let end = src.get_loc(span.hi);
    trace!("Span is from {:?} to {:?}", start, end);


    // ----- Dummyspan -----
    if span.is_dummy() {
        println!("   {} {} ! no snippet due to <dummy-span>, this is a bug !",
            Magenta.bold().paint("?"),
            Magenta.bold().paint("|"),
        );
    }

    // ----- Singleline -----
    else if start.line == end.line {
        let line_orig = expect_line(src, start.line);
        trace!("Printing single line span. Orig line: {:?}", line_orig);

        // replace tabs
        let line = line_orig.replace("\t", "    ");
        let num_tabs = line_orig[..start.col.0 as usize]
            .chars()
            .filter(|&c| c == '\t')
            .count();

        // adjust cols in case of replaced tabs
        let startcol = start.col.0 as usize + 3*num_tabs;
        let endcol = end.col.0 as usize + 3*num_tabs;

        let (middle, underline_len, color) = match *snippet {
            Snippet::Replace { ref with, .. } => {
                (&with[..], with.len(), Green)
            },
            Snippet::Orig(_) =>  {
                (&line[startcol..endcol], endcol - startcol, Yellow)
            },
            _ => unreachable!(),
        };

        // print the line
        println!("{:>#4} {} {}{}{}",
            Magenta.bold().paint(start.line),
            Magenta.bold().paint("|"),
            &line[..startcol],
            color.paint(middle),
            &line[endcol..],
        );

        // print the underline
        color.with(|| {
            println!("      {: <2$}{:^<3$}",
                " ", "^",
                startcol + 1,
                underline_len,
            );
        });
    }

    // ----- Multiline -----
    else {
        let (lines, color) = match *snippet {
            Snippet::Replace { ref with, .. } => {
                let mut lines = Vec::new();

                if let Some(first_break) =  with.find("\n") {
                    // we can unwrap, because we found it from the beginning
                    let last_break = with.rfind("\n").unwrap();

                    // first line
                    let line = expect_line(src, start.line);
                    let startcol = start.col.0 as usize;
                    lines.push((&line[..startcol], &with[..first_break], ""));

                    // lines in the middle
                    for line in with[..last_break].lines().skip(1) {
                        lines.push(("", line, ""));
                    }

                    // last line
                    let line = expect_line(src, end.line);
                    let endcol = end.col.0 as usize;
                    lines.push(("", &with[last_break + 1..], &line[endcol..]));

                    (lines, Green)
                } else {
                    let first_line = expect_line(src, start.line);
                    let startcol = start.col.0 as usize;
                    let last_line = expect_line(src, end.line);
                    let endcol = end.col.0 as usize;

                    (vec![(
                        &first_line[..startcol],
                        &with[..],
                        &last_line[endcol..]
                    )], Green)
                }
            },
            Snippet::Orig(_) =>  {
                let mut lines = Vec::new();

                // first line
                let line = expect_line(src, start.line);
                let startcol = start.col.0 as usize;
                lines.push((&line[..startcol], &line[startcol..], ""));

                // lines in the middle
                for line_idx in (start.line.0 + 1)..end.line.0 {
                    let line = expect_line(src, LineIdx(line_idx));
                    lines.push(("", line, ""));
                }

                // last line
                let line = expect_line(src, end.line);
                let endcol = end.col.0 as usize;
                lines.push(("", &line[..endcol], &line[endcol..]));

                (lines, Yellow)
            },
            _ => unreachable!(),
        };


        for (i, &(pre, middle, post)) in lines.iter().enumerate() {
            println!("{:>#4} {} {}{}{}",
                Magenta.bold().paint(start.line + LineIdx(i as u32)),
                Magenta.bold().paint("|"),
                pre,
                color.paint(middle),
                post,
            );
        }
    }
}

fn expect_line(src: &FileMap, line: LineIdx) -> &str {
    src.get_line(line).expect("`Loc` from FileMap should return a valid line")
}
