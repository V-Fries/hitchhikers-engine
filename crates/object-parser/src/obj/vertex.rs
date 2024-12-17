use std::mem::{self, MaybeUninit};

use linear_algebra::Vector;
use rs42::extensions::PipeLine;

use super::ObjParsingErrorDetail;

pub fn parse_vertex_line<'a>(
    components: &mut impl Iterator<Item = &'a str>,
    vertices: &mut Vec<Vector<f32, 4>>,
) -> Result<(), ObjParsingErrorDetail> {
    let mut vertex = [const { MaybeUninit::<f32>::uninit() }; 4];

    for elem in vertex.iter_mut().take(3) {
        let Some(str) = components.next() else {
            return Err(ObjParsingErrorDetail::NotEnoughComponentsInVertex);
        };
        *elem = str_to_maybe_uninit_f32(str)?;
    }

    if let Some(str) = components.next() {
        vertex[3] = str_to_maybe_uninit_f32(str)?;
    } else {
        vertex[3] = MaybeUninit::new(1.0f32);
    }

    if components.next().is_some() {
        return Err(ObjParsingErrorDetail::TooManyComponentsInVertex);
    }

    let vertex = unsafe { mem::transmute::<[MaybeUninit<f32>; 4], [f32; 4]>(vertex) };
    vertices
        .try_reserve(1)
        .map_err(ObjParsingErrorDetail::AllocationFailure)?;
    vertices.push(vertex.into());
    Ok(())
}

fn str_to_maybe_uninit_f32(str: &str) -> Result<MaybeUninit<f32>, ObjParsingErrorDetail> {
    str.parse::<f32>()
        .map_err(ObjParsingErrorDetail::InvalidComponentInVertex)?
        .pipe(MaybeUninit::new)
        .pipe(Ok)
}
