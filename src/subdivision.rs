use crate::render;
use crate::world_noise::{Data2D, DataGenerator};
use bevy::prelude::*;
use rayon::prelude::*;

const LARGEST_CUBE_SIZE: f32 = 2.0;
const SMALLEST_CUBE_SIZE: f32 = 0.25;

pub struct Cube {
    pub pos: Vec3,
    pub size: f32,
    pub color: Color,
}

pub struct Chunk {
    pub n_cubes: usize,
    pub n_triangles: usize,
}

pub fn chunk_render(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    data_generator: &DataGenerator,
    pos: Vec3,
    chunk_size: f32,
) -> Chunk {
    // Subdivide the cube and store the result in the cubes vector
    let cubes: Vec<Cube> = subdivide_cube(data_generator, (pos.x, pos.z, pos.y), chunk_size);

    // Render the mesh
    let (render_mesh, n_triangles) = render::cubes(&cubes, pos);
    commands.spawn(PbrBundle {
        mesh: meshes.add(render_mesh),
        material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            metallic: 0.8,
            perceptual_roughness: 0.3,
            ..default()
        }),
        transform: Transform::from_xyz(pos.x, pos.y, pos.z),
        ..Default::default()
    });

    Chunk {
        n_cubes: cubes.len(),
        n_triangles,
    }
}

#[allow(clippy::cast_precision_loss)]
fn subdivide_cube(
    data_generator: &DataGenerator,
    pos: (f32, f32, f32),
    cube_size: f32,
) -> Vec<Cube> {
    let (px, pz, py) = pos;
    let mut cubes: Vec<Cube> = Vec::new();

    let half_cube_size = cube_size / 2.0;
    let quarter_cube_size = cube_size / 4.0;

    if cube_size <= LARGEST_CUBE_SIZE {
        // Calculate how much of the cube is air
        let mut n_air_cubes = 0;
        // Smaller cubes have higher threshold for air, so less small cubes made
        let max_air_cubes: i32 = match cube_size {
            x if (x - 0.25).abs() < f32::EPSILON => 4,
            x if (x - 0.5).abs() < f32::EPSILON => 2,
            x if (x - 1.0).abs() < f32::EPSILON => 1,
            _ => 0,
        };

        for x in [px - half_cube_size, px + half_cube_size] {
            for z in [pz - half_cube_size, pz + half_cube_size] {
                let data2d = data_generator.get_data_2d(x, z);
                for y in [py - half_cube_size, py + half_cube_size] {
                    let is_inside = data_generator.get_data_3d(&data2d, x, z, y);
                    if is_inside {
                        n_air_cubes += 1;
                    }
                }
            }
        }
        // If fully air, skip
        if n_air_cubes == 8 {
            return cubes;
        }
        // If air cubes in threshold range, render it
        if n_air_cubes <= max_air_cubes {
            let data2d = data_generator.get_data_2d(px, pz);
            cubes.push(render_cube(data_generator, &data2d, (px, pz, py), cube_size));
            return cubes;
        }
    }

    // Otherwise, subdivide it into 8 smaller cubes
    let new_cubes: Vec<Cube> = (0..8)
        .into_par_iter()
        .flat_map(|i| {
            let corner_pos = (
                px + ((i & 1) * 2 - 1) as f32 * quarter_cube_size,
                pz + ((i >> 1 & 1) * 2 - 1) as f32 * quarter_cube_size,
                py + ((i >> 2 & 1) * 2 - 1) as f32 * quarter_cube_size,
            );
            let (c_pos_x, c_pos_z, c_pos_y) = corner_pos;

            let mut local_cubes: Vec<Cube> = Vec::new();
            if half_cube_size < SMALLEST_CUBE_SIZE {
                let data2d = data_generator.get_data_2d(c_pos_x, c_pos_z);
                let is_inside = data_generator.get_data_3d(&data2d, c_pos_x, c_pos_z, c_pos_y);
                if !is_inside {
                    local_cubes.push(render_cube(data_generator, &data2d, corner_pos, half_cube_size));
                }
            } else {
                local_cubes = subdivide_cube(data_generator, corner_pos, half_cube_size);
            }
            local_cubes.into_par_iter()
        })
        .collect();
    cubes.par_extend(new_cubes);

    cubes
}

fn render_cube(
    data_generator: &DataGenerator,
    data2d: &Data2D,
    pos: (f32, f32, f32),
    size: f32,
) -> Cube {
    let (px, pz, py) = pos;
    let data_color = data_generator.get_data_color(data2d, px, pz, py);
    Cube {
        pos: data_color.pos_jittered,
        size: size * 1.175,
        color: data_color.color,
    }
}
