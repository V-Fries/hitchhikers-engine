use std::collections::HashMap;

use crate::{
    obj::Obj,
    vertex::{Vec2, Vec3, Vertex},
    ObjFile,
};

#[derive(Default)]
pub struct Model {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl<'a> TryFrom<ObjFile<'a>> for Model {
    type Error = <Obj as TryFrom<ObjFile<'a>>>::Error;

    fn try_from(obj_file: ObjFile) -> Result<Self, Self::Error> {
        let obj: Obj = obj_file.try_into()?;
        let mut model = Model::default();

        let mut vertex_map = HashMap::new();

        for (i, face_geometry) in obj.faces_geometry.iter().enumerate() {
            #[allow(clippy::needless_range_loop)]
            for j in 0..3 {
                let position = obj.geometry[face_geometry[j] as usize].take::<3>();

                let texture_coordinate = if i < obj.faces_textures.len() {
                    let texture = &obj.textures[obj.faces_textures[i][j] as usize];
                    Vec2::from([texture[0], 1. - texture[1]])
                } else {
                    [0.; 2].into()
                };

                add_vertex(position, texture_coordinate, &mut vertex_map, &mut model);
            }
        }

        Ok(model)
    }
}

fn add_vertex(
    position: Vec3,
    texture_coordinate: Vec2,
    vertex_map: &mut HashMap<Vertex, u32>,
    model: &mut Model,
) {
    let vertex = Vertex::new(position, [1., 1., 1.], texture_coordinate);

    vertex_map
        .entry(vertex.clone())
        .and_modify(|index| model.indices.push(*index))
        .or_insert_with(|| {
            let index = model.vertices.len() as u32;
            model.vertices.push(vertex);
            model.indices.push(index);
            index
        });
}
