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
    tris: [[Vec3; 3]; 2],
    color: [f32; 4],
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

pub fn cubes_mesh(cubes: &Vec<Cube>, chunk_pos: (f32, f32, f32)) -> (Mesh, usize) {
    let (cube_faces, min_pos, max_pos) = generate_cube_faces(cubes, chunk_pos);
    let cube_faces = perform_raycasts(&cube_faces, min_pos, max_pos);
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
            cube_faces[face_index].faces.push(Face {
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

fn perform_raycasts(cube_faces: &Vec<CubeFace>, min_pos: Vec3, max_pos: Vec3) -> Vec<CubeFace> {
    // Get max size of the shape and shape center
    let max_size: f32 = (max_pos - min_pos).max_element();
    let shape_center = (max_pos + min_pos) / 2.0;

    // Perform raycasts against the mesh each direction
    let ray_step = 0.1;

    let left_faces = &cube_faces[4].faces;
    let mut new_left_faces: Vec<Face> = Vec::new();
    let right_faces = &cube_faces[5].faces;
    let mut new_right_faces: Vec<Face> = Vec::new();

    let mut y = shape_center.y - max_size / 2.0;
    while y < shape_center.y + max_size / 2.0 {
        let mut z = shape_center.z - max_size / 2.0;
        while z < shape_center.z + max_size / 2.0 {
            let left_ray = Ray {
                origin: Vec3::new(max_pos.x + 1.0, y, z),
                direction: Vec3::new(-1.0, 0.0, 0.0),
            };
            if let Some(hit_face) = raycast_mesh(&left_ray, left_faces).as_mut() {
                let mut hit_face_clone = hit_face.clone();
                hit_face_clone.color = [1.0, 0.0, 0.0, 1.0];
                new_left_faces.push(hit_face_clone);
            }

            let right_ray = Ray {
                origin: Vec3::new(min_pos.x - 1.0, y, z),
                direction: Vec3::new(1.0, 0.0, 0.0),
            };
            if let Some(hit_face) = raycast_mesh(&right_ray, right_faces).as_mut() {
                let mut hit_face_clone = hit_face.clone();
                hit_face_clone.color = [1.0, 0.0, 0.0, 1.0];
                new_right_faces.push(hit_face_clone);
            }

            z += ray_step;
        }
        y += ray_step;
    }

    // Replace faces with new right faces
    let mut new_cube_faces = cube_faces.clone();
    new_cube_faces[4].faces = new_left_faces;
    new_cube_faces[5].faces = new_right_faces;

    new_cube_faces
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

/// Perform a raycast against the mesh faces
fn raycast_mesh<'a>(ray: &Ray, faces: &'a Vec<Face>) -> Option<&'a Face> {
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

    hit_face
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
