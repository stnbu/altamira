use bevy::prelude::*;
use bevy::render::render_resource::*;

pub fn texture(pixels: &[&[[u8; 4]]]) -> Image {
    // TODO: assert stuff!
    let poloidal = pixels.len();
    let toroidal = pixels.get(0).unwrap().len();
    let texture_data = pixels.concat().concat();
    assert!(poloidal <= toroidal);
    assert!(poloidal * toroidal * 4 == texture_data.len());
    Image::new_fill(
        Extent3d {
            width: toroidal as u32,
            height: poloidal as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    )
}
