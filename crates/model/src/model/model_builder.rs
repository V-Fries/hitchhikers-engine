use std::collections::HashMap;

use crate::{obj::Obj, vertex::TextureCoordinate, ObjFile, Vertex};

use super::{VertexIndex, Model};

#[derive(Default)]
pub struct ModelBuilder {
    vertices: Vec<Vertex>,
    vertex_indices: Vec<VertexIndex>,

    vertex_map: HashMap<Vertex, VertexIndex>,
}

impl ModelBuilder {
    pub(crate) fn build(self) -> Model {
        Model {
            vertices: self.vertices.into_boxed_slice(),
            vertex_indices: self.vertex_indices.into_boxed_slice(),
        }
    }
}

impl<'a> TryFrom<ObjFile<'a>> for ModelBuilder {
    type Error = <Obj as TryFrom<ObjFile<'a>>>::Error;

    fn try_from(obj_file: ObjFile) -> Result<Self, Self::Error> {
        let obj: Obj = obj_file.try_into()?;
        let mut builder = Self::default();

        for (i, face_geometry) in obj.faces_geometry.iter().enumerate() {
            #[allow(clippy::needless_range_loop)]
            for j in 0..3 {
                let position = obj.geometry[face_geometry[j] as usize].take::<3>();

                let texture_coordinate = if i < obj.faces_textures.len() {
                    let texture = &obj.textures[obj.faces_textures[i][j] as usize];
                    TextureCoordinate::from([texture[0], 1. - texture[1]])
                } else {
                    [0.; 2].into()
                };

                let vertex = Vertex::new(position, [1., 1., 1.], texture_coordinate);

                builder.add_vertex(vertex);
            }
        }

        Ok(builder)
    }
}

impl ModelBuilder {
    fn add_vertex(&mut self, vertex: Vertex) {
        self.vertex_map
            .entry(vertex.clone())
            .and_modify(|index| self.vertex_indices.push(*index))
            .or_insert_with(|| {
                let index = self.vertices.len() as u32;
                self.vertices.push(vertex);
                self.vertex_indices.push(index);
                index
            });
    }
}
