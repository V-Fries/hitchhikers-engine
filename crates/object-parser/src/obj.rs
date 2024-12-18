mod face;
mod handle_unrecognized_line;
mod normal;
mod texture;
mod vertex;

use face::parse_face_line;
use handle_unrecognized_line::handle_unrecognized_line;
use normal::parse_normal_line;
use texture::parse_texture_line;
use vertex::parse_vertex_line;

use std::{
    collections::TryReserveError,
    error::Error,
    fmt::{Debug, Display},
    fs::File,
    io::{self, BufRead, BufReader, Read},
};

use linear_algebra::Vector;

pub struct ObjFile<'a>(pub &'a str);

#[allow(dead_code)]
#[derive(Debug)]
pub struct Obj {
    pub geometry: Box<[Vector<f32, 4>]>,
    //vp: Box<[Vector<f32, 3>]>,
    pub normals: Box<[Vector<f32, 3>]>,
    pub textures: Box<[Vector<f32, 3>]>,
    pub faces_geometry: Box<[[u32; 3]]>,
    pub faces_textures: Box<[[u32; 3]]>,
    pub faces_normals: Box<[[u32; 3]]>,
}

#[derive(Default, Debug)]
struct ObjBuilder {
    geometry: Vec<Vector<f32, 4>>,
    //vp: Vec<Vector<f32, 3>>,
    normals: Vec<Vector<f32, 3>>,
    textures: Vec<Vector<f32, 3>>,
    faces_geometry: Vec<[u32; 3]>,
    faces_textures: Vec<[u32; 3]>,
    faces_normals: Vec<[u32; 3]>,
}

#[allow(dead_code)]
pub struct ObjParsingError {
    line: Option<(usize, String)>,
    detail: ObjParsingErrorDetail,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum ObjParsingErrorDetail {
    FailedToOpenFile(io::Error),
    FailedToReadFile(io::Error),

    AllocationFailure(TryReserveError),

    NotEnoughComponentsInVertex,
    TooManyComponentsInVertex,
    InvalidComponentInVertex(<f32 as std::str::FromStr>::Err),

    NotEnoughComponentsInNormal,
    TooManyComponentsInNormal,
    InvalidComponentInNormal(<f32 as std::str::FromStr>::Err),

    NotEnoughComponentsInTexture,
    TooManyComponentsInTexture,
    InvalidComponentInTexture(<f32 as std::str::FromStr>::Err),
    ComponentInNormalIsNotInRange0To1,

    FaceGeometryDoesNotExist,
    FaceTextureDoesNotExist,
    FaceNormalDoesNotExist,
    FaceShouldHaveAtLeast3Components,
    ShouldHaveEitherNoTextureOrNTextures,
    ShouldHaveEitherNoNormalsOrNNormals,
    NotEnoughSubComponentsInFace,
    TooManySubComponentsInFace,
    InvalidSubComponentInFace(<i32 as std::str::FromStr>::Err),
    FaceSubComponentCanNotBe0,
}

impl Debug for ObjParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(line) = self.line.as_ref() {
            return write!(
                f,
                "ObjParsingError {{\n\tline: {}\n\tline_content: \"{}\"\n\tdetails: {:?}\n}}",
                line.0, line.1, self.detail,
            );
        }
        write!(f, "ObjParsingError({:?})", self.detail)
    }
}

impl Display for ObjParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for ObjParsingError {}

impl ObjBuilder {
    fn build(self) -> Obj {
        Obj {
            geometry: self.geometry.into_boxed_slice(),
            //vp: self.vp.into_boxed_slice(),
            normals: self.normals.into_boxed_slice(),
            textures: self.textures.into_boxed_slice(),
            faces_geometry: self.faces_geometry.into_boxed_slice(),
            faces_textures: self.faces_textures.into_boxed_slice(),
            faces_normals: self.faces_normals.into_boxed_slice(),
        }
    }
}

impl TryFrom<ObjFile<'_>> for Obj {
    type Error = ObjParsingError;

    fn try_from(file_name: ObjFile) -> Result<Self, Self::Error> {
        let file = File::open(file_name.0).map_err(|err| ObjParsingError {
            line: None,
            detail: ObjParsingErrorDetail::FailedToOpenFile(err),
        })?;
        BufReader::new(file).try_into()
    }
}

impl<R> TryFrom<BufReader<R>> for Obj
where
    R: Read,
{
    type Error = ObjParsingError;

    fn try_from(buf_reader: BufReader<R>) -> Result<Self, ObjParsingError> {
        let mut obj_builder = ObjBuilder::default();

        for (line_count, line) in buf_reader.lines().enumerate() {
            let line = line.map_err(|err| ObjParsingError {
                line: None,
                detail: ObjParsingErrorDetail::FailedToReadFile(err),
            })?;

            parse_line(line_count, &line, &mut obj_builder).map_err(|err| ObjParsingError {
                line: Some((line_count, line)),
                detail: err,
            })?;
        }

        Ok(obj_builder.build())
    }
}

fn parse_line(
    line_count: usize,
    line: &str,
    obj_builder: &mut ObjBuilder,
) -> Result<(), ObjParsingErrorDetail> {
    let mut split = line.split(' ').filter(|str| !str.is_empty());
    let Some(first_word) = split.next() else {
        return Ok(());
    };

    match first_word {
        "v" => parse_vertex_line(&mut split, &mut obj_builder.geometry),
        "vt" => parse_texture_line(&mut split, &mut obj_builder.textures),
        "vn" => parse_normal_line(&mut split, &mut obj_builder.normals),
        "f" => parse_face_line(&mut split, obj_builder),
        _ => {
            handle_unrecognized_line(first_word, line_count, line);
            Ok(())
        }
    }
}

// TODO write tests
