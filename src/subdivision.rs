use crate::render;
use crate::world_noise;
use bevy::prelude::*;

const LARGEST_CUBE_SIZE: f32 = 2.0;
const SMALLEST_CUBE_SIZE: f32 = 0.25;

pub struct Cube {
    pub pos: Vec3,
    pub size: f32,
    pub color: Color,
}

pub struct Chunk {
    pub cubes: usize,
    pub triangles: usize,
}

pub fn chunk_render(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    data_generator: &world_noise::DataGenerator,
    pos: Vec3,
    chunk_size: f32,
) -> Chunk {
    // Subdivide the cube and store the result in the cubes vector
    let mut cubes: Vec<Cube> = Vec::new();
    subdivide_cube(&mut cubes, data_generator, pos, chunk_size);
    let cubes = cubes;

    // Render the mesh
    let (render_mesh, cubes, triangles) = render::cubes(&cubes, pos);
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

    Chunk { cubes, triangles }
}

fn subdivide_cube(
    cubes: &mut Vec<Cube>,
    data_generator: &world_noise::DataGenerator,
    pos3d: Vec3,
    cube_size: f32,
) {
    let half_cube_size = cube_size / 2.0;

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

        for x in [pos3d.x - half_cube_size, pos3d.x + half_cube_size] {
            for z in [pos3d.z - half_cube_size, pos3d.z + half_cube_size] {
                let data2d = data_generator.get_data_2d(x, z);
                for y in [pos3d.y - half_cube_size, pos3d.y + half_cube_size] {
                    let is_inside = data_generator.get_data_3d(&data2d, x, z, y);
                    if is_inside {
                        n_air_cubes += 1;
                    }
                }
            }
        }
        // If fully air, skip
        if n_air_cubes == 8 {
            return;
        }
        // If air cubes in threshold range, render it
        if n_air_cubes <= max_air_cubes {
            let data2d = data_generator.get_data_2d(pos3d.x, pos3d.z);
            render_cube(cubes, data_generator, &data2d, pos3d, cube_size);
            return;
        }
    }
    // Otherwise, subdivide it into 8 smaller cubes
    for x in [-half_cube_size, half_cube_size] {
        for z in [-half_cube_size, half_cube_size] {
            for y in [-half_cube_size, half_cube_size] {
                let pos2 = pos3d + Vec3::new(x, y, z) * 0.5;
                if half_cube_size < SMALLEST_CUBE_SIZE {
                    let data2d = data_generator.get_data_2d(x, z);
                    let is_inside = data_generator.get_data_3d(&data2d, x, z, y);
                    if !is_inside {
                        render_cube(cubes, data_generator, &data2d, pos2, half_cube_size);
                    }
                } else {
                    subdivide_cube(cubes, data_generator, pos2, half_cube_size);
                }
            }
        }
    }
}

fn render_cube(
    cubes: &mut Vec<Cube>,
    data_generator: &world_noise::DataGenerator,
    data2d: &world_noise::Data2D,
    pos: Vec3,
    size: f32,
) {
    let data_color = data_generator.get_data_color(data2d, pos.x, pos.z, pos.y);
    cubes.push(Cube {
        pos,
        size,
        color: data_color.color,
    });
}
