use bevy::prelude::*;

use crate::subdivision::chunk_render;

const CHUNK_SIZE: f32 = 8.0;
const RENDER_DISTANCE: i32 = 2;

/// Chunk search algorithm to generate chunks around the player
pub fn chunk_search(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Get start time for benchmarking
    let start = std::time::Instant::now();

    // Get chunks in efficient order
    let rotate_angles = 8 * RENDER_DISTANCE;
    let angle_per = (360.0 / rotate_angles as f32).to_radians();
    let mut total = 0;
    // Store chunks in a set to prevent duplicates
    let mut chunks = std::collections::HashSet::new();
    // Store number of hits each angle has had, a lookup with angle as index and number of hits as value
    let mut hits = vec![0; 2 * rotate_angles as usize];
    let offset = rotate_angles as isize;

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
                let angle_i = ((b * c) as isize + offset) as usize;

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
                commands.spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: CHUNK_SIZE })),
                    material: materials
                        .add(Color::rgb(1.0 - (chunks.len() as f32 / 21.0), 0.0, 0.0).into()),
                    transform: Transform::from_translation(Vec3::new(
                        next_chunk.0 as f32,
                        -CHUNK_SIZE,
                        next_chunk.1 as f32,
                    )),
                    ..Default::default()
                });
                chunk_render(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    next_chunk.0 as f32,
                    next_chunk.1 as f32,
                    CHUNK_SIZE,
                );
                total += 1;
            }
        }
    }

    println!("Total: {total}");

    // Get end time for benchmarking
    let end = std::time::Instant::now();
    println!("Time: {:#?}", end - start);
}
