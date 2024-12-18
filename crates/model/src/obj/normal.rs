use std::mem::{self, MaybeUninit};

use linear_algebra::Vector;
use rs42::extensions::PipeLine;

use super::ObjParsingErrorDetail;

pub fn parse_normal_line<'a>(
    components: &mut impl Iterator<Item = &'a str>,
    normals: &mut Vec<Vector<f32, 3>>,
) -> Result<(), ObjParsingErrorDetail> {
    let mut normal = [const { MaybeUninit::<f32>::uninit() }; 3];

    for elem in normal.iter_mut() {
        *elem = components.next().map_or_else(
            || Err(ObjParsingErrorDetail::NotEnoughComponentsInNormal),
            parse_normal_component,
        )?;
    }

    if components.next().is_some() {
        return Err(ObjParsingErrorDetail::TooManyComponentsInNormal);
    }

    let normal = unsafe { mem::transmute::<[MaybeUninit<f32>; 3], [f32; 3]>(normal) };
    let normal = Vector::<_, 3>::from(normal).normalize();
    normals.push(normal);
    Ok(())
}

fn parse_normal_component(str: &str) -> Result<MaybeUninit<f32>, ObjParsingErrorDetail> {
    str.parse::<f32>()
        .map_err(ObjParsingErrorDetail::InvalidComponentInNormal)?
        .pipe(MaybeUninit::new)
        .pipe(Ok)
}
