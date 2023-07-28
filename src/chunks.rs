use crate::subdivision::{self, chunk_render};
use crate::world_noise;
use bevy::prelude::*;
use std::collections::VecDeque;

pub const CHUNK_SIZE: usize = 4;
const RENDER_DISTANCE: usize = 16;

/// Chunk search algorithm to generate chunks around the player
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
    let mut state = ChunkSearchState::new();

    while let Some(chunk) = state.queue.pop_front() {
        explore_chunk(
            &mut state,
            &mut commands,
            &mut meshes,
            &mut materials,
            &data_generator,
            chunk,
        );
    }

    println!(
        "Total: {total} Cubes: {cubes} Triangles: {triangles}",
        total = state.total,
        cubes = state.cubes,
        triangles = state.triangles
    );
    println!("Time: {:#?}", start.elapsed());
}

/// Contains state for chunk search
struct ChunkSearchState {
    queue: VecDeque<(i32, i32, i32)>,
    visited: Vec<Vec<Vec<bool>>>,
    total: usize,
    cubes: usize,
    triangles: usize,
}

impl ChunkSearchState {
    fn new() -> Self {
        let mut queue = VecDeque::new();
        let mut visited =
            vec![vec![vec![false; RENDER_DISTANCE * 2]; RENDER_DISTANCE * 2]; RENDER_DISTANCE * 2];

        queue.push_back((0, 0, 0));
        visited[RENDER_DISTANCE / 2][RENDER_DISTANCE / 2][RENDER_DISTANCE / 2] = true;

        Self {
            queue,
            visited,
            total: 0,
            cubes: 0,
            triangles: 0,
        }
    }
}

/// Function to handle exploration of each chunk
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
fn explore_chunk(
    state: &mut ChunkSearchState,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    data_generator: &world_noise::DataGenerator,
    (chunk_x, chunk_y, chunk_z): (i32, i32, i32),
) {
    let directions = [
        (-1, 0, 0),
        (1, 0, 0),
        (0, -1, 0),
        (0, 1, 0),
        (0, 0, -1),
        (0, 0, 1),
    ];

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
        let is_visited = state.visited[voxel.0 as usize][voxel.1 as usize][voxel.2 as usize];
        if is_out_of_bounds || is_visited {
            continue;
        }
        state.visited[voxel.0 as usize][voxel.1 as usize][voxel.2 as usize] = true;

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

        state.cubes += render_result.n_cubes;
        state.triangles += render_result.n_triangles;

        spawn_chunk(commands, meshes, materials, render_result, neighbor);

        state.total += 1;

        if !blocking {
            state.queue.push_back(neighbor);
        }
    }
}

/// Function to spawn a chunk
#[allow(clippy::cast_precision_loss)]
fn spawn_chunk(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    render_result: subdivision::Chunk,
    neighbor: (i32, i32, i32),
) {
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
}
