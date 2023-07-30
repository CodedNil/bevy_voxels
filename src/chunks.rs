// mod raycast;
mod render;
mod subdivision;
mod world_noise;

use bevy::prelude::*;
use rayon::prelude::*;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use subdivision::{chunk_render, Chunk};

pub const CHUNK_SIZE: f32 = 2.0;
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
const RENDER_DISTANCE: usize = (16f32 / CHUNK_SIZE) as usize;

type VisitedSet = Arc<Mutex<HashSet<(i32, i32, i32)>>>;

struct ExploreResult {
    chunks: Vec<Chunk>,
    new_queue: Vec<(i32, i32, i32)>,
}

/// Chunk search algorithm to generate chunks around the player
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
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
    let mut queue = Vec::new();
    let visited: VisitedSet = Arc::default();

    queue.push((0, 0, 0));

    let mut chunks: Vec<Chunk> = Vec::new();
    while !queue.is_empty() {
        let results: Vec<ExploreResult> = queue
            .par_iter()
            .map(|&chunk| explore_chunk(&visited, &data_generator, chunk))
            .collect();
        queue.clear();
        for result in results {
            chunks.extend(result.chunks);
            queue.extend(result.new_queue);
        }
    }

    // After all chunks have been explored, spawn them
    let total = chunks.len();
    let mut cubes = 0;
    let mut triangles = 0;

    for chunk in chunks {
        if let Some(mesh) = chunk.mesh {
            commands.spawn(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    ..default()
                }),
                transform: Transform::from_translation(chunk.chunk_pos),
                ..Default::default()
            });
        }
        cubes += chunk.n_cubes;
        triangles += chunk.n_triangles;
    }

    println!("Total: {total} Cubes: {cubes} Triangles: {triangles}");
    println!("Time: {:#?}", start.elapsed());
}

/// Function to handle exploration of each chunk
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
fn explore_chunk(
    visited: &VisitedSet,
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
    let mut new_queue = Vec::new();

    for &direction in &directions {
        let neighbor = (
            chunk_x + direction.0,
            chunk_y + direction.1,
            chunk_z + direction.2,
        );
        // Get position in visited array
        let neighbor_normalised = (
            neighbor.0 + RENDER_DISTANCE as i32,
            neighbor.1 + RENDER_DISTANCE as i32,
            neighbor.2 + RENDER_DISTANCE as i32,
        );

        let is_out_of_bounds = neighbor_normalised.0 < 0
            || neighbor_normalised.1 < 0
            || neighbor_normalised.2 < 0
            || neighbor_normalised.0 > RENDER_DISTANCE as i32 * 2
            || neighbor_normalised.1 > RENDER_DISTANCE as i32 * 2
            || neighbor_normalised.2 > RENDER_DISTANCE as i32 * 2;
        if is_out_of_bounds {
            continue;
        }
        if visited.lock().unwrap().contains(&neighbor_normalised) {
            continue;
        }
        // Calculate the distance from the origin, only create the chunk if it's within the render distance
        let distance = ((neighbor.0.pow(2) + neighbor.1.pow(2) + neighbor.2.pow(2)) as f32).sqrt();
        if distance > RENDER_DISTANCE as f32 {
            continue;
        }

        visited.lock().unwrap().insert(neighbor_normalised);

        let chunk = chunk_render(
            data_generator,
            Vec3::new(
                neighbor.0 as f32 * CHUNK_SIZE,
                neighbor.2 as f32 * CHUNK_SIZE,
                neighbor.1 as f32 * CHUNK_SIZE,
            ),
            CHUNK_SIZE,
        );

        let blocking = chunk.n_cubes == 1;
        // If chunk is empty don't render it
        if chunk.n_cubes > 0 {
            chunks.push(chunk);
        }
        // If chunk is blocking, don't explore it further
        if !blocking {
            new_queue.push(neighbor);
        }
    }

    ExploreResult { chunks, new_queue }
}
