mod parse_face_components;
mod triangulate_face;

use super::{ObjBuilder, ObjParsingErrorDetail};
use parse_face_components::parse_face_components;
use triangulate_face::triangulate_face;

#[derive(Default)]
struct Face {
    geometries_indices: Vec<u32>,
    textures_indices: Vec<u32>,
    normals_indices: Vec<u32>,
}

pub fn parse_face_line<'a>(
    components: &mut impl Iterator<Item = &'a str>,
    obj_builder: &mut ObjBuilder,
) -> Result<(), ObjParsingErrorDetail> {
    let face = parse_face_components(components, obj_builder)?;

    let triangles = triangulate_face(face, obj_builder);

    obj_builder.faces_geometry.extend(triangles.geometries);

    // TODO check if it is possible for an object to have textures/normals on some faces but not
    // others
    // If it is this won't work as textures and normals indices will not be associated with the
    // right geometry
    obj_builder.faces_textures.extend(triangles.textures);
    obj_builder.faces_normals.extend(triangles.normals);

    Ok(())
}
