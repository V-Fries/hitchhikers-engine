use std::mem::{self, MaybeUninit};

use linear_algebra::Vector;
use rs42::extensions::{vec::TryPush, PipeLine};

use super::ObjParsingErrorDetail;

pub fn parse_texture_line<'a>(
    components: &mut impl Iterator<Item = &'a str>,
    textures: &mut Vec<Vector<f32, 3>>,
) -> Result<(), ObjParsingErrorDetail> {
    let mut texture = [const { MaybeUninit::<f32>::uninit() }; 3];

    texture[0] = components.next().map_or_else(
        || Err(ObjParsingErrorDetail::NotEnoughComponentsInTexture),
        parse_texture_component,
    )?;

    for elem in texture.iter_mut().skip(1) {
        *elem = components
            .next()
            .map_or_else(|| Ok(MaybeUninit::new(0.)), parse_texture_component)?;
    }

    if components.next().is_some() {
        return Err(ObjParsingErrorDetail::TooManyComponentsInTexture);
    }

    let texture = unsafe { mem::transmute::<[MaybeUninit<f32>; 3], [f32; 3]>(texture) };
    textures
        .try_push(texture.into())
        .map_err(ObjParsingErrorDetail::AllocationFailure)?;
    Ok(())
}

fn parse_texture_component(str: &str) -> Result<MaybeUninit<f32>, ObjParsingErrorDetail> {
    let component = str
        .parse::<f32>()
        .map_err(ObjParsingErrorDetail::InvalidComponentInTexture)?;

    // TODO check behaviour when component is not in range
    if !(0. ..=1.).contains(&component) {
        return Err(ObjParsingErrorDetail::ComponentInNormalIsNotInRange0To1);
    }

    component.pipe(MaybeUninit::new).pipe(Ok)
}
