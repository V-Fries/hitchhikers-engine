use linear_algebra::{Matrix, Vec3, Vec4};
use rs42::extensions::vec::TryPush;

use crate::obj::{ObjBuilder, ObjParsingErrorDetail};

use super::Face;

#[derive(Default)]
pub struct Triangles {
    pub geometries: Vec<[u32; 3]>,
    pub textures: Vec<[u32; 3]>,
    pub normals: Vec<[u32; 3]>,
}

pub fn triangulate_face(
    face: Face,
    _obj_builder: &ObjBuilder,
) -> Result<Triangles, ObjParsingErrorDetail> {
    // TODO write an algorithm that works with concave polygons

    (1..face.geometries_indices.len() - 1).try_fold(Triangles::default(), |mut triangles, i| {
        try_push_triangle(&mut triangles.geometries, &face.geometries_indices, i)?;
        if !face.textures_indices.is_empty() {
            try_push_triangle(&mut triangles.textures, &face.textures_indices, i)?;
        }
        if !face.normals_indices.is_empty() {
            try_push_triangle(&mut triangles.normals, &face.normals_indices, i)?;
        }
        Ok(triangles)
    })
}

fn try_push_triangle(
    dest_triangles_vec: &mut Vec<[u32; 3]>,
    indices: &[u32],
    i: usize,
) -> Result<(), ObjParsingErrorDetail> {
    dest_triangles_vec
        .try_push([indices[0], indices[i], indices[i + 1]])
        .map_err(ObjParsingErrorDetail::AllocationFailure)
}

// This will be used when we will support triangulation of concave polygons
#[allow(dead_code)]
fn get_rotation_matrix_to_flatten_polygon_z_axis(
    geometry: &[Vec4<f32>],
    geometries_indices: &[u32],
) -> Matrix<f32, 4, 4> {
    let a = Vec3::from_fn(|i| geometry[geometries_indices[0] as usize][i]);
    let b = Vec3::from_fn(|i| geometry[geometries_indices[1] as usize][i]);
    let c = Vec3::from_fn(|i| geometry[geometries_indices[2] as usize][i]);
    let ab = b - &a;
    let ac = c - a;
    let normal = ab ^ ac;
    let angle = normal.angle_cos(&[0., 0., 1.].into()).acos();
    let axis = normal ^ Vec3::from([0., 0., 1.]);
    Matrix::rotate(&Matrix::identity(), axis, angle)
}

#[cfg(test)]
mod test {
    use linear_algebra::assert_approximately_equal;

    use super::*;

    #[test]
    fn flatten_polygon() {
        let geometry = [
            Vec4::<f32>::from([45., 56., 34., 1.]),
            Vec4::from([5., -54., 5., 1.]),
            Vec4::from([-45., 4., -56., 1.]),
        ];
        let indices = [0, 1, 2];
        let rotation = get_rotation_matrix_to_flatten_polygon_z_axis(&geometry, &indices);
        let rotated_geometry = geometry.map(|v| &rotation * &v);
        let expect = rotated_geometry[0][2];
        for vec in rotated_geometry.into_iter().skip(1) {
            assert_approximately_equal(vec[2], expect);
        }
    }
}
