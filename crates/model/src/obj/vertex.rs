use std::mem::{self, MaybeUninit};

use linear_algebra::Vector;
use rs42::extensions::{vec::TryPush, PipeLine};

use super::ObjParsingErrorDetail;

pub fn parse_vertex_line<'a>(
    components: &mut impl Iterator<Item = &'a str>,
    vertices: &mut Vec<Vector<f32, 4>>,
) -> Result<(), ObjParsingErrorDetail> {
    let mut vertex = [const { MaybeUninit::<f32>::uninit() }; 4];

    for elem in vertex.iter_mut().take(3) {
        *elem = components.next().map_or_else(
            || Err(ObjParsingErrorDetail::NotEnoughComponentsInVertex),
            parse_vertex_component,
        )?;
    }

    vertex[3] = components
        .next()
        .map_or_else(|| Ok(MaybeUninit::new(1.)), parse_vertex_component)?;

    if components.next().is_some() {
        return Err(ObjParsingErrorDetail::TooManyComponentsInVertex);
    }

    let vertex = unsafe { mem::transmute::<[MaybeUninit<f32>; 4], [f32; 4]>(vertex) };
    vertices
        .try_push(vertex.into())
        .map_err(ObjParsingErrorDetail::AllocationFailure)?;
    Ok(())
}

fn parse_vertex_component(str: &str) -> Result<MaybeUninit<f32>, ObjParsingErrorDetail> {
    str.parse::<f32>()
        .map_err(ObjParsingErrorDetail::InvalidComponentInVertex)?
        .pipe(MaybeUninit::new)
        .pipe(Ok)
}
