extern crate dfml;

use dfml::errors::ErrorKind;
use dfml::diag::{print, PrintOptions};

fn main() {
    let file_map = match dfml::open_file("simple.dfml") {
        Ok(v) => v,
        Err(e) => {
            // TODO: better output
            println!("Error opening file: {}", e);
            return;
        }
    };

    let res = dfml::Shape::load(&file_map);
    match res {
        Err(e) => {
            match *e.kind() {
                ErrorKind::LexingError(ref reps) | ErrorKind::ParsingError(ref reps) => {
                    for rep in reps {
                        print(rep, &file_map, PrintOptions::default());
                    }
                }
                _ => {
                    // TODO: better output
                    println!("an error occured: {}", e);
                }
            }
        }
        Ok(v) => println!("{:?}", v),
    }

    // let src2 = "
    //     shape Sphere {
    //         param RADIUS: Float;
    //     }
    // ";


    // let f = FileMap::new("<anon>", src2);

    // let mut lex_errors = Vec::new();

    // let lexer = Lexer::new(&f)
    //     .map(|res| match res {
    //         Ok(t) => Some(t),
    //         Err(lex::Error { report, poison, .. }) => {
    //             lex_errors.push(report);
    //             poison
    //         }
    //     })
    //     .take_while(|t| t.is_some())
    //     .map(Option::unwrap)
    //     .map(|ts| { (ts.span.lo, ts.tok, ts.span.hi) });
    // // for t in lexer {
    // //     println!("{:?}", t);
    // // }
    // let res = grammar::parse_ShapeDef(lexer);
    // println!("{:#?}", res);
}
