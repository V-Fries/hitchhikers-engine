mod model_builder;

use model_builder::ModelBuilder;
use rs42::extensions::PipeLine;

use crate::{vertex::Vertex, ObjFile};

type VertexIndex = u32;

pub struct Model {
    vertices: Box<[Vertex]>,
    vertex_indices: Box<[VertexIndex]>,
}


// Constructors:

impl<'a> TryFrom<ObjFile<'a>> for Model {
    type Error = <ModelBuilder as TryFrom<ObjFile<'a>>>::Error;

    fn try_from(obj_file: ObjFile<'a>) -> Result<Self, Self::Error> {
        ModelBuilder::try_from(obj_file)?.build().pipe(Ok)
    }
}


// Getters:

impl Model {
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    pub fn vertex_indices(&self) -> &[VertexIndex] {
        &self.vertex_indices
    }
}
