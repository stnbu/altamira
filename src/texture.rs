use bevy::prelude::*;
use bevy::render::render_resource::*;

pub fn texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    )
}

//

fn generate_colors(n: u8) -> Vec<[u8; 3]> {
    let intensity = 255 / (n - 1);
    let mut colors = Vec::new();
    for i in 0..n {
        let r = (i as u16 * intensity as u16) as u8;
        let g = (255 - i as u16 * intensity as u16) as u8;
        let b = ((i as f32 + 0.5) as u16 * intensity as u16) as u8;
        colors.push([r, g, b]);
    }
    colors
}
