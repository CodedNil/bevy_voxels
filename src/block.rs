use crate::world_noise;
use crate::{chunks::CHUNK_SIZE, subdivision::Chunk};
use bevy::{
    pbr::wireframe::WireframeConfig,
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
};
use block_mesh::ilattice::glam::Vec3A;
use block_mesh::ndshape::{ConstShape, ConstShape3u32};
use block_mesh::{
    greedy_quads, GreedyQuadsBuffer, MergeVoxel, Voxel, VoxelVisibility, RIGHT_HANDED_Y_UP_CONFIG,
};

#[derive(Clone, Copy, Eq, PartialEq)]
struct BoolVoxel(bool);

const EMPTY: BoolVoxel = BoolVoxel(false);

impl Voxel for BoolVoxel {
    fn get_visibility(&self) -> VoxelVisibility {
        if *self == EMPTY {
            VoxelVisibility::Empty
        } else {
            VoxelVisibility::Opaque
        }
    }
}

impl MergeVoxel for BoolVoxel {
    type MergeValue = Self;
    type MergeValueFacingNeighbour = Self;

    fn merge_value(&self) -> Self::MergeValue {
        *self
    }

    fn merge_value_facing_neighbour(&self) -> Self::MergeValueFacingNeighbour {
        *self
    }
}

const CHUNK_SIZE_BIG: u32 = CHUNK_SIZE as u32 + 1;
const VOXEL_SCALE: f32 = 0.25;
const CHUNK_SIZE_SMALL: u32 = (CHUNK_SIZE_BIG as f32 / VOXEL_SCALE) as u32;
type ChunkShape = ConstShape3u32<CHUNK_SIZE_SMALL, CHUNK_SIZE_SMALL, CHUNK_SIZE_SMALL>;

pub fn setup(
    commands: Commands,
    materials: ResMut<Assets<StandardMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
    mut wireframe_config: ResMut<WireframeConfig>,
) {
    wireframe_config.global = true;
}

pub fn render_chunk(
    data_generator: &world_noise::DataGenerator,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    chunk_pos: Vec3,
) -> Chunk {
    let offset = CHUNK_SIZE_BIG as f32 / 2.0;

    let mut samples = [EMPTY; ChunkShape::SIZE as usize];
    for i in 0..(ChunkShape::SIZE) {
        let [x, y, z] = ChunkShape::delinearize(i);
        let pos = Vec3A::new(
            (x as f32 * VOXEL_SCALE as f32) - offset + chunk_pos.x,
            (y as f32 * VOXEL_SCALE as f32) - offset + chunk_pos.y,
            (z as f32 * VOXEL_SCALE as f32) - offset + chunk_pos.z,
        );

        let data2d = data_generator.get_data_2d(pos.x, pos.z);
        let is_inside = data_generator.get_data_3d(&data2d, pos.x, pos.z, pos.y);
        samples[i as usize] = BoolVoxel(!is_inside);
    }

    let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;

    let mut buffer = GreedyQuadsBuffer::new(samples.len());
    greedy_quads(
        &samples,
        &ChunkShape {},
        [0; 3],
        [CHUNK_SIZE_SMALL - 1; 3],
        &faces,
        &mut buffer,
    );
    let num_indices = buffer.quads.num_quads() * 6;
    let num_vertices = buffer.quads.num_quads() * 4;
    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    for (group, face) in buffer.quads.groups.into_iter().zip(faces.into_iter()) {
        for quad in group {
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            positions.extend_from_slice(&face.quad_mesh_positions(&quad, VOXEL_SCALE));
            normals.extend_from_slice(&face.quad_mesh_normals());
        }
    }

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );
    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );
    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        VertexAttributeValues::Float32x2(vec![[0.0; 2]; num_vertices]),
    );
    render_mesh.set_indices(Some(Indices::U32(indices.clone())));

    let mut material = StandardMaterial::from(Color::rgb(0.0, 0.0, 0.0));
    material.perceptual_roughness = 0.9;

    commands.spawn(PbrBundle {
        mesh: meshes.add(render_mesh),
        material: materials.add(material),
        transform: Transform::from_translation(Vec3::new(
            chunk_pos.x - offset,
            chunk_pos.y - offset,
            chunk_pos.z - offset,
        )),
        ..Default::default()
    });

    Chunk {
        n_cubes: 0,
        n_triangles: indices.len() / 3,
    }
}

fn jitter_position(pos: [f32; 3], data_generator: &world_noise::DataGenerator) -> [f32; 3] {
    let (x, y, z) = (pos[0], pos[1], pos[2]);
    [
        x + (data_generator.get_noise2d(z, y) * 0.5),
        y,
        z + (data_generator.get_noise2d(x, y) * 0.5),
    ]
}
