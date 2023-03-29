use bevy::input::{
    mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseWheel},
    ButtonState,
};
use bevy::{
    pbr::wireframe::{Wireframe, WireframeConfig, WireframePlugin},
    prelude::*,
    render::{mesh::Indices, render_resource::WgpuFeatures, settings::WgpuSettings, RenderPlugin},
};

use wgpu::PrimitiveTopology;

const HUMAN_FOOT: f32 = 100.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            wgpu_settings: WgpuSettings {
                features: WgpuFeatures::POLYGON_MODE_LINE,
                ..default()
            },
        }))
        .add_plugin(WireframePlugin)
        .add_startup_system(setup)
        .add_startup_system(spawn_camera)
        .run();
}

fn setup(
    mut commands: Commands,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    wireframe_config.global = false;
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: false,
            ..default()
        },
        ..default()
    });

    let torus = Torus {
        radius: 0.5 * HUMAN_FOOT,
        ring_radius: 0.25 * HUMAN_FOOT,
        ..Default::default()
    };
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(torus)),
            material: materials.add(Color::WHITE.into()),
            ..Default::default()
        },
        Wireframe,
    ));
}

// torus
#[derive(Debug, Clone, Copy)]
pub struct Torus {
    pub radius: f32,
    pub ring_radius: f32,
    pub subdivisions_segments: usize,
    pub subdivisions_sides: usize,
}

impl Default for Torus {
    fn default() -> Self {
        Torus {
            radius: 1.0,
            ring_radius: 0.5,
            subdivisions_segments: 32,
            subdivisions_sides: 24,
        }
    }
}

impl From<Torus> for Mesh {
    fn from(torus: Torus) -> Self {
        // code adapted from http://apparat-engine.blogspot.com/2013/04/procedural-meshes-torus.html
        // (source code at https://github.com/SEilers/Apparat)

        let n_vertices = (torus.subdivisions_segments + 1) * (torus.subdivisions_sides + 1);
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(n_vertices);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n_vertices);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(n_vertices);

        let segment_stride = 2.0 * std::f32::consts::PI / torus.subdivisions_segments as f32;
        let side_stride = 2.0 * std::f32::consts::PI / torus.subdivisions_sides as f32;

        for segment in 0..=torus.subdivisions_segments {
            let theta = segment_stride * segment as f32;

            for side in 0..=torus.subdivisions_sides {
                let phi = side_stride * side as f32;

                let position = Vec3::new(
                    theta.cos() * (torus.radius + torus.ring_radius * phi.cos()),
                    torus.ring_radius * phi.sin(),
                    theta.sin() * (torus.radius + torus.ring_radius * phi.cos()),
                );

                let center = Vec3::new(torus.radius * theta.cos(), 0., torus.radius * theta.sin());
                let normal = (position - center).normalize();

                positions.push(position.into());
                normals.push(normal.into());
                uvs.push([
                    segment as f32 / torus.subdivisions_segments as f32,
                    side as f32 / torus.subdivisions_sides as f32,
                ]);
            }
        }

        let n_faces = (torus.subdivisions_segments) * (torus.subdivisions_sides);
        let n_triangles = n_faces * 2;
        let n_indices = n_triangles * 3;

        let mut indices: Vec<u32> = Vec::with_capacity(n_indices);

        let n_vertices_per_row = torus.subdivisions_sides + 1;
        for segment in 0..torus.subdivisions_segments {
            for side in 0..torus.subdivisions_sides {
                let lt = side + segment * n_vertices_per_row;
                let rt = (side + 1) + segment * n_vertices_per_row;

                let lb = side + (segment + 1) * n_vertices_per_row;
                let rb = (side + 1) + (segment + 1) * n_vertices_per_row;

                indices.push(lt as u32);
                indices.push(rt as u32);
                indices.push(lb as u32);

                indices.push(rt as u32);
                indices.push(rb as u32);
                indices.push(lb as u32);
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}
// end torus

// pan-orbit cam
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::render::camera::Projection;

// ANCHOR: example
/// Tags an entity as capable of panning and orbiting.
#[derive(Component)]
struct PanOrbitCamera {
    /// The "focus point" to orbit around. It is automatically updated when panning the camera
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        PanOrbitCamera {
            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,
        }
    }
}

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
fn pan_orbit_camera(
    windows: Res<Windows>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<Input<MouseButton>>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
) {
    // change input mapping for orbit and panning here
    let orbit_button = MouseButton::Right;
    let pan_button = MouseButton::Middle;

    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut orbit_button_changed = false;

    if input_mouse.pressed(orbit_button) {
        for ev in ev_motion.iter() {
            rotation_move += ev.delta;
        }
    } else if input_mouse.pressed(pan_button) {
        // Pan only if we're not rotating at the moment
        for ev in ev_motion.iter() {
            pan += ev.delta;
        }
    }
    for ev in ev_scroll.iter() {
        scroll += ev.y;
    }
    if input_mouse.just_released(orbit_button) || input_mouse.just_pressed(orbit_button) {
        orbit_button_changed = true;
    }

    for (mut pan_orbit, mut transform, projection) in query.iter_mut() {
        if orbit_button_changed {
            // only check for upside down when orbiting started or ended this frame
            // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
            let up = transform.rotation * Vec3::Y;
            pan_orbit.upside_down = up.y <= 0.0;
        }

        let mut any = false;
        if rotation_move.length_squared() > 0.0 {
            any = true;
            let window = get_primary_window_size(&windows);
            let delta_x = {
                let delta = rotation_move.x / window.x * std::f32::consts::PI * 2.0;
                if pan_orbit.upside_down {
                    -delta
                } else {
                    delta
                }
            };
            let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation = transform.rotation * pitch; // rotate around local x axis
        } else if pan.length_squared() > 0.0 {
            any = true;
            // make panning distance independent of resolution and FOV,
            let window = get_primary_window_size(&windows);
            if let Projection::Perspective(projection) = projection {
                pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window;
            }
            // translate by local axes
            let right = transform.rotation * Vec3::X * -pan.x;
            let up = transform.rotation * Vec3::Y * pan.y;
            // make panning proportional to distance away from focus point
            let translation = (right + up) * pan_orbit.radius;
            pan_orbit.focus += translation;
        } else if scroll.abs() > 0.0 {
            any = true;
            pan_orbit.radius -= scroll * pan_orbit.radius * 0.2;
            // dont allow zoom to reach zero or you get stuck
            pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
        }

        if any {
            // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
            // parent = x and y rotation
            // child = z-offset
            let rot_matrix = Mat3::from_quat(transform.rotation);
            transform.translation =
                pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
        }
    }

    // consume any remaining events, so they don't pile up if we don't need them
    // (and also to avoid Bevy warning us about not checking events every frame update)
    ev_motion.clear();
}

fn get_primary_window_size(windows: &Res<Windows>) -> Vec2 {
    let window = windows.get_primary().unwrap();
    let window = Vec2::new(window.width() as f32, window.height() as f32);
    window
}

/// Spawn a camera like this
fn spawn_camera(mut commands: Commands) {
    let translation = Vec3::new(-2.0, 2.5, 5.0);
    let radius = translation.length();

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(translation).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        PanOrbitCamera {
            radius,
            ..Default::default()
        },
    ));
}
// ANCHOR_END: example

fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // spawn a cube and a light
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
        ..Default::default()
    });
    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
        ..Default::default()
    });

    spawn_camera(commands);
}

// fn main() {
//     App::new()
//         .add_plugins(DefaultPlugins)
//         .add_startup_system(spawn_scene)
//         .add_system(pan_orbit_camera)
//         .run();

//     // just to catch compilation errors
//     let _ = App::new()
//         .add_startup_system(spawn_camera);
// }

// end pan-orbit cam
