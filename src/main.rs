use bevy::prelude::*;
use smooth_bevy_cameras::{
    controllers::unreal::{UnrealCameraBundle, UnrealCameraController, UnrealCameraPlugin},
    LookTransformPlugin,
};

const CHUNK_SIZE: f32 = 8.0;
const LARGEST_VOXEL_SIZE: f32 = 4.0;
const SMALLEST_VOXEL_SIZE: f32 = 0.25;
const RENDER_DISTANCE: i32 = 8;
const RENDER_DISTANCE_VERTICAL: i32 = 2;

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins)
        .add_plugins(LookTransformPlugin)
        .add_plugins(UnrealCameraPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Startup, chunk_search)
        // .add_systems(Update, spawn_cube)
        .run();
}

/// Chunk search algorithm to generate chunks around the player
fn chunk_search(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(&Transform, &Camera)>,
) {
    // First get all the border chunks at render distance in a circle, all chunks on the edge of the circle are border chunks
    // Do this in rotational steps, starting with chunks at 0, 90, -90, 180 degrees, renderdistance away
    // Step from center towards the border chunk, on chunk at a time, if this chunk isnt already in the list of chunks, add it
    //

    // Bake out 2 lists of chunk search orders, for straight on and 45 degree angle, these can then be rotated to have 8 lists based on players rotation

    // Get x to render distance
    for x in 0..RENDER_DISTANCE {
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: CHUNK_SIZE })),
            material: materials
                .add(Color::rgb((x as f32) / (RENDER_DISTANCE as f32), 0.0, 0.0).into()),
            transform: Transform::from_translation(Vec3::new((x as f32) * CHUNK_SIZE, 0.0, 0.0)),
            ..default()
        });
    }
}

// fn spawn_cube(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     query: Query<(&Transform, &Camera)>,
//     cube_query: Query<(&Transform, &Handle<Mesh>)>,
// ) {
//     for (transform, _) in query.iter() {
//         let cube_mesh = meshes.add(Mesh::from(shape::Cube { size: 0.25 }));
//         let cube_material = materials.add(Color::rgb(1.0, 0.0, 0.0).into());

//         // Round the translation coordinates to the nearest whole number
//         let grid_x = (transform.translation.x * 4.0).round() / 4.0;
//         let grid_y = ((transform.translation.y - 1.0) * 4.0).round() / 4.0;
//         let grid_z = (transform.translation.z * 4.0).round() / 4.0;
//         let grid = Vec3::new(grid_x, grid_y, grid_z);

//         let mut cube_exists = false;
//         for (cube_transform, mesh_handle) in cube_query.iter() {
//             if (cube_transform.translation - grid).length() < 0.01 {
//                 cube_exists = true;
//                 break;
//             }
//         }

//         if !cube_exists {
//             commands.spawn(PbrBundle {
//                 mesh: cube_mesh,
//                 material: cube_material,
//                 transform: Transform::from_translation(Vec3::new(grid_x, grid_y, grid_z)),
//                 ..default()
//             });
//             println!("Spawned cube at {:?}", grid);
//         }
//     }
// }

/// Set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands
        .spawn(Camera3dBundle::default())
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
    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}
