use errors::*;
use glium::Program;
use glium::backend::Facade;
use std::path::Path;

pub fn load_program<F, S>(facade: &F, src: S) -> Result<Program>
    where F: Facade,
          S: ShaderSource
{
    use std::fs::File;
    use std::io::Read;

    const SHADER_FOLDER: &'static str = "shader";

    let shader_folder = Path::new(SHADER_FOLDER);

    // Load vertex shader
    let vert_path = shader_folder.join(src.vert_path()).with_extension("vert");
    debug!("Loading vertex shader '{}' ...", vert_path.display());

    let mut vert_buf = String::new();
    File::open(vert_path).and_then(|mut f| f.read_to_string(&mut vert_buf))?;

    // Load fragment shader
    let frag_path = shader_folder.join(src.frag_path()).with_extension("frag");
    debug!("Loading fragment shader '{}' ...", frag_path.display());

    let mut frag_buf = String::new();
    File::open(frag_path).and_then(|mut f| f.read_to_string(&mut frag_buf))?;

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
    fn vert_path(&self) -> &Path;
    fn frag_path(&self) -> &Path;
}

impl<'a> ShaderSource for &'a str {
    fn vert_path(&self) -> &Path { Path::new(self) }
    fn frag_path(&self) -> &Path { Path::new(self) }
}
impl<V: AsRef<Path>, F: AsRef<Path>> ShaderSource for (V, F) {
    fn vert_path(&self) -> &Path { self.0.as_ref() }
    fn frag_path(&self) -> &Path { self.1.as_ref() }
}
