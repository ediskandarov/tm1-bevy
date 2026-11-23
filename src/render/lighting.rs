pub(crate) type LightMatrix = [[i16; 3]; 3];
pub(crate) const WORLD_LIGHT: LightMatrix =
    [[617, -3494, 2046], [-3425, 1982, 1056], [3332, 1928, 1398]];

pub(crate) fn rotate_light(light: LightMatrix, rotation: LightMatrix) -> LightMatrix {
    light.map(|row| {
        rotation.map(|axis| {
            let dot = (0..3)
                .map(|i| i64::from(axis[i]) * i64::from(row[i]))
                .sum::<i64>();
            (dot >> 12).clamp(-32768, 32767) as i16
        })
    })
}

#[derive(Clone, Copy)]
pub(crate) struct LightEnvironment {
    ambient: i32,
    colors: [[i32; 3]; 3],
}

impl LightEnvironment {
    pub const WORLD: Self = Self {
        ambient: 32,
        colors: [[200, 128, 60], [200, 75, 128], [200, 75, 60]],
    };
    // screenVehicleChoice changes the third light to (60, 60, 128).
    pub const GARAGE: Self = Self {
        ambient: 32,
        colors: [[200, 128, 60], [200, 75, 60], [200, 75, 128]],
    };
    pub fn battle(id: u8) -> Self {
        match id {
            1 => Self {
                ambient: 32,
                colors: [[255; 3], [0; 3], [0; 3]],
            },
            2 => Self {
                ambient: 32,
                colors: [[60; 3], [140; 3], [170; 3]],
            },
            3 => Self {
                ambient: 256,
                colors: [[256, 0, 0]; 3],
            },
            _ => Self::WORLD,
        }
    }
    pub fn vertex_rgb(
        self,
        v: &tm1_assets::scene::Vertex,
        scale: f32,
        light: LightMatrix,
    ) -> [i32; 3] {
        let rgb = std::array::from_fn(|i| (v.color[i] * scale).round() as i32);
        v.normal
            .map_or(rgb, |normal| self.lit_rgb(rgb, normal, light))
    }
    fn lit_rgb(self, rgb: [i32; 3], normal: [i16; 3], light: LightMatrix) -> [i32; 3] {
        // NCCS clamps IR after both matrix products, then modulates the source RGB.
        let incidence = light.map(|row| {
            let dot = (0..3)
                .map(|i| i32::from(row[i]) * i32::from(normal[i]))
                .sum::<i32>();
            (dot >> 12).clamp(0, 32767)
        });
        std::array::from_fn(|k| {
            let sum = (0..3)
                .map(|i| self.colors[k][i] * incidence[i])
                .sum::<i32>();
            let intensity = (self.ambient * 16 + (sum >> 8)).clamp(0, 32767);
            (rgb[k] * intensity >> 12).min(255)
        })
    }
}
