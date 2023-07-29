use crate::subdivision::Cube;
use bevy::prelude::*;
use bevy::render::{mesh::Indices, render_resource::PrimitiveTopology};
use std::collections::HashSet;

#[derive(Copy, Clone)]
enum FaceIndex {
    Front = 0,
    Back = 1,
    Top = 2,
    Bottom = 3,
    Left = 4,
    Right = 5,
}
impl FaceIndex {
    fn as_usize(self) -> usize {
        self as usize
    }
}

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
struct CubeFace {
    faces: Vec<Face>,
    normal: Vec3,
}

#[derive(Clone)]
struct Face {
    vertices: [Vec3; 4],
    tris: [[Vec3; 3]; 2],
    color: [f32; 4],
}

struct FaceRaycast {
    index: usize,
    face_index: usize,
    vertices: [Vec3; 4],
    tris: [[Vec3; 3]; 2],
}

struct MeshData {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

struct Ray {
    origin: Vec3,
    direction: Vec3,
}

pub struct Ray2 {
    pub origin: Vec3,
    pub direction: Vec3,
    pub length: f32,
}

pub fn cubes_mesh(cubes: &Vec<Cube>, chunk_pos: (f32, f32, f32)) -> (Mesh, usize, Vec<Ray2>) {
    let (cube_faces, min_pos, max_pos) = generate_cube_faces(cubes, chunk_pos);
    let (cube_faces, rays) = perform_raycasts(&cube_faces, min_pos, max_pos);
    let mesh_data = generate_mesh_data(&cube_faces, cubes.len());
    // let rays = Vec::new();

    let n_triangles = mesh_data.indices.len() / 3;

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals);
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, mesh_data.colors);
    render_mesh.set_indices(Some(Indices::U32(mesh_data.indices)));

    (render_mesh, n_triangles, rays)
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
fn generate_mesh_data(cube_faces: &Vec<CubeFace>, n_cubes: usize) -> MeshData {
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(n_cubes * 36);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n_cubes * 36);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(n_cubes * 36);
    let mut indices: Vec<u32> = Vec::with_capacity(n_cubes * 36);

    for cube_face in cube_faces {
        let normal: [f32; 3] = cube_face.normal.into();
        for current_face in &cube_face.faces {
            let base_index = indices.len() as u32;
            // Render out both tris
            for (vertex_index, vertex) in current_face.tris[0].iter().enumerate() {
                indices.push(base_index + vertex_index as u32);
                positions.push((*vertex).into());
                normals.push(normal);
                colors.push(current_face.color);
            }
            for (vertex_index, vertex) in current_face.tris[1].iter().enumerate() {
                indices.push(base_index + vertex_index as u32 + 3);
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

fn perform_raycasts(
    cube_faces: &[CubeFace],
    min_pos: Vec3,
    max_pos: Vec3,
) -> (Vec<CubeFace>, Vec<Ray2>) {
    let max_size = (max_pos - min_pos).max_element();
    let shape_center = (max_pos + min_pos) / 2.0;
    let off = max_size * 2.0;
    let (off_x, off_y, off_z) = (
        shape_center.x + off,
        shape_center.y + off,
        shape_center.z + off,
    );

    let raycast_data = [
        // Each of the 6 directions
        // (vec![FaceIndex::Front], Vec3::new(0.0, 0.0, off_z)),
        // (vec![FaceIndex::Back], Vec3::new(0.0, 0.0, -off_z)),
        // (vec![FaceIndex::Top], Vec3::new(0.0, off_y, 0.0)),
        // (vec![FaceIndex::Bottom], Vec3::new(0.0, -off_y, 0.0)),
        // (vec![FaceIndex::Left], Vec3::new(off_x, 0.0, 0.0)),
        // (vec![FaceIndex::Right], Vec3::new(-off_x, 0.0, 0.0)),
        // The 8 corners
        (
            vec![FaceIndex::Front, FaceIndex::Top, FaceIndex::Left],
            Vec3::new(off_x, off_y, off_z),
        ),
        // (
        //     vec![FaceIndex::Back, FaceIndex::Bottom, FaceIndex::Right],
        //     Vec3::new(-off_x, -off_y, -off_z),
        // ),
        // (
        //     vec![FaceIndex::Front, FaceIndex::Bottom, FaceIndex::Left],
        //     Vec3::new(off_x, -off_y, off_z),
        // ),
        // (
        //     vec![FaceIndex::Front, FaceIndex::Top, FaceIndex::Right],
        //     Vec3::new(off_x, off_y, off_z),
        // ),
        // (
        //     vec![FaceIndex::Back, FaceIndex::Top, FaceIndex::Left],
        //     Vec3::new(-off_x, off_y, -off_z),
        // ),
        // (
        //     vec![FaceIndex::Back, FaceIndex::Bottom, FaceIndex::Left],
        //     Vec3::new(-off_x, -off_y, -off_z),
        // ),
        // (
        //     vec![FaceIndex::Front, FaceIndex::Bottom, FaceIndex::Right],
        //     Vec3::new(off_x, -off_y, off_z),
        // ),
        // (
        //     vec![FaceIndex::Back, FaceIndex::Top, FaceIndex::Right],
        //     Vec3::new(-off_x, off_y, -off_z),
        // ),
    ];

    let mut hit_faces: [HashSet<usize>; 6] = Default::default();
    let mut rays_used: Vec<Ray2> = Vec::new();
    for (cube_face_indices, origin) in raycast_data {
        // Get all faces to cast against
        let mut total_faces: Vec<FaceRaycast> = Vec::new();
        for cube_face_index in cube_face_indices {
            for (index, face) in cube_faces[cube_face_index.as_usize()].faces.iter().enumerate() {
                total_faces.push(FaceRaycast {
                    index,
                    face_index: cube_face_index.as_usize(),
                    vertices: face.vertices,
                    tris: face.tris,
                });
            }
        }
        let total_faces = total_faces;

        for face in &total_faces {
            for vertex in &face.vertices {
                let origin = origin + *vertex;
                let direction = (*vertex - origin).normalize();
                let ray = Ray { origin, direction };
                if let (Some(hit_face), Some(length)) = raycast_mesh(&ray, &total_faces) {
                    hit_faces[hit_face.face_index].insert(hit_face.index);
                    rays_used.push(Ray2 {
                        origin: ray.origin,
                        direction: ray.direction,
                        length,
                    });
                }
            }
        }
    }
    let hit_faces = hit_faces;

    let new_cube_faces: Vec<CubeFace> = hit_faces
        .iter()
        .enumerate()
        .map(|(i, face_indices)| {
            let cube_face = &cube_faces[i];
            let new_faces: Vec<Face> = face_indices
                .iter()
                .map(|&face_index| cube_face.faces[face_index].clone())
                .collect();

            CubeFace {
                faces: new_faces,
                normal: cube_face.normal,
            }
        })
        .collect();

    (new_cube_faces, rays_used)
}

/// Perform a raycast against the mesh faces
fn raycast_mesh<'a>(
    ray: &'a Ray,
    faces: &'a Vec<FaceRaycast>,
) -> (Option<&'a FaceRaycast>, Option<f32>) {
    let mut closest_t = None;
    let mut hit_face = None;

    for face in faces {
        for triangle in face.tris {
            if let Some(t) = ray_triangle_intersect(ray, &triangle) {
                closest_t = match closest_t {
                    Some(current_t) if t < current_t => {
                        hit_face = Some(face);
                        Some(t)
                    }
                    None => {
                        hit_face = Some(face);
                        Some(t)
                    }
                    _ => closest_t,
                };
            }
        }
    }

    (hit_face, closest_t)
}

fn ray_triangle_intersect(ray: &Ray, triangle: &[Vec3; 3]) -> Option<f32> {
    let edge1 = triangle[1] - triangle[0];
    let edge2 = triangle[2] - triangle[0];

    let direction_cross_edge2 = ray.direction.cross(edge2);
    let determinant = edge1.dot(direction_cross_edge2);

    // Near zero determinant, no intersection.
    if determinant.abs() < 0.00001 {
        return None;
    }

    let inverse_determinant = 1.0 / determinant;
    let diff_origin_vertex = ray.origin - triangle[0];
    let u = inverse_determinant * diff_origin_vertex.dot(direction_cross_edge2);

    // Check the intersection point lies within the triangle.
    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let diff_origin_vertex_cross_edge1 = diff_origin_vertex.cross(edge1);
    let v = inverse_determinant * ray.direction.dot(diff_origin_vertex_cross_edge1);

    // Check the intersection point lies within the triangle.
    if v < 0.0 || (u + v) > 1.0 {
        return None;
    }

    let t = inverse_determinant * edge2.dot(diff_origin_vertex_cross_edge1);

    if t > 0.00001 {
        Some(t)
    } else {
        None
    }
}
