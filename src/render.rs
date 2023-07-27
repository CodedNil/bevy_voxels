use crate::subdivision::Cube;
use bevy::prelude::*;
use bevy::render::{mesh::Indices, render_resource::PrimitiveTopology};

const FACES: [[usize; 6]; 6] = [
    [2, 1, 0, 3, 1, 2], // Front face
    [4, 5, 6, 6, 5, 7], // Back face
    [2, 0, 4, 4, 6, 2], // Top face
    [1, 3, 5, 3, 7, 5], // Bottom face
    [0, 1, 5, 5, 4, 0], // Left face
    [3, 2, 6, 6, 7, 3], // Right face
];
const FACE_NORMALS: [[f32; 3]; 6] = [
    [0.0, 0.0, 1.0],  // Front face
    [0.0, 0.0, -1.0], // Back face
    [0.0, 1.0, 0.0],  // Top face
    [0.0, -1.0, 0.0], // Bottom face
    [1.0, 0.0, 0.0],  // Left face
    [-1.0, 0.0, 0.0], // Right face
];

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::similar_names)]
pub fn cubes_mesh(cubes: &Vec<Cube>, chunk_pos: (f32, f32, f32)) -> (Mesh, usize) {
    let (chunk_x, chunk_z, chunk_y) = chunk_pos;

    let n_cubes = cubes.len();
    let n_triangles = n_cubes * 12;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(n_cubes * 36);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n_cubes * 36);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(n_cubes * 36);
    let mut indices: Vec<u32> = Vec::with_capacity(n_cubes * 36);

    for (i, cube) in cubes.iter().enumerate() {
        let half_size = cube.size / 2.0;

        let (corner_x, corner_z, corner_y) = cube.pos;
        let (real_x, real_z, real_y) = (corner_x - chunk_x, corner_z - chunk_z, corner_y - chunk_y);
        let (real_x_minus, real_x_plus, real_z_minus, real_z_plus, real_y_minus, real_y_plus) = (
            real_x - half_size,
            real_x + half_size,
            real_z - half_size,
            real_z + half_size,
            real_y - half_size,
            real_y + half_size,
        );
        let corners = [
            [real_x_plus, real_y_plus, real_z_plus],
            [real_x_plus, real_y_minus, real_z_plus],
            [real_x_minus, real_y_plus, real_z_plus],
            [real_x_minus, real_y_minus, real_z_plus],
            [real_x_plus, real_y_plus, real_z_minus],
            [real_x_plus, real_y_minus, real_z_minus],
            [real_x_minus, real_y_plus, real_z_minus],
            [real_x_minus, real_y_minus, real_z_minus],
        ];

        let color = [cube.color.0, cube.color.1, cube.color.2, 1.0];

        // Loop over each face of the cube
        let base_index = (i * 36) as u32;
        for face_index in 0..6 {
            let current_face = FACES[face_index];
            let current_normal = FACE_NORMALS[face_index];

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

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    render_mesh.set_indices(Some(Indices::U32(indices)));

    (render_mesh, n_triangles)
}
