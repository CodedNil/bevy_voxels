use bevy::prelude::*;

const LARGEST_VOXEL_SIZE: f32 = 4.0;
const SMALLEST_VOXEL_SIZE: f32 = 0.25;

pub fn chunk_render(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    x: f32,
    y: f32,
) {
    subdivide_voxel(
        commands,
        meshes,
        materials,
        Vec3::new(x, 0.0, y),
        LARGEST_VOXEL_SIZE,
    );
}

fn subdivide_voxel(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    pos3d: Vec3,
    voxel_size: f32,
) {
    // If voxel is too small, render it
    if voxel_size <= SMALLEST_VOXEL_SIZE {
        // Temporary inside3d, if y is less than x
        let inside3d = pos3d.y < -1.3;
        // If outside the room, render
        if !inside3d {
            render_voxel(commands, meshes, materials, pos3d, voxel_size);
        }
        return;
    }
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
                    // Temporary inside3d, if y is less than x
                    let inside3d = y < -1.3;
                    if inside3d {
                        n_air_voxels += 1;
                    }
                }
            }
        }
        // If air voxels in threshold range, render it
        if n_air_voxels <= max_air_voxels {
            render_voxel(commands, meshes, materials, pos3d, voxel_size);
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
                subdivide_voxel(commands, meshes, materials, pos3d, half_voxel_size);
            }
        }
    }
}

fn render_voxel(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    pos3d: Vec3,
    voxel_size: f32,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: voxel_size })),
        material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
        transform: Transform::from_translation(pos3d),
        ..Default::default()
    });
}
