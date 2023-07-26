use crate::subdivision::Voxel;
use crate::world_noise;
use bevy::prelude::*;
use bevy::render::{mesh::Indices, render_resource::PrimitiveTopology};

#[allow(clippy::cast_possible_truncation)]
pub fn voxels(
    data_generator: &world_noise::DataGenerator,
    voxels: &Vec<Voxel>,
    pos: Vec3,
) -> (Mesh, usize, usize) {
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

                // Jitter the position with noise
                let pos = corners[current_face[vertex_index]];
                let pos_jittered = [
                    pos[0] + data_generator.get_noise2d(pos[2], pos[1]) * 0.5,
                    pos[1],
                    pos[2] + data_generator.get_noise2d(pos[0], pos[1]) * 0.5,
                ];

                // Push the current index, position, normal, and color to their respective arrays
                indices.push(current_index);
                positions.push(pos_jittered);
                normals.push(current_normal);
                colors.push(color);
            }
        }
    }
    let triangles = indices.len() / 3;

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    render_mesh.set_indices(Some(Indices::U32(indices)));

    (render_mesh, n, triangles)
}
