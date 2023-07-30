use crate::chunks::raycast;
use crate::chunks::subdivision::Cube;
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
const FACES_VERTICES: [[usize; 4]; 6] = [
    [0, 1, 2, 3], // Front face
    [4, 5, 6, 7], // Back face
    [0, 2, 4, 6], // Top face
    [1, 3, 5, 7], // Bottom face
    [0, 1, 4, 5], // Left face
    [2, 3, 6, 7], // Right face
];
const FACE_NORMALS: [Vec3; 6] = [
    Vec3::new(0.0, 0.0, 1.0),  // Front face
    Vec3::new(0.0, 0.0, -1.0), // Back face
    Vec3::new(0.0, 1.0, 0.0),  // Top face
    Vec3::new(0.0, -1.0, 0.0), // Bottom face
    Vec3::new(1.0, 0.0, 0.0),  // Left face
    Vec3::new(-1.0, 0.0, 0.0), // Right face
];

// Struct for a cubes face, contains faces within for all the smaller cubes
#[derive(Clone)]
pub struct CubeFace {
    pub faces: Vec<Face>,
    pub normal: Vec3,
}

#[derive(Clone)]
pub struct Face {
    pub vertices: [Vec3; 4],
    pub tris: [[Vec3; 3]; 2],
    pub color: [f32; 4],
}

struct MeshData {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

pub fn cubes_mesh(cubes: &Vec<Cube>, chunk_pos: (f32, f32, f32)) -> (Mesh, usize) {
    let (cube_faces, min_pos, max_pos) = generate_cube_faces(cubes, chunk_pos);
    let cube_faces = raycast::perform_raycasts(&cube_faces, min_pos, max_pos);
    let mesh_data = generate_mesh_data(&cube_faces, cubes.len());

    let n_triangles = mesh_data.indices.len() / 3;

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, mesh_data.colors);
    render_mesh.set_indices(Some(Indices::U32(mesh_data.indices)));

    (render_mesh, n_triangles)
}

#[allow(clippy::similar_names)]
fn generate_cube_faces(
    cubes: &Vec<Cube>,
    chunk_pos: (f32, f32, f32),
) -> (Vec<CubeFace>, Vec3, Vec3) {
    let (chunk_x, chunk_z, chunk_y) = chunk_pos;

    let n_cubes = cubes.len();

    let mut cube_faces: Vec<CubeFace> = Vec::with_capacity(6);
    for normal in FACE_NORMALS {
        cube_faces.push(CubeFace {
            faces: Vec::with_capacity(n_cubes),
            normal,
        });
    }

    // Initialize min and max positions with the first cube's position
    let mut min_pos = Vec3::new(cubes[0].pos.0, cubes[0].pos.1, cubes[0].pos.2);
    let mut max_pos = Vec3::new(cubes[0].pos.0, cubes[0].pos.1, cubes[0].pos.2);

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
            Vec3::new(real_x_plus, real_y_plus, real_z_plus),
            Vec3::new(real_x_plus, real_y_minus, real_z_plus),
            Vec3::new(real_x_minus, real_y_plus, real_z_plus),
            Vec3::new(real_x_minus, real_y_minus, real_z_plus),
            Vec3::new(real_x_plus, real_y_plus, real_z_minus),
            Vec3::new(real_x_plus, real_y_minus, real_z_minus),
            Vec3::new(real_x_minus, real_y_plus, real_z_minus),
            Vec3::new(real_x_minus, real_y_minus, real_z_minus),
        ];

        // Update min and max positions
        min_pos = min_pos.min(Vec3::new(real_x_minus, real_y_minus, real_z_minus));
        max_pos = max_pos.max(Vec3::new(real_x_plus, real_y_plus, real_z_plus));

        let color = [cube.color.0, cube.color.1, cube.color.2, 1.0];

        // Loop over each face of the cube
        for (face_index, current_face) in FACES.iter().enumerate() {
            let verts = FACES_VERTICES[face_index];
            let shift_amount = 0.01;
            let center =
                (corners[verts[0]] + corners[verts[1]] + corners[verts[2]] + corners[verts[3]])
                    / 4.0;

            let shifted_corners = [
                corners[verts[0]] + (center - corners[verts[0]]) * shift_amount,
                corners[verts[1]] + (center - corners[verts[1]]) * shift_amount,
                corners[verts[2]] + (center - corners[verts[2]]) * shift_amount,
                corners[verts[3]] + (center - corners[verts[3]]) * shift_amount,
            ];
            cube_faces[face_index].faces.push(Face {
                vertices: shifted_corners,
                tris: [
                    [
                        corners[current_face[0]],
                        corners[current_face[1]],
                        corners[current_face[2]],
                    ],
                    [
                        corners[current_face[3]],
                        corners[current_face[4]],
                        corners[current_face[5]],
                    ],
                ],
                color,
            });
        }
    }

    (cube_faces, min_pos, max_pos)
}

/// Generate the mesh data from the faces
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
fn generate_mesh_data(cube_faces: &Vec<CubeFace>, n_cubes: usize) -> MeshData {
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(n_cubes * 36);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n_cubes * 36);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(n_cubes * 36);
    let mut indices: Vec<u32> = Vec::with_capacity(n_cubes * 36);

    for cube_face in cube_faces {
        let normal: [f32; 3] = cube_face.normal.into();
        for current_face in &cube_face.faces {
            let base_index = indices.len() as u32;

            for (tri_index, vertex) in current_face
                .tris
                .iter()
                .flat_map(|tri| tri.iter())
                .enumerate()
            {
                let index = base_index + tri_index as u32;
                indices.push(index);
                positions.push((*vertex).into());
                normals.push(normal);
                colors.push(current_face.color);
            }
        }
    }

    MeshData {
        positions,
        normals,
        colors,
        indices,
    }
}
