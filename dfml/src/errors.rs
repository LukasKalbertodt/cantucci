use std::io;
use lex;
use diag::Report;

error_chain! {
    foreign_links {
        Io(io::Error);
    }

    errors {
        LexingError(reports: Vec<Report>) {
            description("lexing error")
        }
        ParsingError(reports: Vec<Report>) {
            description("parsing error")
        }
    }
}
