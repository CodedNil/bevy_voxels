use crate::render;
use crate::world_noise;
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
    data_generator: &world_noise::DataGenerator,
    pos: Vec3,
    chunk_size: f32,
) -> Chunk {
    // Subdivide the cube and store the result in the cubes vector
    let cubes: Vec<Cube> = subdivide_cube(data_generator, pos, chunk_size);

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

fn subdivide_cube(
    data_generator: &world_noise::DataGenerator,
    pos3d: Vec3,
    cube_size: f32,
) -> Vec<Cube> {
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
            return cubes;
        }
        // If air cubes in threshold range, render it
        if n_air_cubes <= max_air_cubes {
            let data2d = data_generator.get_data_2d(pos3d.x, pos3d.z);
            let cube = render_cube(data_generator, &data2d, pos3d, cube_size);
            cubes.push(cube);
            return cubes;
        }
    }

    // Otherwise, subdivide it into 8 smaller cubes
    let mut sub_cubes_positions = Vec::with_capacity(8);
    for x in [pos3d.x - quarter_cube_size, pos3d.x + quarter_cube_size] {
        for z in [pos3d.z - quarter_cube_size, pos3d.z + quarter_cube_size] {
            for y in [pos3d.y - quarter_cube_size, pos3d.y + quarter_cube_size] {
                sub_cubes_positions.push([x, z, y]);
            }
        }
    }
    let sub_cubes_positions = sub_cubes_positions;

    let new_cubes: Vec<Cube> = sub_cubes_positions
        .par_iter()
        .flat_map(|&pos| {
            let mut local_cubes: Vec<Cube> = Vec::new();
            if half_cube_size < SMALLEST_CUBE_SIZE {
                let data2d = data_generator.get_data_2d(pos[0], pos[1]);
                let is_inside = data_generator.get_data_3d(&data2d, pos[0], pos[1], pos[2]);
                if !is_inside {
                    let pos_vec3 = Vec3::new(pos[0], pos[2], pos[1]);
                    let cube = render_cube(data_generator, &data2d, pos_vec3, half_cube_size);
                    local_cubes.push(cube);
                }
            } else {
                let pos_vec3 = Vec3::new(pos[0], pos[2], pos[1]);
                local_cubes = subdivide_cube(data_generator, pos_vec3, half_cube_size);
            }
            local_cubes.into_par_iter()
        })
        .collect();
    cubes.par_extend(new_cubes);

    cubes
}

fn render_cube(
    data_generator: &world_noise::DataGenerator,
    data2d: &world_noise::Data2D,
    pos: Vec3,
    size: f32,
) -> Cube {
    let data_color = data_generator.get_data_color(data2d, pos.x, pos.z, pos.y);
    Cube {
        pos: data_color.pos_jittered,
        size: size * 1.175,
        color: data_color.color,
    }
}
