use crate::subdivision::chunk_render;
use crate::world_noise;
use bevy::prelude::*;
use std::collections::HashSet;

pub const CHUNK_SIZE: f32 = 4.0;
const RENDER_DISTANCE: usize = 16;

/// Chunk search algorithm to generate chunks around the player
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
pub fn chunk_search(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Get start time for benchmarking
    let start = std::time::Instant::now();
    let mut total = 0;
    let mut cubes = 0;
    let mut triangles = 0;

    // Make world noise data generator
    let data_generator = world_noise::DataGenerator::new();

    let mut rendered_chunks = HashSet::new();
    let mut filled_chunks = HashSet::new();

    // Get every direction of sphere
    let step = 1;
    let pi_over_180 = std::f32::consts::PI / 180.0;
    let render_distance_f32 = RENDER_DISTANCE as f32;
    for phi in (0..180).step_by(step) {
        let phi_rad = phi as f32 * pi_over_180;
        let phi_sin = phi_rad.sin();
        let phi_cos = phi_rad.cos();

        for theta in (0..360).step_by(step) {
            let theta_rad = theta as f32 * pi_over_180;
            let theta_cos = theta_rad.cos();
            let theta_sin = theta_rad.sin();

            // Get border of render distance
            let border_x = (render_distance_f32 * phi_sin * theta_cos).round() as i32;
            let border_y = (render_distance_f32 * phi_sin * theta_sin).round() as i32;
            let border_z = (render_distance_f32 * phi_cos).round() as i32;

            // Iterate towards the border from the origin in steps of chunk size
            let direction = Vec3::new(
                border_x as f32 * CHUNK_SIZE,
                border_y as f32 * CHUNK_SIZE,
                border_z as f32 * CHUNK_SIZE,
            )
            .normalize()
                * CHUNK_SIZE;
            for distance in 0..RENDER_DISTANCE {
                let current_pos = direction * distance as f32;

                let current_chunk_x = (current_pos.x / CHUNK_SIZE).round() * CHUNK_SIZE;
                let current_chunk_y = (current_pos.y / CHUNK_SIZE).round() * CHUNK_SIZE;
                let current_chunk_z = (current_pos.z / CHUNK_SIZE).round() * CHUNK_SIZE;

                // If filled chunk, break, if rendered chunk, continue
                let key = (
                    current_chunk_x as i32,
                    current_chunk_y as i32,
                    current_chunk_z as i32,
                );
                if filled_chunks.contains(&key) {
                    break;
                }
                if !rendered_chunks.insert(key) {
                    continue;
                }

                let chunk = chunk_render(
                    &data_generator,
                    (current_chunk_x, current_chunk_z, current_chunk_y),
                    CHUNK_SIZE,
                );
                if chunk.n_cubes != 0 {
                    total += 1;
                    cubes += chunk.n_cubes;
                    triangles += chunk.n_triangles;

                    commands.spawn(PbrBundle {
                        mesh: meshes.add(chunk.mesh),
                        material: materials.add(StandardMaterial {
                            base_color: Color::WHITE,
                            metallic: 0.8,
                            perceptual_roughness: 0.3,
                            ..default()
                        }),
                        transform: Transform::from_xyz(
                            current_chunk_x,
                            current_chunk_y,
                            current_chunk_z,
                        ),
                        ..Default::default()
                    });
                }

                if chunk.n_cubes == 1 {
                    filled_chunks.insert(key);
                    break;
                }
            }
        }
    }

    println!("Total: {total} Cubes: {cubes} Triangles: {triangles}");

    // Get end time for benchmarking
    let end = std::time::Instant::now();
    println!("Time: {:#?}", end - start);
}
