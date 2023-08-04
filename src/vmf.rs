use std::path::Path;
use scanf::sscanf;

use crate::keyvalue::{KeyValues, KeyValuesError};

pub struct VMF {
    worldbrushes: Vec<Brush>,
}

impl VMF {
    pub fn parse(file: &Path) -> Option<Self> {
        let kv = KeyValues::parse(file).ok()?;

        let mut worldbrushes = vec![];
        for solid in kv.get("world")?.get_all("solid") {
            worldbrushes.push(Brush::parse(solid));
        }

        None
    }
}

/// Represents a Brush
/// In the VMF, this is called a "solid"
struct Brush {
    id: i32,
    faces: Vec<Face>,
}

impl Brush {
    fn parse(kv: &KeyValues) -> Option<Self> {
        let mut faces = vec![];
        for side in kv.get_all("side") {
            faces.push(Face::parse(side)?);
        }
        Some(Self {
            id: kv.get("id")?.get_value()?.parse().ok()?,
            faces,
        })
    }
}

/// Represents a Face of a Brush
/// In the VMF, this is called a "side"
struct Face {
    id: i32,
    plane: (),
    material: String,
    uaxis: UVAxis,
    vaxis: UVAxis,
    rotation: f32,
    lightmapscale: i32,
    smoothing_groups: i32,
}

impl Face {
    fn parse(kv: &KeyValues) -> Option<Self> {
        Some(Self {
            id: kv.get("id")?.get_value()?.parse().ok()?,
            plane: todo!(),
            material: kv.get("material")?.get_value()?.to_string(),
            uaxis: UVAxis::parse(kv.get("uaxis")?.get_value()?)?,
            vaxis: UVAxis::parse(kv.get("vaxis")?.get_value()?)?,
            rotation: kv.get("rotation")?.get_value()?.parse().ok()?,
            lightmapscale: kv.get("lightmapscale")?.get_value()?.parse().ok()?,
            smoothing_groups: kv.get("smoothing_groups")?.get_value()?.parse().ok()?,
        })
    }
}

struct UVAxis {
    x: f32,
    y: f32,
    z: f32,
    translation: f32,
    scaling: f32,
}

impl UVAxis {
    fn parse(input: &str) -> Option<Self> {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;
        let mut translation = 0.0;
        let mut scaling = 0.0;
        sscanf!(input, "[{} {} {} {}] {}", x, y, z, translation, scaling).ok()?;
        Some( Self {x, y, z, translation, scaling})
    }
}
