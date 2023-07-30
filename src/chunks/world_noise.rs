use bevy::prelude::*;
use noise::{NoiseFn, OpenSimplex};
use std::f32::consts::PI;

const ROOM_SPACING: f32 = 150.0;

fn lerp(start: f32, end: f32, percentage: f32) -> f32 {
    start + percentage * (end - start)
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[derive(PartialEq)]
pub enum FloorMaterial {
    Stone,
    Sand,
    Moss,
    Dirt,
}

pub struct DataGenerator {
    pub world_noise: OpenSimplex,
}

pub struct Data2D {
    pub elevation: f32,
    pub smoothness: f32,
    pub temperature: f32,
    pub humidity: f32,
    pub lushness: f32,
    pub development: f32,
    pub rock_color: Vec3,
    pub room_position: [f32; 2],
    pub room_dist: f32,
    pub room_size: f32,
    pub corridor_width: f32,
    pub corridor_dist: f32,
    pub room_floor: f32,
    pub room_ceiling: f32,
    pub floor_material: FloorMaterial,
    pub floor_variance1: f32,
    pub floor_variance2: f32,
    pub floor_variance3: f32,
}

pub struct DataColor {
    pub color: Vec3,
    pub pos_jittered: Vec3,
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_lossless)]
impl DataGenerator {
    pub fn new() -> Self {
        DataGenerator {
            world_noise: OpenSimplex::new(4321),
        }
    }

    pub fn get_noise(&self, x: f32) -> f32 {
        self.world_noise.get([x as f64, 0.0]) as f32
    }
    pub fn get_noise2d(&self, x: f32, z: f32) -> f32 {
        self.world_noise.get([x as f64, z as f64]) as f32
    }
    pub fn get_world_noise(&self, offset: f64, scale: f64, x: f32) -> f32 {
        let val = self.world_noise.get([offset * 1000.0, x as f64 * scale]);

        ((1.0 + (val * 1.4)) * 0.5).clamp(0.0, 1.0) as f32
    }
    pub fn get_world_noise2d(&self, offset: f64, scale: f64, x: f32, z: f32) -> f32 {
        let val = self
            .world_noise
            .get([offset * 1000.0, x as f64 * scale, z as f64 * scale]);

        ((1.0 + (val * 1.4)) * 0.5).clamp(0.0, 1.0) as f32
    }

    pub fn get_data_2d(&self, x: f32, z: f32) -> Data2D {
        let elevation = self.get_world_noise2d(0.0, 0.01, x, z) * 5.0;
        let smoothness = self.get_world_noise2d(1.0, 0.01, x, z);

        let temperature = self.get_world_noise2d(2.0, 0.0025, x, z);
        let humidity = self.get_world_noise2d(3.0, 0.0025, x, z);
        let lushness = self.get_world_noise2d(4.0, 0.01, x, z);
        let development = self.get_world_noise2d(5.0, 0.01, x, z);

        // Rock types for colour, iron is red, calcium is white, graphite is black, apatite is blue
        let calcium = self.get_world_noise2d(6.0, 0.01, x, z);
        let graphite = self.get_world_noise2d(7.0, 0.01, x, z);
        let iron = self.get_world_noise2d(8.0, 0.01, x, z);
        let rock_color = Vec3::new(
            calcium * 0.8 - graphite * 0.5 + iron * 0.3,
            calcium * 0.8 - graphite * 0.5 + iron * 0.05,
            calcium * 0.8 - graphite * 0.5,
        );

        // Get data for the room
        // Get 2d room center position, pos2d snapped to nearest room spacing point
        let room_position = [
            (x / ROOM_SPACING).round() * ROOM_SPACING,
            (z / ROOM_SPACING).round() * ROOM_SPACING,
        ];
        // Get room noise seed, based on room position
        let room_seed = room_position[0] + room_position[1] * 123.0;

        // Get position offset by noise, so it is not on a perfect grid
        let horizontal_offset = [
            self.get_world_noise(2.0, 0.025, z / 4.0) * (ROOM_SPACING / 3.0),
            self.get_world_noise(3.0, 0.025, x / 4.0) * (ROOM_SPACING / 3.0),
        ];
        let room_position = [
            room_position[0] + horizontal_offset[0],
            room_position[1] + horizontal_offset[1],
        ];

        // Get angle from center with x and z, from -pi to pi
        let room_angle = (z - room_position[1]).atan2(x - room_position[0]);
        // Get 2d distance from center with x and z
        let room_dist = ((x - room_position[0]).powi(2) + (z - room_position[1]).powi(2)).sqrt();

        // Calculate room size, based on noise from the angle
        let room_base_size: f32 = (lerp(20.0, 25.0, smoothness)
            + self.get_noise(room_seed) * lerp(15.0, 2.0, smoothness))
            + self.get_world_noise2d(
                4.0,
                0.01,
                x * lerp(20.0, 4.0, smoothness),
                z * lerp(20.0, 4.0, smoothness),
            ) * 40.0;
        let room_size0 =
            room_base_size + self.get_noise2d(room_seed, -PI) * room_base_size / 3.0 * smoothness;
        let room_size = room_base_size
            + (self.get_noise2d(room_seed, room_angle) * room_base_size / 3.0 * smoothness);

        // For the last 25% of the angle, so from half pi to pi, lerp towards roomSize0
        let room_size_lerp = if room_angle > PI / 2.0 {
            lerp(room_size, room_size0, (room_angle - PI / 2.0) / (PI / 2.0))
        } else {
            room_size
        };

        // Get data for the corridors
        let corridor_width = 6.0 + self.get_noise2d(x, z) * 4.0;
        let corridor_dist = (x + self.get_noise(z) * 8.0 - room_position[0])
            .abs()
            .min(z + self.get_noise(x) * 8.0 - room_position[1])
            .abs();

        // Higher numbers reduce the height exponentially
        let room_floor = 8.0 - self.get_world_noise2d(5.0, 0.01, x, z) * 4.0;
        let room_ceiling = 2.0 + self.get_world_noise2d(6.0, 0.01, x, z) * 3.0;

        // Get floor material variables
        let floor_variance1 = self.get_world_noise2d(7.0, 0.05, x, z);
        let floor_variance2 = self.get_world_noise2d(8.0, 0.15, x, z) * 0.5;
        let floor_variance3 = self.get_world_noise2d(9.0, 0.05, x + 500.0, z + 500.0) * 0.5;
        let noise_offset = self.get_world_noise2d(10.0, 0.05, x, z) * 0.02;

        // Get floor material
        let floor_material = if temperature > 0.6 + noise_offset && humidity < 0.4 + noise_offset {
            FloorMaterial::Sand
        } else if humidity > 0.5 + noise_offset
            && floor_variance1 > 0.3 + noise_offset
            && floor_variance1 - floor_variance2 > 0.05 + noise_offset
        {
            FloorMaterial::Moss
        } else if humidity > 0.5 + noise_offset
            && (floor_variance1 - floor_variance2 * 0.5 > 0.05 + noise_offset
                || floor_variance2 + noise_offset < 0.3)
        {
            FloorMaterial::Dirt
        } else {
            FloorMaterial::Stone
        };

        Data2D {
            elevation,
            smoothness,
            temperature,
            humidity,
            lushness,
            development,
            rock_color,
            room_position,
            room_dist,
            room_size: room_size_lerp,
            corridor_width,
            corridor_dist,
            room_floor,
            room_ceiling,
            floor_material,
            floor_variance1,
            floor_variance2,
            floor_variance3,
        }
    }

    pub fn get_data_3d(&self, data2d: &Data2D, x: f32, z: f32, y: f32) -> bool {
        let room_height_smooth: f32 = if y < 0.0 {
            data2d.room_floor
        } else {
            data2d.room_ceiling
        };
        let room_dist_3d: f32 = ((x - data2d.room_position[0]).powi(2)
            + (z - data2d.room_position[1]).powi(2)
            + (y * room_height_smooth).powi(2))
        .sqrt();
        let room_inside_3d: bool = room_dist_3d < data2d.room_size;

        let corridor_dist_3d: f32 =
            (data2d.corridor_dist.powi(2) + (y * room_height_smooth / 2.0).powi(2)).sqrt();
        let corridor_inside_3d: bool = corridor_dist_3d < data2d.corridor_width;

        room_inside_3d || corridor_inside_3d
    }

    pub fn get_data_color(&self, data2d: &Data2D, x: f32, z: f32, y: f32) -> DataColor {
        // Color from dark to light gray as elevation increases
        let shade: f32 = y / 50.0;
        let mut color = data2d.rock_color + shade;

        // Give the color horizontal lines from noise to make it look more natural
        let noise_shade: f32 = 0.1 + self.get_noise(y * 20.0 + x * 0.01 + z + 0.01) * 0.1;
        color += noise_shade;
        // Add brown colors based on 2d noise
        let noise_color = 0.5 + self.get_world_noise2d(0.0, 0.1, x, z) / 2.0;
        color += Vec3::new(noise_color * 0.1, noise_color * 0.05, 0.0);
        // Add dark stone patches
        if data2d.floor_variance3 < 0.5 {
            color = color.lerp(color * 0.5, smoothstep(0.5, 0.3, data2d.floor_variance3));
        }

        // Add color to floors
        // if y < (data2d.room_floor - 4.0) * 4.0 - 2.0 {
        //     let color_variance = data2d.floor_variance1 * 0.15;
        //     color = match data2d.floor_material {
        //         FloorMaterial::Sand => Vec3::new(
        //             1.0 + color_variance,
        //             0.9 + color_variance,
        //             0.6 + color_variance,
        //         ),
        //         FloorMaterial::Dirt => Vec3::new(
        //             0.6 + color_variance,
        //             0.3 + color_variance,
        //             0.05 + color_variance,
        //         ),
        //         _ => color,
        //     };
        // }
        // if data2d.floor_material == FloorMaterial::Moss {
        //     let color_variance = data2d.floor_variance1 * 0.15;
        //     color = Vec3::new(0.3, 0.4, 0.1).lerp(Vec3::new(0.2, 0.4, 0.15), data2d.lushness)
        //         + Vec3::new(color_variance, color_variance, color_variance);
        // }

        // Jitter the position with noise to make it look more natural
        let pos_jittered = Vec3::new(
            x + (self.get_noise2d(z, y) * 0.2),
            y + data2d.elevation,
            z + (self.get_noise2d(x, y) * 0.2),
        );

        DataColor {
            color,
            pos_jittered,
        }
    }
}
