use crate::subdivision::{chunk_render, Chunk};
use crate::world_noise;
use bevy::prelude::*;
use rayon::prelude::*;
use std::collections::VecDeque;

pub const CHUNK_SIZE: usize = 4;
const RENDER_DISTANCE: usize = 16;

struct ExploreResult {
    chunks: Vec<Chunk>,
    new_visited: Vec<Vec<Vec<bool>>>,
    new_queue: VecDeque<(i32, i32, i32)>,
}

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

    let mut chunks: Vec<Chunk> = Vec::new();
    while let Some(chunk) = queue.pop_front() {
        let results = explore_chunk(&visited, &data_generator, chunk);
        chunks.extend(results.chunks);
        queue.extend(results.new_queue);
        for (i, new_visited_row) in results.new_visited.iter().enumerate() {
            for (j, new_visited_col) in new_visited_row.iter().enumerate() {
                for (k, new_visited_val) in new_visited_col.iter().enumerate() {
                    visited[i][j][k] = *new_visited_val || visited[i][j][k];
                }
            }
        }
    }

    // After all chunks have been explored, spawn them
    let total = chunks.len();
    let mut cubes = 0;
    let mut triangles = 0;
    
    for chunk in chunks {
        commands.spawn(PbrBundle {
            mesh: meshes.add(chunk.mesh),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..default()
            }),
            transform: Transform::from_xyz(
                chunk.chunk_pos.0 as f32,
                chunk.chunk_pos.2 as f32,
                chunk.chunk_pos.1 as f32,
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
    visited: &[Vec<Vec<bool>>],
    data_generator: &world_noise::DataGenerator,
    (chunk_x, chunk_y, chunk_z): (i32, i32, i32),
) -> ExploreResult {
    let directions = [
        (-1, 0, 0),
        (1, 0, 0),
        (0, -1, 0),
        (0, 1, 0),
        (0, 0, -1),
        (0, 0, 1),
    ];

    let mut chunks = Vec::new();

    // Create empty visited and queue to add new data to
    let mut new_visited =
        vec![vec![vec![false; RENDER_DISTANCE * 2]; RENDER_DISTANCE * 2]; RENDER_DISTANCE * 2];
    let mut new_queue = VecDeque::new();

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

        if is_out_of_bounds {
            continue;
        }
        let is_visited1 = visited[voxel.0 as usize][voxel.1 as usize][voxel.2 as usize];
        let is_visited2 = new_visited[voxel.0 as usize][voxel.1 as usize][voxel.2 as usize];
        if is_visited1 || is_visited2 {
            continue;
        }
        new_visited[voxel.0 as usize][voxel.1 as usize][voxel.2 as usize] = true;

        let chunk = chunk_render(
            data_generator,
            (
                neighbor.0 * CHUNK_SIZE as i32,
                neighbor.2 * CHUNK_SIZE as i32,
                neighbor.1 * CHUNK_SIZE as i32,
            ),
            CHUNK_SIZE,
        );
        let blocking = chunk.n_cubes == 1;
        chunks.push(chunk);

        if !blocking {
            new_queue.push_back(neighbor);
        }
    }

    ExploreResult {
        chunks,
        new_visited,
        new_queue,
    }
}
