use bevy::prelude::*;
use smooth_bevy_cameras::{
    controllers::unreal::{UnrealCameraBundle, UnrealCameraController, UnrealCameraPlugin},
    LookTransformPlugin,
};

const CHUNK_SIZE: f32 = 8.0;
const LARGEST_VOXEL_SIZE: f32 = 4.0;
const SMALLEST_VOXEL_SIZE: f32 = 0.25;
const RENDER_DISTANCE: i32 = 16;
const RENDER_DISTANCE_VERTICAL: i32 = 2;

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins)
        .add_plugins(LookTransformPlugin)
        .add_plugins(UnrealCameraPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Startup, chunk_search)
        .run();
}

/// Chunk search algorithm to generate chunks around the player
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
fn chunk_search(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Get start time for benchmarking
    let start = std::time::Instant::now();

    // Get chunks in efficient order
    let rotate_angles = 8 * RENDER_DISTANCE;
    let angle_per = 360.0 / rotate_angles as f32;
    let mut total = 0;
    // Store chunks in a set to prevent duplicates
    let mut chunks = std::collections::HashSet::new();
    // Store number of hits each angle has had, a lookup with angle as index and number of hits as value
    let mut hits = vec![0; 2 * rotate_angles as usize];
    let offset = rotate_angles as isize;

    for a in 0..rotate_angles {
        for b in 0..a {
            for c in 0..2 {
                // Gets angle in order 0, 0 11.25 -11.25, 0 11.25 -11.25 22.5 -22.5
                let angle = (b as f32 * angle_per) * (if c == 1 { -1.0 } else { 1.0 });
                if angle == 0.0 && c == 1 {
                    continue;
                }
                // Step forwards from angle
                let angle_rad = angle.to_radians();
                let dir = Vec3::new(angle_rad.cos(), 0.0, angle_rad.sin());

                // Get angle index for hits shifted by offset
                let angle_i =
                    ((b * (if c == 1 { -1 } else { 1 })) as isize + offset as isize) as usize;
                // If hit count is greater than render distance, skip
                if hits[angle_i] > RENDER_DISTANCE {
                    continue;
                }
                // Increment hit count
                hits[angle_i] += 1;

                let distance = *hits.get(angle_i).unwrap_or(&0) as f32;

                // Round next chunk to nearest chunk size on each axis
                let next_chunk = (
                    ((dir.x * distance).round() * CHUNK_SIZE) as i32,
                    ((dir.y * distance).round() * CHUNK_SIZE) as i32,
                    ((dir.z * distance).round() * CHUNK_SIZE) as i32,
                );
                // If chunk is already in list, skip
                if !chunks.insert(next_chunk) {
                    continue;
                }
                // Get chunk as bevy Vec3
                commands.spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: CHUNK_SIZE })),
                    material: materials
                        .add(Color::rgb(1.0 - (chunks.len() as f32 / 1000.0), 0.0, 0.0).into()),
                    transform: Transform::from_translation(Vec3::new(
                        next_chunk.0 as f32,
                        next_chunk.1 as f32,
                        next_chunk.2 as f32,
                    )),
                    ..default()
                });
                total += 1;
            }
        }
    }

    println!("Total: {total}");

    // Get end time for benchmarking
    let end = std::time::Instant::now();
    println!("Time: {:#?}", end - start);
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
    // commands.spawn(PointLightBundle {
    //     point_light: PointLight {
    //         intensity: 1500.0,
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     transform: Transform::from_xyz(4.0, 8.0, 4.0),
    //     ..default()
    // });
}
