use core::Shape;
use errors::*;
use glium::backend::Facade;
use glium::Program;
use std::fs::File;
use std::io::Read;
use std::path::Path;


const SHADER_FOLDER: &'static str = "shader";


pub fn load_program_with_shape<F: Facade, S: ShaderSource>(
    facade: &F,
    src: S,
    shape: &Shape,
) -> Result<Program> {
    let de_shader = shape.de_shader();
    let vert_buf = load_file(src.vert_path(), "vert")?
        .replace("// INCLUDE(DE)", &de_shader);
    let frag_buf = load_file(src.frag_path(), "frag")?
        .replace("// INCLUDE(DE)", &de_shader);

    link_program(facade, &vert_buf, &frag_buf)
}


pub fn load_program<F: Facade, S: ShaderSource>(
    facade: &F,
    src: S
) -> Result<Program> {
    let vert_buf = load_file(src.vert_path(), "vert")?;
    let frag_buf = load_file(src.frag_path(), "frag")?;

    link_program(facade, &vert_buf, &frag_buf)
}

fn link_program<F: Facade>(
    facade: &F,
    vert_buf: &str,
    frag_buf: &str,
) -> Result<Program> {
    debug!("Linking program ...");

    trace!("Vertex shader:\n{}", vert_buf);
    trace!("Fragment shader:\n{}", frag_buf);

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

fn load_file(file_name: &Path, ext: &str) -> Result<String> {
    let shader_folder = Path::new(SHADER_FOLDER);
    let path = shader_folder.join(file_name).with_extension(ext);
    debug!("Loading shader '{}' ...", path.display());

    let mut buf = String::new();
    File::open(path).and_then(|mut f| f.read_to_string(&mut buf))?;

    Ok(buf)
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
