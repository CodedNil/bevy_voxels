use noise::{NoiseFn, OpenSimplex};
use std::f32::consts::PI;

fn lerp(start: f32, end: f32, percentage: f32) -> f32 {
    start + percentage * (end - start)
}

pub struct DataGenerator {
    pub world_noise: OpenSimplex,
}

pub struct Data2D {
    pub elevation: f32,
    pub smoothness: f32,
    pub room_position: [f32; 2],
    pub room_dist: f32,
    pub room_size: f32,
    pub corridor_width: f32,
    pub corridor_dist: f32,
    pub room_floor: f32,
    pub room_ceiling: f32,
}

pub struct DataColor {
    pub color: (f32, f32, f32),
    pub material: String,
    pub pos_jittered: (f32, f32, f32),
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

        let room_spacing = 150.0;

        // Get data for the room
        // Get 2d room center position, pos2d snapped to nearest room spacing point
        let room_position = [
            (x / room_spacing).round() * room_spacing,
            (z / room_spacing).round() * room_spacing,
        ];
        // Get room noise seed, based on room position
        let room_seed = room_position[0] + room_position[1] * 123.0;

        // Get position offset by noise, so it is not on a perfect grid
        let horizontal_offset = [
            self.get_world_noise(2.0, 0.025, z / 4.0) * (room_spacing / 3.0),
            self.get_world_noise(3.0, 0.025, x / 4.0) * (room_spacing / 3.0),
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

        Data2D {
            elevation,
            smoothness,
            room_position,
            room_dist,
            room_size: room_size_lerp,
            corridor_width,
            corridor_dist,
            room_floor,
            room_ceiling,
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
        let rock_color: (f32, f32, f32) = (0.8, 0.6, 0.3);
        let color: (f32, f32, f32) = (
            rock_color.0 + shade,
            rock_color.1 + shade,
            rock_color.2 + shade,
        );
        let material = "standard".to_string();

        // Give the color horizontal lines from noise to make it look more natural
        let noise_shade: f32 = 0.1 + self.get_noise(y * 20.0 + x * 0.01 + z + 0.01) * 0.1;
        let color = (
            color.0 + noise_shade,
            color.1 + noise_shade,
            color.2 + noise_shade,
        );
        // Add brown colors based on 2d noise
        let noise_color = 0.5 + self.get_world_noise2d(0.0, 0.1, x, z) / 2.0;
        let color = (
            color.0 + noise_color * 0.1,
            color.1 + noise_color * 0.05,
            color.2,
        );

        // Jitter the position with noise to make it look more natural
        let pos_jittered = (
            x + (self.get_noise2d(z, y) * 0.2),
            z + (self.get_noise2d(x, y) * 0.2),
            y + data2d.elevation,
        );

        DataColor {
            color,
            material,
            pos_jittered,
        }
    }
}
