extern crate dfml;

use dfml::lex::{self, Lexer};
use dfml::base::FileMap;
use dfml::parse::grammar;


fn main() {
    let src1 = "shape Sphere";
    let src2 = "
        shape Sphere {
            param RADIUS: f32;
        }
    ";


    let f = FileMap::new("<anon>", src2);

    let mut lex_errors = Vec::new();

    let lexer = Lexer::new(&f)
        .map(|res| match res {
            Ok(t) => Some(t),
            Err(lex::Error { report, poison, .. }) => {
                lex_errors.push(report);
                poison
            }
        })
        .take_while(|t| t.is_some())
        .map(Option::unwrap)
        .map(|ts| { (ts.span.lo, ts.tok, ts.span.hi) });
    // for t in lexer {
    //     println!("{:?}", t);
    // }
    let res = grammar::parse_ShapeDef(lexer);
    println!("{:#?}", res);
}
