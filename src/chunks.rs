use crate::subdivision::chunk_render;
use crate::world_noise;
use bevy::prelude::*;
use std::collections::VecDeque;

pub const CHUNK_SIZE: usize = 4;
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

    let mut queue = VecDeque::new();
    let mut visited =
        vec![vec![vec![false; RENDER_DISTANCE * 2]; RENDER_DISTANCE * 2]; RENDER_DISTANCE * 2];

    queue.push_back((0, 0, 0));
    visited[RENDER_DISTANCE / 2][RENDER_DISTANCE / 2][RENDER_DISTANCE / 2] = true;

    let directions = [
        (-1, 0, 0),
        (1, 0, 0),
        (0, -1, 0),
        (0, 1, 0),
        (0, 0, -1),
        (0, 0, 1),
    ];

    while let Some(chunk) = queue.pop_front() {
        for &direction in &directions {
            let neighbor = (
                chunk.0 + direction.0,
                chunk.1 + direction.1,
                chunk.2 + direction.2,
            );

            let voxel = (
                neighbor.0 + RENDER_DISTANCE as i32,
                neighbor.1 + RENDER_DISTANCE as i32,
                neighbor.2 + RENDER_DISTANCE as i32,
            );

            if voxel.0 < 0
                || voxel.1 < 0
                || voxel.2 < 0
                || voxel.0 >= RENDER_DISTANCE as i32 * 2
                || voxel.1 >= RENDER_DISTANCE as i32 * 2
                || voxel.2 >= RENDER_DISTANCE as i32 * 2
            {
                continue;
            }

            if visited[voxel.0 as usize][voxel.1 as usize][voxel.2 as usize] {
                continue;
            }
            visited[voxel.0 as usize][voxel.1 as usize][voxel.2 as usize] = true;

            let render_result = chunk_render(
                &data_generator,
                (
                    neighbor.0 as f32 * CHUNK_SIZE as f32,
                    neighbor.2 as f32 * CHUNK_SIZE as f32,
                    neighbor.1 as f32 * CHUNK_SIZE as f32,
                ),
                CHUNK_SIZE as f32,
            );
            let blocking = render_result.n_cubes == 1;

            cubes += render_result.n_cubes;
            triangles += render_result.n_triangles;

            commands.spawn(PbrBundle {
                mesh: meshes.add(render_result.mesh),
                material: materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    metallic: 0.8,
                    perceptual_roughness: 0.3,
                    ..default()
                }),
                transform: Transform::from_xyz(
                    neighbor.0 as f32 * CHUNK_SIZE as f32,
                    neighbor.1 as f32 * CHUNK_SIZE as f32,
                    neighbor.2 as f32 * CHUNK_SIZE as f32,
                ),
                ..Default::default()
            });

            total += 1;

            if !blocking {
                queue.push_back(neighbor);
            }
        }
    }

    println!("Total: {total} Cubes: {cubes} Triangles: {triangles}");

    // Get end time for benchmarking
    let end = std::time::Instant::now();
    println!("Time: {:#?}", end - start);
}
