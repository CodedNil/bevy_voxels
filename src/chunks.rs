use crate::subdivision::chunk_render;
use crate::world_noise;
use bevy::prelude::*;

const CHUNK_SIZE: f32 = 8.0;
const RENDER_DISTANCE: usize = 4;

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

    // Get chunks in efficient order
    let rotate_angles = 8 * RENDER_DISTANCE;
    let angle_per = (360.0 / rotate_angles as f32).to_radians();
    // Store chunks in a set to prevent duplicates
    let mut chunks = std::collections::HashSet::new();
    // Store number of hits each angle has had, a lookup with angle as index and number of hits as value
    let mut hits = vec![0; 2 * rotate_angles];

    // Make world noise data generator
    let data_generator = world_noise::DataGenerator::new();

    for a in 0..rotate_angles {
        for b in 0..a {
            // Get direction to travel
            let angle_pos = b as f32 * angle_per;
            let cos_angle_pos = angle_pos.cos();
            let sin_angle_pos = angle_pos.sin();

            for &c in &[-1, 1] {
                // Gets angle in order 0, 0 11.25 -11.25, 0 11.25 -11.25 22.5 -22.5
                if angle_pos == 0.0 && c == 1 {
                    continue;
                }

                // Choose direction based on c
                let dir = if c == 1 {
                    Vec2::new(cos_angle_pos, -sin_angle_pos)
                } else {
                    Vec2::new(cos_angle_pos, sin_angle_pos)
                };
                // Get angle index for hits shifted by offset
                let angle_i = (b as isize * c as isize + rotate_angles as isize).unsigned_abs();

                // If hit count is greater than render distance, skip
                if hits[angle_i] > RENDER_DISTANCE {
                    continue;
                }
                let distance = hits[angle_i] as f32;
                // Increment hit count
                hits[angle_i] += 1;

                // Round next chunk to nearest chunk size on each axis
                let next_chunk = (
                    ((dir.x * distance).round() * CHUNK_SIZE) as i32,
                    ((dir.y * distance).round() * CHUNK_SIZE) as i32,
                );
                // If chunk is already in list, skip
                if !chunks.insert(next_chunk) {
                    continue;
                }

                // Render chunk
                for y in [0, -1, 1, 2] {
                    let chunk = chunk_render(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &data_generator,
                        Vec3::new(
                            next_chunk.0 as f32,
                            y as f32 * CHUNK_SIZE,
                            next_chunk.1 as f32,
                        ),
                        CHUNK_SIZE,
                    );
                    total += 1;
                    cubes += chunk.cubes;
                    triangles += chunk.triangles;
                }
            }
        }
    }

    println!("Total: {total} Cubes: {cubes} Triangles: {triangles}");

    // Get end time for benchmarking
    let end = std::time::Instant::now();
    println!("Time: {:#?}", end - start);
}
