use crate::world_noise;
use bevy::prelude::*;
use bevy::render::{mesh::Indices, render_resource::PrimitiveTopology};
use noise::{NoiseFn, OpenSimplex};

const LARGEST_VOXEL_SIZE: f32 = 4.0;
const SMALLEST_VOXEL_SIZE: f32 = 0.125;

struct Voxel {
    pos: Vec3,
    size: f32,
    color: Color,
}

pub fn chunk_render(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    data_generator: &world_noise::DataGenerator,
    x: f32,
    z: f32,
    chunk_size: f32,
) {
    // Subdivide the voxel and store the result in the voxels vector
    let mut voxels: Vec<Voxel> = Vec::new();
    subdivide_voxel(
        &mut voxels,
        data_generator,
        Vec3::new(x, 0.0, z),
        chunk_size,
    );
    let voxels = voxels;

    // Gather triangles for rendering
    let n = voxels.len();
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(n * 36);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n * 36);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(n * 36);
    let mut indices: Vec<u32> = Vec::with_capacity(n * 36);

    let faces = [
        [2, 1, 0, 3, 1, 2], // Front face
        [4, 5, 6, 6, 5, 7], // Back face
        [2, 0, 4, 4, 6, 2], // Top face
        [1, 3, 5, 3, 7, 5], // Bottom face
        [0, 1, 5, 5, 4, 0], // Left face
        [3, 2, 6, 6, 7, 3], // Right face
    ];
    let face_normals = [
        [0.0, 0.0, 1.0],  // Front face
        [0.0, 0.0, -1.0], // Back face
        [0.0, 1.0, 0.0],  // Top face
        [0.0, -1.0, 0.0], // Bottom face
        [1.0, 0.0, 0.0],  // Left face
        [-1.0, 0.0, 0.0], // Right face
    ];

    for (i, voxel) in voxels.iter().enumerate() {
        let half_size = voxel.size / 2.0;

        let corners = [
            voxel.pos + Vec3::new(half_size, half_size, half_size),
            voxel.pos + Vec3::new(half_size, -half_size, half_size),
            voxel.pos + Vec3::new(-half_size, half_size, half_size),
            voxel.pos + Vec3::new(-half_size, -half_size, half_size),
            voxel.pos + Vec3::new(half_size, half_size, -half_size),
            voxel.pos + Vec3::new(half_size, -half_size, -half_size),
            voxel.pos + Vec3::new(-half_size, half_size, -half_size),
            voxel.pos + Vec3::new(-half_size, -half_size, -half_size),
        ];

        let color = [
            voxel.color.r(),
            voxel.color.g(),
            voxel.color.b(),
            voxel.color.a(),
        ];

        let current_index = (i * 36) as u32;
        for k in 0..6 {
            let fk = faces[k];
            let normal = face_normals[k];

            for j in 0..6 {
                let idx = fk[j];

                indices.push(current_index + (k * 6 + j) as u32);
                positions.push([corners[idx].x, corners[idx].y, corners[idx].z]);
                normals.push(normal);
                colors.push(color);
            }
        }
    }

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    render_mesh.set_indices(Some(Indices::U32(indices)));

    commands.spawn(PbrBundle {
        mesh: meshes.add(render_mesh),
        material: materials.add(StandardMaterial::from(Color::rgb(1.0, 1.0, 1.0))),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });
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
            x if (x - 0.25).abs() < f32::EPSILON => 4,
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
            render_voxel(voxels, pos3d, voxel_size);
            return;
        }
    }
    // Otherwise, subdivide it into 8 smaller voxels
    for x in [-half_voxel_size, half_voxel_size] {
        for z in [-half_voxel_size, half_voxel_size] {
            for y in [-half_voxel_size, half_voxel_size] {
                let pos2 = pos3d + Vec3::new(x, y, z) * 0.5;
                // If voxel is too small, render it
                if half_voxel_size <= SMALLEST_VOXEL_SIZE {
                    let data2d = data_generator.get_data_2d(x, z);
                    let inside3d = data_generator.get_data_3d(&data2d, x, z, y);
                    if !inside3d {
                        render_voxel(voxels, pos2, half_voxel_size);
                    }
                } else {
                    subdivide_voxel(voxels, data_generator, pos2, half_voxel_size);
                }
            }
        }
    }
}

fn render_voxel(voxels: &mut Vec<Voxel>, pos: Vec3, size: f32) {
    // Get color from height
    let height = (pos.y + 10.0) / 20.0;
    let color = Color::rgb(height, height, height);

    // Add voxel to list
    voxels.push(Voxel { pos, size, color });
}
