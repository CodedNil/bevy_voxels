use bevy::prelude::*;
use noise::{NoiseFn, OpenSimplex};

use bevy::render::{
    mesh::{Indices, MeshVertexAttribute, VertexAttributeValues},
    render_resource::{PrimitiveTopology, VertexFormat},
};

const LARGEST_VOXEL_SIZE: f32 = 4.0;
const SMALLEST_VOXEL_SIZE: f32 = 0.25;

struct Voxel {
    pos: Vec3,
    size: f32,
    color: Color,
}

pub fn chunk_render(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    x: f32,
    y: f32,
    chunk_size: f32,
) {
    // Subdivide the voxel and store the result in the voxels vector
    let mut voxels: Vec<Voxel> = Vec::new();
    subdivide_voxel(&mut voxels, Vec3::new(x, 0.0, y), chunk_size);
    let voxels = voxels;

    // Gather triangles for rendering
    let n = voxels.len();
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(n * 36);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n * 36);
    let mut indices: Vec<u32> = Vec::with_capacity(n * 36);

    let faces = [
        [2, 1, 0, 3, 1, 2], // Front face
        [4, 5, 6, 6, 5, 7], // Back face
        [2, 0, 4, 4, 6, 2], // Top face
        [1, 3, 5, 5, 7, 3], // Bottom face
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

        let current_index = (i * 36) as u32;
        for k in 0..6 {
            let fk = faces[k];
            let normal = face_normals[k];

            for j in 0..6 {
                let idx = fk[j];

                indices.push(current_index + (k * 6 + j) as u32);
                positions.push([corners[idx].x, corners[idx].y, corners[idx].z]);
                normals.push(normal);
            }
        }
    }

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    render_mesh.insert_attribute(
        MeshVertexAttribute::new("Vertex_Position", 0, VertexFormat::Float32x3),
        VertexAttributeValues::Float32x3(positions),
    );
    render_mesh.insert_attribute(
        MeshVertexAttribute::new("Vertex_Normal", 1, VertexFormat::Float32x3),
        VertexAttributeValues::Float32x3(normals),
    );
    render_mesh.set_indices(Some(Indices::U32(indices)));

    commands.spawn(PbrBundle {
        mesh: meshes.add(render_mesh),
        material: materials.add(StandardMaterial::from(Color::rgb(1.0, 0.0, 0.0))),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });
}

fn is_inside3d(pos3d: Vec3) -> bool {
    let simplex = OpenSimplex::new(42);
    let noise_value = simplex.get([pos3d.x as f64 / 10.0, pos3d.z as f64 / 10.0]) * 5.0;
    pos3d.y > noise_value as f32
}

fn subdivide_voxel(voxels: &mut Vec<Voxel>, pos3d: Vec3, voxel_size: f32) {
    let half_voxel_size = voxel_size / 2.0;

    if voxel_size <= LARGEST_VOXEL_SIZE {
        // Calculate how much of the voxel is air
        let mut n_air_voxels = 0;
        // Smaller voxels have higher threshold for air, so less small voxels made
        let max_air_voxels: i32 = if (voxel_size - 0.5).abs() < f32::EPSILON {
            4
        } else if (voxel_size - 1.0).abs() < f32::EPSILON {
            2
        } else {
            0
        };

        for x in [pos3d.x - half_voxel_size, pos3d.x + half_voxel_size] {
            for z in [pos3d.z - half_voxel_size, pos3d.z + half_voxel_size] {
                for y in [pos3d.y - half_voxel_size, pos3d.y + half_voxel_size] {
                    if is_inside3d(Vec3::new(x, y, z)) {
                        n_air_voxels += 1;
                    }
                }
            }
        }
        // If air voxels in threshold range, render it
        if n_air_voxels <= max_air_voxels {
            render_voxel(voxels, pos3d, voxel_size);
            return;
        }
        // If fully air, skip
        if n_air_voxels == 8 {
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
                    if !is_inside3d(pos2) {
                        render_voxel(voxels, pos2, half_voxel_size);
                    }
                } else {
                    subdivide_voxel(voxels, pos2, half_voxel_size);
                }
            }
        }
    }
}

fn render_voxel(voxels: &mut Vec<Voxel>, pos3d: Vec3, voxel_size: f32) {
    voxels.push(Voxel {
        pos: pos3d,
        size: voxel_size,
        color: Color::rgb(1.0, 0.0, 0.0),
    });
}
