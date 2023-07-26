use crate::world_noise;
use bevy::prelude::*;
use bevy::render::{mesh::Indices, render_resource::PrimitiveTopology};

const LARGEST_VOXEL_SIZE: f32 = 4.0;
const SMALLEST_VOXEL_SIZE: f32 = 0.25;

struct Voxel {
    pos: Vec3,
    size: f32,
    color: Color,
}

pub struct Chunk {
    pub cubes: i32,
    pub triangles: i32,
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

        let px = voxel.pos.x - pos.x;
        let py = voxel.pos.y - pos.y;
        let pz = voxel.pos.z - pos.z;
        let corners = [
            [px + half_size, py + half_size, pz + half_size],
            [px + half_size, py - half_size, pz + half_size],
            [px - half_size, py + half_size, pz + half_size],
            [px - half_size, py - half_size, pz + half_size],
            [px + half_size, py + half_size, pz - half_size],
            [px + half_size, py - half_size, pz - half_size],
            [px - half_size, py + half_size, pz - half_size],
            [px - half_size, py - half_size, pz - half_size],
        ];

        let color = [
            voxel.color.r(),
            voxel.color.g(),
            voxel.color.b(),
            voxel.color.a(),
        ];

        // Loop over each face of the cube
        let base_index = (i * 36) as u32;
        for face_index in 0..6 {
            let current_face = faces[face_index];
            let current_normal = face_normals[face_index];

            // Loop over each vertex of the face
            for vertex_index in 0..6 {
                // Calculate the index for the current vertex
                let current_index = base_index + (face_index * 6 + vertex_index) as u32;

                // Push the current index, position, normal, and color to their respective arrays
                indices.push(current_index);
                positions.push(corners[current_face[vertex_index]]);
                normals.push(current_normal);
                colors.push(color);
            }
        }
    }
    let triangles = indices.len() as i32 / 3;

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    render_mesh.set_indices(Some(Indices::U32(indices)));

    commands.spawn(PbrBundle {
        mesh: meshes.add(render_mesh),
        material: materials.add(StandardMaterial::from(Color::rgb(1.0, 1.0, 1.0))),
        transform: Transform::from_xyz(pos.x, pos.y, pos.z),
        ..Default::default()
    });

    Chunk {
        cubes: n as i32,
        triangles,
    }
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
            render_voxel(voxels, pos3d, voxel_size);
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
