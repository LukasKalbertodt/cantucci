#[macro_use] extern crate log;
#[macro_use] extern crate error_chain;
extern crate lalrpop_util;
extern crate term_painter;

mod ast;
pub mod lex;
pub mod base;
pub mod diag;
pub mod parse;
pub mod hir;
pub mod errors;

use std::path::Path;

use errors::*;
pub use hir::Shape;
use base::FileMap;

const MAX_PREVIEW_LEN: usize = 15;


pub fn open_file<P: AsRef<Path>>(file_name: P) -> Result<FileMap> {
    use std::fs::File;
    use std::io::Read;

    // Read file contents into buffer
    let mut file = File::open(&file_name)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;

    // Create filemap and parse
    let lossy_filename = file_name.as_ref().to_string_lossy().into_owned();
    Ok(base::FileMap::new(lossy_filename, buffer))
}

impl Shape {
    pub fn load(file_map: &FileMap) -> Result<Self> {
        use std::rc::Rc;
        use std::cell::RefCell;
        use lalrpop_util::ParseError;

        use base::Span;
        use lex::Lexer;
        use parse::grammar;
        use diag::Report;


        let lex_errors = Rc::new(RefCell::new(Vec::new()));
        let mut parse_errors = Vec::new();

        // Adjust lexer iterator to fit the parser and to store all encountered
        // errors.
        let lex_errors_closure = lex_errors.clone();
        let lexer = Lexer::new(&file_map)
            .map(|res| {
                match res {
                    Ok(ts) => Ok((ts.span.lo, ts.tok, ts.span.hi)),
                    Err(lex::Error { report, poison: Some(p) }) => {
                        lex_errors_closure.borrow_mut().push(report);
                        Ok((p.span.lo, p.tok, p.span.hi))
                    }
                    Err(e) => Err(e.report),
                }
            });
            // .map(|res| match res {
            //     Ok(t) => Some(t),
            //     Err(lex::Error { report, poison }) => {
            //         lex_errors.push(report);
            //         poison
            //     }
            // })
            // .take_while(|t| t.is_some())
            // .map(Option::unwrap)
            // .map(|ts| { (ts.span.lo, ts.tok, ts.span.hi) });

        // Try to parse
        let res = grammar::parse_ShapeDef(&mut parse_errors, lexer);

        if !lex_errors.borrow().is_empty() {
            let errors = Rc::try_unwrap(lex_errors).unwrap().into_inner();
            return Err(ErrorKind::LexingError(errors).into());
        }

        let res = res.map_err(|e| {
            let rep = match e {
                ParseError::User { error: e } => e,
                ParseError::InvalidToken { location: loc } => {
                    Report::simple_error("lalrpop InvalidToken", Span::single(loc))
                },
                ParseError::UnrecognizedToken { token, expected } => {
                    match token {
                        None => {
                            Report::simple_spanless_error(
                                "Unexpected end of file (EOF) while parsing. Maybe you \
                                    forgot to close a parentheses or brace?",
                            )
                        },
                        Some((lo, tok, hi)) => {
                            let tok_name = tok.as_str();
                            let span = Span::new(lo, hi);

                            // Prepare token preview
                            let mut chs = file_map.src()[span.into_range()].chars();
                            let mut tok_str: String = chs.by_ref().take(MAX_PREVIEW_LEN).collect();
                            if chs.next().is_some() {
                                // if the original span was longer than 15 chars, we append '...'
                                tok_str.push_str("...");
                            }

                            let mut msg = format!("unexpected '{}'", tok_name);
                            if tok_name != tok_str {
                                msg.push_str(&format!(" (`{}`)", tok_str));
                            }
                            if expected.len() == 1 {
                                msg.push_str(&format!(". Expected `{}`", expected[0]));
                            } else if expected.len() > 1 {
                                msg.push_str(&format!(". Expected one of {:?}", expected));
                            }

                            Report::simple_error(msg, span)
                        }
                    }
                },
                ParseError::ExtraToken  { token: (lo, tok, hi) } => {
                    Report::simple_error(
                        format!("lalrpop ExtraToken {:?}", tok),
                        Span::new(lo, hi),
                    )
                },
            };
            parse_errors.push(rep);
        });

        if !parse_errors.is_empty() {
            return Err(ErrorKind::ParsingError(parse_errors).into());
        }

        Self::from_ast(res.unwrap())
    }

    fn from_ast(root: ast::ShapeDef) -> Result<Self> {
        unimplemented!()

    }
}
