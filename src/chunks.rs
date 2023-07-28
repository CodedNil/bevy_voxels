use crate::subdivision::{chunk_render, Chunk};
use crate::world_noise;
use bevy::prelude::*;
use rayon::prelude::*;
use std::collections::VecDeque;

pub const CHUNK_SIZE: usize = 4;
const RENDER_DISTANCE: usize = 16;

/// Chunk search algorithm to generate chunks around the player
#[allow(clippy::cast_precision_loss)]
pub fn chunk_search(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Start timer
    let start = std::time::Instant::now();
    // Create world noise data generator
    let data_generator = world_noise::DataGenerator::new();

    // Initialize state
    let mut queue = VecDeque::new();
    let mut visited =
        vec![vec![vec![false; RENDER_DISTANCE * 2]; RENDER_DISTANCE * 2]; RENDER_DISTANCE * 2];

    queue.push_back((0, 0, 0));
    visited[RENDER_DISTANCE / 2][RENDER_DISTANCE / 2][RENDER_DISTANCE / 2] = true;

    let mut chunks_to_spawn = Vec::new();
    while let Some(chunk) = queue.pop_front() {
        chunks_to_spawn.append(&mut explore_chunk(
            &mut visited,
            &mut queue,
            &data_generator,
            chunk,
        ));
    }

    // After all chunks have been explored, spawn them
    let total = chunks_to_spawn.len();
    let mut cubes = 0;
    let mut triangles = 0;
    for (chunk, neighbor) in chunks_to_spawn {
        commands.spawn(PbrBundle {
            mesh: meshes.add(chunk.mesh),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..default()
            }),
            transform: Transform::from_xyz(
                neighbor.0 as f32 * CHUNK_SIZE as f32,
                neighbor.1 as f32 * CHUNK_SIZE as f32,
                neighbor.2 as f32 * CHUNK_SIZE as f32,
            ),
            ..Default::default()
        });
        cubes += chunk.n_cubes;
        triangles += chunk.n_triangles;
    }

    println!("Total: {total} Cubes: {cubes} Triangles: {triangles}");
    println!("Time: {:#?}", start.elapsed());
}

/// Function to handle exploration of each chunk
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
fn explore_chunk(
    visited: &mut Vec<Vec<Vec<bool>>>,
    queue: &mut VecDeque<(i32, i32, i32)>,
    data_generator: &world_noise::DataGenerator,
    (chunk_x, chunk_y, chunk_z): (i32, i32, i32),
) -> Vec<(Chunk, (i32, i32, i32))> {
    let directions = [
        (-1, 0, 0),
        (1, 0, 0),
        (0, -1, 0),
        (0, 1, 0),
        (0, 0, -1),
        (0, 0, 1),
    ];

    let mut chunks_to_spawn = Vec::new();

    for &direction in &directions {
        let neighbor = (
            chunk_x + direction.0,
            chunk_y + direction.1,
            chunk_z + direction.2,
        );

        let voxel = (
            neighbor.0 + RENDER_DISTANCE as i32,
            neighbor.1 + RENDER_DISTANCE as i32,
            neighbor.2 + RENDER_DISTANCE as i32,
        );

        let is_out_of_bounds = voxel.0 < 0
            || voxel.1 < 0
            || voxel.2 < 0
            || voxel.0 >= RENDER_DISTANCE as i32 * 2
            || voxel.1 >= RENDER_DISTANCE as i32 * 2
            || voxel.2 >= RENDER_DISTANCE as i32 * 2;
        let is_visited = visited[voxel.0 as usize][voxel.1 as usize][voxel.2 as usize];
        if is_out_of_bounds || is_visited {
            continue;
        }
        visited[voxel.0 as usize][voxel.1 as usize][voxel.2 as usize] = true;

        let render_result = chunk_render(
            data_generator,
            (
                neighbor.0 as f32 * CHUNK_SIZE as f32,
                neighbor.2 as f32 * CHUNK_SIZE as f32,
                neighbor.1 as f32 * CHUNK_SIZE as f32,
            ),
            CHUNK_SIZE as f32,
        );
        let blocking = render_result.n_cubes == 1;

        chunks_to_spawn.push((render_result, neighbor));

        if !blocking {
            queue.push_back(neighbor);
        }
    }

    chunks_to_spawn
}
