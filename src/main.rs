use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};
use std::f32::consts::TAU;

// mod texture;
// use texture::*;

const POLOIDAL: usize = 8;
const TOROIDAL: usize = 8;

const EPSILON: f32 = 0.00001;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_startup_system(setup)
        .add_startup_system(spawn_camera)
        .add_system(control)
        .run();
}

#[derive(Component)]
struct CameraGimbal;

#[derive(Component)]
struct CameraBoom;

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn((TransformBundle::default(), CameraBoom))
        .with_children(|children| {
            children.spawn((
                Camera3dBundle {
                    transform: Transform::from_translation(Vec3::X * 2.0 + Vec3::Y * 0.2)
                        .looking_at(Vec3::ZERO, Vec3::Y),
                    ..Default::default()
                },
                CameraGimbal,
            ));
        });
}

fn get_colors(n: u64) -> Vec<[u8; 4]> {
    let mut colors = vec![];
    for i in 0..n {
        let x = (n - i) as f32 / n as f32;

        let red = 0.5 * (x - 0.0 * TAU / 3.0).sin() + 0.5;
        let green = 0.5 * (x - 1.0 * TAU / 3.0).sin() + 0.5;
        let blue = 0.5 * (x - 2.0 * TAU / 3.0).sin() + 0.5;

        let red = (red * 255.0) as u8;
        let green = (green * 255.0) as u8;
        let blue = (blue * 255.0) as u8;
        let alpha = 255;
        colors.push([red, green, blue, alpha]);
    }
    colors
}

fn get_flat_index(x: usize, y: usize) -> usize {
    y * POLOIDAL + x * TOROIDAL * 4
}

fn control(
    mut boom_query: Query<&mut Transform, With<CameraBoom>>,
    mut gimbal_query: Query<&mut Transform, (With<CameraGimbal>, Without<CameraBoom>)>,
    mut scroll_evr: EventReader<MouseWheel>,
    keys: Res<Input<KeyCode>>,
    mut motion_evr: EventReader<MouseMotion>,
) {
    let mut boom = boom_query.get_single_mut().unwrap();
    let mut gimbal = gimbal_query.get_single_mut().unwrap();
    // FIXME: Handle "line" vs "pixel" at some point.
    let boom_scale_delta = scroll_evr.iter().fold(0.0_f32, |b, delta| b + delta.y);
    let new_scale = boom.scale - Vec3::ONE * boom_scale_delta / 100.0;
    if new_scale.x > EPSILON && new_scale.y > EPSILON && new_scale.z > EPSILON {
        boom.scale = new_scale;
    }
    let mouse_delta = motion_evr
        .iter()
        .fold(Vec2::ZERO, |b, ev| b + Vec2::new(ev.delta.x, ev.delta.y))
        * 0.01;
    if mouse_delta.length() > EPSILON {
        if keys.pressed(KeyCode::B) {
            let local_x = boom.local_x();
            let local_y = boom.local_y();
            boom.rotate(Quat::from_axis_angle(local_x, -mouse_delta.y));
            boom.rotate(Quat::from_axis_angle(local_y, -mouse_delta.x));
        }
        if keys.pressed(KeyCode::G) {
            let local_x = gimbal.local_x();
            let local_y = gimbal.local_y();
            gimbal.rotate(Quat::from_axis_angle(local_x, -mouse_delta.y));
            gimbal.rotate(Quat::from_axis_angle(local_y, -mouse_delta.x));
        }
    }
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn(DirectionalLightBundle {
        transform: Transform::default().looking_at(Vec3::new(1.0, 1.0, 1.0), Vec3::Y),
        directional_light: DirectionalLight {
            shadows_enabled: false,
            ..Default::default()
        },
        ..Default::default()
    });

    commands.spawn(DirectionalLightBundle {
        transform: Transform::default().looking_at(Vec3::new(-1.0, -1.0, -1.0), Vec3::Y),
        directional_light: DirectionalLight {
            shadows_enabled: false,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut pixels: Vec<u8> = (0..(TOROIDAL * POLOIDAL * 4))
        .map(|i| if i % 4 == 3 { 255 } else { 0 })
        .collect();

    pixels[get_flat_index(0, 0)..4].copy_from_slice(&[255, 0, 0, 255]);

    let material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(texture(pixels))),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(
            shape::UVSphere {
                radius: 0.015,
                ..Default::default()
            }
            .into(),
        ),
        material: materials.add(Color::WHITE.into()),
        ..Default::default()
    });

    for (color, axis) in [
        (Color::GREEN, Vec3::Y),
        (Color::BLUE, Vec3::Z),
        (Color::RED, Vec3::X),
    ] {
        for i in 1..4 {
            commands.spawn(PbrBundle {
                mesh: meshes.add(
                    shape::UVSphere {
                        radius: 0.01,
                        ..Default::default()
                    }
                    .into(),
                ),
                material: materials.add(color.into()),
                transform: Transform::from_translation(axis * i as f32),
                ..Default::default()
            });
        }
    }

    commands.spawn(PbrBundle {
        mesh: meshes.add(
            shape::Torus {
                radius: 0.5,
                ring_radius: 0.25,
                ..Default::default()
            }
            .into(),
        ),
        material,
        ..Default::default()
    });
}

//

use bevy::render::render_resource::*;

pub fn texture(pixels: Vec<u8>) -> Image {
    dbg!(POLOIDAL * TOROIDAL * 4);
    dbg!(pixels.len());
    //assert!(POLOIDAL * TOROIDAL * 4 == pixels.len() as u32);
    Image::new_fill(
        Extent3d {
            width: TOROIDAL as u32,
            height: POLOIDAL as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels.as_slice(),
        TextureFormat::Rgba8UnormSrgb,
    )
}
