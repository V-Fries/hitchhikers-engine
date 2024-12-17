use rs42::extensions::PipeLine;

use crate::obj::{ObjBuilder, ObjParsingErrorDetail};

use super::Face;

pub fn parse_face_components<'a>(
    components: &mut impl Iterator<Item = &'a str>,
    obj_builder: &ObjBuilder,
) -> Result<Face, ObjParsingErrorDetail> {
    components
        .try_fold(Face::default(), |mut face, component| {
            parse_face_sub_components(&mut face, component.split('/'), obj_builder)?;
            Ok(face)
        })?
        .pipe(check_face_validity)
}

fn parse_face_sub_components<'a>(
    face: &mut Face,
    mut sub_components: impl Iterator<Item = &'a str>,
    obj_builder: &ObjBuilder,
) -> Result<(), ObjParsingErrorDetail> {
    push_face_sub_component_to_index_vec(
        sub_components.next(),
        &obj_builder.geometry,
        ObjParsingErrorDetail::FaceGeometryDoesNotExist,
        &mut face.geometries_indices,
    )?;

    if let Err(err) = push_face_sub_component_to_index_vec(
        sub_components.next(),
        &obj_builder.faces_textures,
        ObjParsingErrorDetail::FaceTextureDoesNotExist,
        &mut face.textures_indices,
    ) {
        if let ObjParsingErrorDetail::NotEnoughSubComponentsInFace = err {
            return Ok(());
        }
        return Err(err);
    };

    push_face_sub_component_to_index_vec(
        sub_components.next(),
        &obj_builder.faces_normals,
        ObjParsingErrorDetail::FaceNormalDoesNotExist,
        &mut face.normals_indices,
    )?;

    if sub_components.next().is_some() {
        return Err(ObjParsingErrorDetail::TooManySubComponentsInFace);
    }
    Ok(())
}

fn push_face_sub_component_to_index_vec<T>(
    sub_component: Option<&str>,
    associated_vec: &[T],
    invalid_index_err: ObjParsingErrorDetail,
    dest_index_vec: &mut Vec<u32>,
) -> Result<(), ObjParsingErrorDetail> {
    let index = sub_component
        .ok_or(ObjParsingErrorDetail::NotEnoughSubComponentsInFace)?
        .parse::<u32>()
        .map_err(ObjParsingErrorDetail::InvalidSubComponentInFace)?
        .checked_sub(1)
        .ok_or(ObjParsingErrorDetail::FaceSubComponentCanNotBe0)?;
    if index as usize >= associated_vec.len() {
        return Err(invalid_index_err);
    }
    dest_index_vec
        .try_reserve(1)
        .map_err(ObjParsingErrorDetail::AllocationFailure)?;
    dest_index_vec.push(index);
    Ok(())
}

fn check_face_validity(face: Face) -> Result<Face, ObjParsingErrorDetail> {
    if face.geometries_indices.len() < 3 {
        return Err(ObjParsingErrorDetail::FaceShouldHaveAtLeast3Components);
    }
    if !face.textures_indices.is_empty()
        && face.textures_indices.len() != face.geometries_indices.len()
    {
        return Err(ObjParsingErrorDetail::ShouldHaveEitherNoTextureOrNTextures);
    }
    if !face.normals_indices.is_empty()
        && face.normals_indices.len() != face.geometries_indices.len()
    {
        return Err(ObjParsingErrorDetail::ShouldHaveEitherNoNormalsOrNNormals);
    }
    Ok(face)
}
