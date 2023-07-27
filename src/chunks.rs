use crate::subdivision::chunk_render;
use crate::world_noise;
use bevy::prelude::*;
use rayon::prelude::*;

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

    // Get chunks in efficient order
    let rotate_angles = 8 * RENDER_DISTANCE;
    let angle_per = (360.0 / rotate_angles as f32).to_radians();
    // Store chunks in a set to prevent duplicates
    let mut chunks = std::collections::HashSet::new();
    // Store number of hits each angle has had, a lookup with angle as index and number of hits as value
    let mut hits = vec![0; 2 * rotate_angles];

    // Get y levels to render eg 0, -1, 1, -2, 2, 3, 4
    let (y_min, y_max) = (-2_i32, 4_i32);
    let y_levels: Vec<i32> = std::iter::once(0)
        .chain((1..=y_min.abs().max(y_max)).flat_map(|i| {
            std::iter::once(-i)
                .filter(move |_| -i >= y_min)
                .chain(std::iter::once(i).filter(move |_| i <= y_max))
        }))
        .collect();

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
                let mut filled_y = 0;
                let results: Vec<_> = y_levels
                    .par_iter()
                    .map(|&y| {
                        chunk_render(
                            &data_generator,
                            (
                                next_chunk.0 as f32,
                                next_chunk.1 as f32,
                                y as f32 * CHUNK_SIZE,
                            ),
                            CHUNK_SIZE,
                        )
                    })
                    .collect();
                // Sum up the results from all threads
                for chunk in results {
                    total += 1;
                    cubes += chunk.n_cubes;
                    triangles += chunk.n_triangles;

                    // If chunk has been filled entirely,
                    if chunk.n_cubes == 8 {
                        filled_y += 1;
                    }

                    let (chunk_x, chunk_z, chunk_y) = chunk.chunk_pos;
                    commands.spawn(PbrBundle {
                        mesh: meshes.add(chunk.mesh),
                        material: materials.add(StandardMaterial {
                            base_color: Color::WHITE,
                            metallic: 0.8,
                            perceptual_roughness: 0.3,
                            ..default()
                        }),
                        transform: Transform::from_xyz(chunk_x, chunk_y, chunk_z),
                        ..Default::default()
                    });
                }
                // If chunk has been filled entirely, skip the rest of the chunks in this direction
                if filled_y == y_levels.len() {
                    hits[angle_i] += RENDER_DISTANCE;
                }
            }
        }
    }

    println!("Total: {total} Cubes: {cubes} Triangles: {triangles}");

    // Get end time for benchmarking
    let end = std::time::Instant::now();
    println!("Time: {:#?}", end - start);
}
