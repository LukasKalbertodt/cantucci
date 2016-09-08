use errors::*;
use glium::Program;
use glium::backend::Facade;

pub fn load_program<F, S>(facade: &F, src: S) -> Result<Program>
    where F: Facade,
          S: ShaderSource
{
    use std::fs::File;
    use std::io::Read;

    const SHADER_FOLDER: &'static str = "shader";

    // Load vertex shader
    let vert_path = format!("{}/{}.vert", SHADER_FOLDER, src.vert_path());
    debug!("Loading vertex shader '{}' ...", vert_path);

    let mut vert_buf = String::new();
    try!(File::open(vert_path).and_then(|mut f| f.read_to_string(&mut vert_buf)));

    // Load fragment shader
    let frag_path = format!("{}/{}.frag", SHADER_FOLDER, src.frag_path());
    debug!("Loading fragment shader '{}' ...", frag_path);

    let mut frag_buf = String::new();
    try!(File::open(frag_path).and_then(|mut f| f.read_to_string(&mut frag_buf)));


    // Link program
    debug!("Linking program ...");
    Program::from_source(
        facade,
        &vert_buf,
        &frag_buf,
        None
    ).map_err(|e| {
        warn!("Linking program failed. Additional information:");
        warn!("{}", e);

        e.into()
    })
}

pub trait ShaderSource {
    fn vert_path(&self) -> &str;
    fn frag_path(&self) -> &str;
}
impl<'a> ShaderSource for &'a str {
    fn vert_path(&self) -> &str { self }
    fn frag_path(&self) -> &str { self }
}
impl<'a> ShaderSource for (&'a str, &'a str) {
    fn vert_path(&self) -> &str { self.0 }
    fn frag_path(&self) -> &str { self.1 }
}
