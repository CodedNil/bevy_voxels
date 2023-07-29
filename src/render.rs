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

// Struct for a cubes face, contains faces within for all the smaller cubes
struct CubeFace {
    faces: Vec<Face>,
    normal: [f32; 3],
}

struct Face {
    tri_0: [[f32; 3]; 3],
    tri_1: [[f32; 3]; 3],
    color: [f32; 4],
}

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

    let mut cube_faces: Vec<CubeFace> = Vec::with_capacity(6);
    for normal in FACE_NORMALS {
        cube_faces.push(CubeFace {
            faces: Vec::with_capacity(n_cubes),
            normal,
        });
    }

    // Initialize min and max positions with the first cube's position
    let mut min_pos = [cubes[0].pos.0, cubes[0].pos.1, cubes[0].pos.2];
    let mut max_pos = [cubes[0].pos.0, cubes[0].pos.1, cubes[0].pos.2];

    for cube in cubes {
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

        // Update min and max positions
        min_pos = [
            min_pos[0].min(real_x_minus),
            min_pos[1].min(real_y_minus),
            min_pos[2].min(real_z_minus),
        ];
        max_pos = [
            max_pos[0].max(real_x_plus),
            max_pos[1].max(real_y_plus),
            max_pos[2].max(real_z_plus),
        ];

        let color = [cube.color.0, cube.color.1, cube.color.2, 1.0];

        // Loop over each face of the cube
        for (face_index, current_face) in FACES.iter().enumerate() {
            cube_faces[face_index].faces.push(Face {
                tri_0: [
                    corners[current_face[0]],
                    corners[current_face[1]],
                    corners[current_face[2]],
                ],
                tri_1: [
                    corners[current_face[3]],
                    corners[current_face[4]],
                    corners[current_face[5]],
                ],
                color,
            });
        }
    }

    // Now, generate the mesh data from the faces
    for cube_face in cube_faces {
        for current_face in cube_face.faces {
            let base_index = indices.len() as u32;
            // Render out both tris
            for (vertex_index, vertex) in current_face.tri_0.iter().enumerate() {
                indices.push(base_index + vertex_index as u32);
                positions.push(*vertex);
                normals.push(cube_face.normal);
                colors.push(current_face.color);
            }
            for (vertex_index, vertex) in current_face.tri_1.iter().enumerate() {
                indices.push(base_index + vertex_index as u32 + 3);
                positions.push(*vertex);
                normals.push(cube_face.normal);
                colors.push(current_face.color);
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
