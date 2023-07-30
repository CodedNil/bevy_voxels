use crate::chunks::render::{CubeFace, Face};
use bevy::prelude::*;
use rayon::prelude::*;
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

struct FaceRaycast {
    index: usize,
    face_index: usize,
    vertices: [Vec3; 4],
    tris: [[Vec3; 3]; 2],
}

struct Ray {
    origin: Vec3,
    direction: Vec3,
}

pub fn perform_raycasts(cube_faces: &[CubeFace], min_pos: Vec3, max_pos: Vec3) -> Vec<CubeFace> {
    let raycast_data = get_raycast_data(min_pos, max_pos);

    let mut hit_faces: [HashSet<usize>; 6] = Default::default();

    let hit_faces_temp: Vec<[HashSet<usize>; 6]> = raycast_data
        .par_iter()
        .map(|(cube_face_indices, origin)| {
            let mut hit_faces_temp: [HashSet<usize>; 6] = Default::default();

            // Get all faces to cast against
            let total_faces: Vec<FaceRaycast> = cube_face_indices
                .par_iter()
                .map(|&cube_face_index| {
                    cube_faces[cube_face_index.as_usize()]
                        .faces
                        .par_iter()
                        .enumerate()
                        .map(|(index, face)| FaceRaycast {
                            index,
                            face_index: cube_face_index.as_usize(),
                            vertices: face.vertices,
                            tris: face.tris,
                        })
                        .collect::<Vec<_>>()
                })
                .flatten()
                .collect();

            total_faces
                .par_iter()
                .map(|face| {
                    let mut local_hit_faces: [HashSet<usize>; 6] = Default::default();
                    for vertex in &face.vertices {
                        let origin = *origin + *vertex;
                        let direction = (*vertex - origin).normalize();
                        let ray = Ray { origin, direction };
                        if let Some(hit_face) = raycast_mesh(&ray, &total_faces) {
                            local_hit_faces[hit_face.face_index].insert(hit_face.index);
                        }
                    }
                    local_hit_faces
                })
                .collect::<Vec<_>>()
                .into_iter()
                .for_each(|local_hit_faces| {
                    for i in 0..6 {
                        hit_faces_temp[i].extend(&local_hit_faces[i]);
                    }
                });

            hit_faces_temp
        })
        .collect();

    // Merge the temporary hit_faces into the main hit_faces
    for temp in hit_faces_temp {
        for (i, set) in temp.iter().enumerate() {
            hit_faces[i].extend(set);
        }
    }

    let new_cube_faces: Vec<CubeFace> = (0..6)
        .into_par_iter()
        .map(|i| {
            let cube_face = &cube_faces[i];
            let new_faces: Vec<Face> = hit_faces[i]
                .iter()
                .map(|&face_index| cube_face.faces[face_index].clone())
                .collect();

            CubeFace {
                faces: new_faces,
                normal: cube_face.normal,
            }
        })
        .collect();

    new_cube_faces
}

/// Perform a raycast against the mesh faces
fn raycast_mesh<'a>(ray: &'a Ray, faces: &'a Vec<FaceRaycast>) -> Option<&'a FaceRaycast> {
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

fn get_raycast_data(min_pos: Vec3, max_pos: Vec3) -> [(Vec<FaceIndex>, Vec3); 26] {
    let max_size = (max_pos - min_pos).max_element();
    let shape_center = (max_pos + min_pos) / 2.0;
    let (off_x, off_y, off_z) = (
        shape_center.x + max_size * 1.5,
        shape_center.y + max_size * 1.5,
        shape_center.z + max_size * 1.5,
    );

    [
        // Each of the 6 directions
        (vec![FaceIndex::Front], Vec3::new(0.0, 0.0, off_z)),
        (vec![FaceIndex::Back], Vec3::new(0.0, 0.0, -off_z)),
        (vec![FaceIndex::Top], Vec3::new(0.0, off_y, 0.0)),
        (vec![FaceIndex::Bottom], Vec3::new(0.0, -off_y, 0.0)),
        (vec![FaceIndex::Left], Vec3::new(off_x, 0.0, 0.0)),
        (vec![FaceIndex::Right], Vec3::new(-off_x, 0.0, 0.0)),
        // The 12 2d corners
        (
            vec![FaceIndex::Left, FaceIndex::Front],
            Vec3::new(off_x, 0.0, off_z),
        ),
        (
            vec![FaceIndex::Left, FaceIndex::Back],
            Vec3::new(off_x, 0.0, -off_z),
        ),
        (
            vec![FaceIndex::Right, FaceIndex::Front],
            Vec3::new(-off_x, 0.0, off_z),
        ),
        (
            vec![FaceIndex::Right, FaceIndex::Back],
            Vec3::new(-off_x, 0.0, -off_z),
        ),
        (
            vec![FaceIndex::Top, FaceIndex::Front],
            Vec3::new(0.0, off_y, off_z),
        ),
        (
            vec![FaceIndex::Top, FaceIndex::Back],
            Vec3::new(0.0, off_y, -off_z),
        ),
        (
            vec![FaceIndex::Top, FaceIndex::Left],
            Vec3::new(-off_x, off_y, 0.0),
        ),
        (
            vec![FaceIndex::Top, FaceIndex::Right],
            Vec3::new(-off_x, off_y, 0.0),
        ),
        (
            vec![FaceIndex::Bottom, FaceIndex::Front],
            Vec3::new(0.0, -off_y, off_z),
        ),
        (
            vec![FaceIndex::Bottom, FaceIndex::Back],
            Vec3::new(0.0, -off_y, -off_z),
        ),
        (
            vec![FaceIndex::Bottom, FaceIndex::Left],
            Vec3::new(-off_x, -off_y, 0.0),
        ),
        (
            vec![FaceIndex::Bottom, FaceIndex::Right],
            Vec3::new(-off_x, -off_y, 0.0),
        ),
        // The 8 3dr corners
        (
            vec![FaceIndex::Left, FaceIndex::Top, FaceIndex::Front],
            Vec3::new(off_x, off_y, off_z),
        ),
        (
            vec![FaceIndex::Right, FaceIndex::Bottom, FaceIndex::Back],
            Vec3::new(-off_x, -off_y, -off_z),
        ),
        (
            vec![FaceIndex::Right, FaceIndex::Top, FaceIndex::Front],
            Vec3::new(-off_x, off_y, off_z),
        ),
        (
            vec![FaceIndex::Left, FaceIndex::Bottom, FaceIndex::Front],
            Vec3::new(off_x, -off_y, off_z),
        ),
        (
            vec![FaceIndex::Left, FaceIndex::Top, FaceIndex::Back],
            Vec3::new(off_x, off_y, -off_z),
        ),
        (
            vec![FaceIndex::Right, FaceIndex::Bottom, FaceIndex::Front],
            Vec3::new(-off_x, -off_y, off_z),
        ),
        (
            vec![FaceIndex::Left, FaceIndex::Bottom, FaceIndex::Back],
            Vec3::new(off_x, -off_y, -off_z),
        ),
        (
            vec![FaceIndex::Right, FaceIndex::Top, FaceIndex::Back],
            Vec3::new(-off_x, off_y, -off_z),
        ),
    ]
}
