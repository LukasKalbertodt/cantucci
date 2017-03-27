extern crate dfml;

use dfml::lex::Lexer;
use dfml::base::FileMap;


fn main() {
    let f = FileMap::new("<anon>", "shape Sphere {
        param RADIUS: f32;
    }");
    let lexer = Lexer::new(&f);
    for t in lexer {
        println!("{:?}", t);
    }
}
