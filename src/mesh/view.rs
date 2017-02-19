use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium::{VertexBuffer, IndexBuffer};
use mesh::buffer::{MeshBuffer, Vertex};


pub struct MeshView {
    vbuf: VertexBuffer<Vertex>,
    ibuf: IndexBuffer<u32>,
    raw_buf: MeshBuffer,
}

impl MeshView {
    pub fn from_raw_buf<F: Facade>(buf: MeshBuffer, facade: &F) -> Self {
        let vbuf = VertexBuffer::new(facade, buf.raw_vbuf()).unwrap();

        let ibuf = IndexBuffer::new(
            facade,
            PrimitiveType::TrianglesList,
            buf.raw_ibuf(),
        ).unwrap();

        MeshView {
            vbuf: vbuf,
            ibuf: ibuf,
            raw_buf: buf,
        }
    }

    pub fn vbuf(&self) -> &VertexBuffer<Vertex> {
        &self.vbuf
    }

    pub fn ibuf(&self) -> &IndexBuffer<u32> {
        &self.ibuf
    }

    pub fn raw_buf(&self) -> &MeshBuffer {
        &self.raw_buf
    }
}
