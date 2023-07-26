use bevy::{
    core_pipeline::experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
    pbr::{
        NotShadowCaster, ScreenSpaceAmbientOcclusionBundle,
        ScreenSpaceAmbientOcclusionQualityLevel, ScreenSpaceAmbientOcclusionSettings,
    },
    prelude::*,
};
use bevy_debug_text_overlay::{screen_print, OverlayPlugin};
use smooth_bevy_cameras::{
    controllers::unreal::{UnrealCameraBundle, UnrealCameraController, UnrealCameraPlugin},
    LookTransformPlugin,
};

mod chunks;
mod render;
mod subdivision;
mod world_noise;
fn main() {
    App::new()
        .insert_resource(AmbientLight {
            brightness: 0.2,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(TemporalAntiAliasPlugin)
        .add_plugins(OverlayPlugin::default())
        .add_plugins((LookTransformPlugin, UnrealCameraPlugin::default()))
        .add_systems(Startup, setup)
        .add_systems(Startup, chunks::chunk_search)
        .add_systems(Update, screen_print_text)
        .run();
}

fn screen_print_text(time: Res<Time>) {
    let current_time = time.elapsed_seconds_f64();
    let at_interval = |t: f64| current_time % t < time.delta_seconds_f64();
    if at_interval(0.1) {
        let last_fps = 1.0 / time.delta_seconds();
        screen_print!("current time: {current_time:.2}");
        screen_print!(col: Color::CYAN, "fps: {last_fps:.0}");
    }
}

/// Set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands
        .spawn((
            Camera3dBundle::default(),
            FogSettings {
                color: Color::rgba(0.05, 0.05, 0.05, 1.0),
                falloff: FogFalloff::Linear {
                    start: 50.0,
                    end: 200.0,
                },
                ..default()
            },
        ))
        .insert(ScreenSpaceAmbientOcclusionBundle {
            settings: ScreenSpaceAmbientOcclusionSettings {
                quality_level: ScreenSpaceAmbientOcclusionQualityLevel::Low,
            },
            ..Default::default()
        })
        .insert(TemporalAntiAliasBundle::default())
        .insert(UnrealCameraBundle::new(
            UnrealCameraController::default(),
            Vec3::new(-2.0, 5.0, 5.0),
            Vec3::new(0., 0., 0.),
            Vec3::Y,
        ));

    // Plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    // Cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    // Sky
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::default())),
            material: materials.add(StandardMaterial {
                base_color: Color::hex("888888").unwrap(),
                unlit: true,
                cull_mode: None,
                ..default()
            }),
            transform: Transform::from_scale(Vec3::splat(1_000_000.0)),
            ..default()
        },
        NotShadowCaster,
    ));
    // Sun
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(0.98, 0.95, 0.82),
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0)
            .looking_at(Vec3::new(-0.15, -0.05, 0.25), Vec3::Y),
        ..default()
    });
}
