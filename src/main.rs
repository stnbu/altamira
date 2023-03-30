// use bevy::input::{
//     mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseWheel},
//     ButtonState,
// };
use bevy::{
    pbr::wireframe::{Wireframe, WireframeConfig, WireframePlugin},
    prelude::*,
    render::{render_resource::WgpuFeatures, settings::WgpuSettings, RenderPlugin},
};

mod torus;
use torus::*;

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
                    transform: Transform::from_translation(Vec3::Z).looking_at(Vec3::ZERO, Vec3::Y),
                    ..Default::default()
                },
                CameraGimbal,
            ));
        });
}

use bevy::input::{
    mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseWheel},
    ButtonState,
};

fn control(
    mut commands: Commands,
    mut boom_query: Query<&mut Transform, With<CameraBoom>>,
    mut gimbal_query: Query<&mut Transform, (With<CameraGimbal>, Without<CameraBoom>)>,
    mut scroll_evr: EventReader<MouseWheel>,
    keys: Res<Input<KeyCode>>,
) {
    let mut boom = boom_query.get_single_mut().unwrap();
    let mut gimbal = gimbal_query.get_single_mut().unwrap();
    // FIXME: Handle "line" vs "pixel" at some point.
    let boom_scale_delta = scroll_evr.iter().fold(0.0_f32, |b, delta| b + delta.y);
    dbg!(boom_scale_delta);
    let new_scale = boom.scale + Vec3::ONE * boom_scale_delta / 100.0;
    let e = 0.00001;
    if new_scale.x > e && new_scale.y > e && new_scale.z > e {
        boom.scale = new_scale;
    }
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
        radius: 0.5,
        ring_radius: 0.25,
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
// end torus

/*

// //

fn mouse_motion(
    mut motion_evr: EventReader<MouseMotion>,
) {
    for ev in motion_evr.iter() {
        println!("Mouse moved: X: {} px, Y: {} px", ev.delta.x, ev.delta.y);
    }
}


*/
