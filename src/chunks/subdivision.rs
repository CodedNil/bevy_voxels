use crate::chunks::render;
use crate::chunks::world_noise::{Data2D, DataGenerator};
use bevy::prelude::*;
use rayon::prelude::*;

const SMALLEST_CUBE_SIZE: f32 = 0.25;

pub struct Cube {
    pub pos: Vec3,
    pub size: f32,
    pub color: Vec3,
}

pub struct Chunk {
    pub mesh: Option<Mesh>,
    pub chunk_pos: Vec3,
    pub n_cubes: usize,
    pub n_triangles: usize,
}

#[allow(clippy::cast_precision_loss)]
pub fn chunk_render(data_generator: &DataGenerator, chunk_pos: Vec3, chunk_size: f32) -> Chunk {
    let cubes: Vec<Cube> = subdivide_cube(data_generator, chunk_pos, chunk_size);
    let (render_mesh, n_triangles) = if cubes.is_empty() {
        (None, 0)
    } else {
        let (mesh, triangles) = render::cubes_mesh(&cubes, chunk_pos);
        (Some(mesh), triangles)
    };
    Chunk {
        mesh: render_mesh,
        chunk_pos,
        n_cubes: cubes.len(),
        n_triangles,
    }
}

#[allow(clippy::cast_precision_loss)]
fn subdivide_cube(data_generator: &DataGenerator, cube_pos: Vec3, cube_size: f32) -> Vec<Cube> {
    let (px, py, pz) = cube_pos.into();
    let mut cubes: Vec<Cube> = Vec::new();

    let half_cube_size = cube_size / 2.0;
    let quarter_cube_size = cube_size / 4.0;

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
        cubes.push(render_cube(data_generator, &data2d, cube_pos, cube_size));
        return cubes;
    }

    // Otherwise, subdivide it into 8 smaller cubes
    let new_cubes: Vec<Cube> = (0..8)
        .into_par_iter()
        .flat_map(|i| {
            let corner_pos = Vec3::new(
                px + ((i & 1) * 2 - 1) as f32 * quarter_cube_size,
                py + ((i >> 2 & 1) * 2 - 1) as f32 * quarter_cube_size,
                pz + ((i >> 1 & 1) * 2 - 1) as f32 * quarter_cube_size,
            );
            let (c_pos_x, c_pos_y, c_pos_z) = corner_pos.into();

            let mut local_cubes: Vec<Cube> = Vec::new();
            if half_cube_size < SMALLEST_CUBE_SIZE {
                let data2d = data_generator.get_data_2d(c_pos_x, c_pos_z);
                let is_inside = data_generator.get_data_3d(&data2d, c_pos_x, c_pos_z, c_pos_y);
                if !is_inside {
                    local_cubes.push(render_cube(
                        data_generator,
                        &data2d,
                        corner_pos,
                        half_cube_size,
                    ));
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

fn render_cube(data_generator: &DataGenerator, data2d: &Data2D, pos: Vec3, size: f32) -> Cube {
    let data_color = data_generator.get_data_color(data2d, pos.x, pos.z, pos.y);
    Cube {
        pos: data_color.pos_jittered,
        size: size * 1.175,
        color: data_color.color,
    }
}
