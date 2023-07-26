use crate::render;
use crate::world_noise;
use bevy::prelude::*;

const LARGEST_VOXEL_SIZE: f32 = 2.0;
const SMALLEST_VOXEL_SIZE: f32 = 0.25;

pub struct Voxel {
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
    // Subdivide the voxel and store the result in the voxels vector
    let mut voxels: Vec<Voxel> = Vec::new();
    subdivide_voxel(&mut voxels, data_generator, pos, chunk_size);
    let voxels = voxels;

    // Render the mesh
    let (render_mesh, cubes, triangles) = render::voxels(data_generator, &voxels, pos);
    commands.spawn(PbrBundle {
        mesh: meshes.add(render_mesh),
        material: materials.add(StandardMaterial::from(Color::rgb(1.0, 1.0, 1.0))),
        transform: Transform::from_xyz(pos.x, pos.y, pos.z),
        ..Default::default()
    });

    Chunk { cubes, triangles }
}

fn subdivide_voxel(
    voxels: &mut Vec<Voxel>,
    data_generator: &world_noise::DataGenerator,
    pos3d: Vec3,
    voxel_size: f32,
) {
    let half_voxel_size = voxel_size / 2.0;

    if voxel_size <= LARGEST_VOXEL_SIZE {
        // Calculate how much of the voxel is air
        let mut n_air_voxels = 0;
        // Smaller voxels have higher threshold for air, so less small voxels made
        let max_air_voxels: i32 = match voxel_size {
            x if (x - 0.5).abs() < f32::EPSILON => 2,
            x if (x - 1.0).abs() < f32::EPSILON => 1,
            _ => 0,
        };

        for x in [pos3d.x - half_voxel_size, pos3d.x + half_voxel_size] {
            for z in [pos3d.z - half_voxel_size, pos3d.z + half_voxel_size] {
                let data2d = data_generator.get_data_2d(x, z);
                for y in [pos3d.y - half_voxel_size, pos3d.y + half_voxel_size] {
                    let inside3d = data_generator.get_data_3d(&data2d, x, z, y);
                    if inside3d {
                        n_air_voxels += 1;
                    }
                }
            }
        }
        // If fully air, skip
        if n_air_voxels == 8 {
            return;
        }
        // If air voxels in threshold range, render it
        if n_air_voxels <= max_air_voxels {
            let data2d = data_generator.get_data_2d(pos3d.x, pos3d.z);
            render_voxel(voxels, data_generator, &data2d, pos3d, voxel_size);
            return;
        }
    }
    // Otherwise, subdivide it into 8 smaller voxels
    for x in [-half_voxel_size, half_voxel_size] {
        for z in [-half_voxel_size, half_voxel_size] {
            for y in [-half_voxel_size, half_voxel_size] {
                let pos2 = pos3d + Vec3::new(x, y, z) * 0.5;
                if half_voxel_size < SMALLEST_VOXEL_SIZE {
                    let data2d = data_generator.get_data_2d(x, z);
                    let inside3d = data_generator.get_data_3d(&data2d, x, z, y);
                    if !inside3d {
                        render_voxel(voxels, data_generator, &data2d, pos2, half_voxel_size);
                    }
                } else {
                    subdivide_voxel(voxels, data_generator, pos2, half_voxel_size);
                }
            }
        }
    }
}

fn render_voxel(
    voxels: &mut Vec<Voxel>,
    data_generator: &world_noise::DataGenerator,
    data2d: &world_noise::Data2D,
    pos: Vec3,
    size: f32,
) {
    let data_color = data_generator.get_data_color(data2d, pos.x, pos.z, pos.y);

    // Add voxel to list
    voxels.push(Voxel {
        pos,
        size,
        color: data_color.color,
    });
}
