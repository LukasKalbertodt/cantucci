use std::{error::Error, fs, path::Path};

use shaderc::{Compiler, ShaderKind};



fn main() -> Result<(), Box<dyn Error>> {
    compile_shaders()?;

    Ok(())
}


const SHADERS: &[&str] = &[
    "sky.vert",
    "sky.frag",
];

fn compile_shaders() -> Result<(), Box<dyn Error>> {
    let out_dir = Path::new(&std::env::var("OUT_DIR").unwrap()).join("shaders");
    if !out_dir.exists() {
        fs::create_dir(&out_dir)?;
    }

    let mut compiler = Compiler::new().unwrap();

    for filename in SHADERS {
        let full_path = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("src")
            .join("shaders")
            .join(filename);
        let out_path = out_dir.join(format!("{}.spirv", filename));

        // If the spirv file is newer than the source file, we skip this shader.
        // Cargo makes sure the build script is only rerun if any of the shader
        // files change, but we still want to avoid recompiling all shaders if
        // one changed.
        let skip = out_path.exists()
            && full_path.metadata()?.modified()? < out_path.metadata()?.modified()?;
        if skip {
            continue;
        }

        // Figure out shader kind by file extension.
        let path = Path::new(filename);
        let kind = match path.extension().ok_or("no extension in shader filename")?.to_str() {
            Some("vert") => ShaderKind::Vertex,
            Some("frag") => ShaderKind::Fragment,
            _ => Err("invalid shader file extension")?,
        };

        // Actually compile shader and deal with errors.
        let src = fs::read_to_string(&full_path)?;
        let result = compiler.compile_into_spirv(&src, kind, filename, "main", None);
        let artifact = match result {
            Ok(v) => v,
            Err(shaderc::Error::CompilationError(_, msg)) => {
                eprintln!("{}", msg);
                Err("failed to compile shader")?
            }
            Err(e) => Err(e)?,
        };

        for warning in artifact.get_warning_messages().lines() {
            println!("cargo:warning={}", warning);
        }

        // Write out result.
        fs::write(out_path, artifact.as_binary_u8())?;
        println!("cargo:rerun-if-changed={}", full_path.display());
    }

    Ok(())
}
