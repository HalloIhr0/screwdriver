use glm::Vec3;
use nalgebra_glm as glm;
use scanf::sscanf;
use std::path::Path;

use crate::{
    keyvalue::KeyValues,
    math::{self, Polyhedron},
};

const MAX_MAP_EXTENT: f32 = 16384.0;

#[derive(Debug)]
pub struct VMF {
    pub worldbrushes: Vec<Brush>,
}

impl VMF {
    pub fn parse(file: &Path) -> Option<Self> {
        let kv = KeyValues::parse(file).ok()?;
        let mut worldbrushes = vec![];
        for solid in kv.get("world")?.get_all("solid") {
            worldbrushes.push(Brush::parse(solid)?);
        }

        Some(Self { worldbrushes })
    }
}

pub type BrushShape = Polyhedron<Option<Face>>;

/// Represents a Brush
/// In the VMF, this is called a "solid"
#[derive(Debug)]
pub struct Brush {
    id: i32,
    pub shape: BrushShape,
}

impl Brush {
    fn parse(kv: &KeyValues) -> Option<Self> {
        let mut faces = vec![];
        for side in kv.get_all("side") {
            faces.push(Face::parse(side)?);
        }
        Some(Self {
            id: kv.get("id")?.get_value()?.parse().ok()?,
            shape: get_polyhedron(faces),
        })
    }
}

/// Represents a Face of a Brush
/// In the VMF, this is called a "side"
#[derive(Debug, Clone)]
pub struct Face {
    id: i32,
    plane: (glm::Vec3, glm::Vec3, glm::Vec3),
    pub material: String,
    pub uaxis: UVAxis,
    pub vaxis: UVAxis,
    lightmapscale: i32,
    smoothing_groups: i32,
    dispinfo: Option<Dispinfo>,
}

impl Face {
    fn parse(kv: &KeyValues) -> Option<Self> {
        let mut x1: f32 = 0.0;
        let mut y1: f32 = 0.0;
        let mut z1: f32 = 0.0;
        let mut x2: f32 = 0.0;
        let mut y2: f32 = 0.0;
        let mut z2: f32 = 0.0;
        let mut x3: f32 = 0.0;
        let mut y3: f32 = 0.0;
        let mut z3: f32 = 0.0;
        sscanf!(
            kv.get("plane")?.get_value()?,
            "({} {} {}) ({} {} {}) ({} {} {})",
            x1,
            y1,
            z1,
            x2,
            y2,
            z2,
            x3,
            y3,
            z3
        )
        .ok()?;
        Some(Self {
            id: kv.get("id")?.get_value()?.parse().ok()?,
            plane: (
                glm::vec3(x1, y1, z1),
                glm::vec3(x2, y2, z2),
                glm::vec3(x3, y3, z3),
            ),
            material: kv.get("material")?.get_value()?.to_string(),
            uaxis: UVAxis::parse(kv.get("uaxis")?.get_value()?)?,
            vaxis: UVAxis::parse(kv.get("vaxis")?.get_value()?)?,
            lightmapscale: kv.get("lightmapscale")?.get_value()?.parse().ok()?,
            smoothing_groups: kv.get("smoothing_groups")?.get_value()?.parse().ok()?,
            dispinfo: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct UVAxis {
    pub dir: Vec3,
    pub translation: f32,
    pub scaling: f32,
}

impl UVAxis {
    fn parse(input: &str) -> Option<Self> {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;
        let mut translation = 0.0;
        let mut scaling = 0.0;
        sscanf!(input, "[{} {} {} {}] {}", x, y, z, translation, scaling).ok()?;
        let dir = glm::vec3(x, y, z);
        if f32::abs(dir.norm_squared() - 1.0) > 0.1 {
            eprintln!("UVaxis isn't normalized");
            // TODO: handle this properly instead of just giving an error message
        }
        Some(Self {
            dir,
            translation,
            scaling,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Dispinfo {
    power: DispPower,
    startpos: glm::Vec3,
    elevation: f32,
    subdiv: bool,
    normals: Vec<Vec<glm::Vec3>>,
    distances: Vec<Vec<f32>>,
    offsets: Vec<Vec<glm::Vec3>>,
    offset_normals: Vec<Vec<glm::Vec3>>,
    alphas: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
enum DispPower {
    Power2 = 2,
    Power3 = 3,
    Power4 = 4,
}

fn get_polyhedron(faces: Vec<Face>) -> BrushShape {
    let mut poly = Polyhedron {
        vertices: vec![
            glm::vec3(-MAX_MAP_EXTENT, -MAX_MAP_EXTENT, MAX_MAP_EXTENT),
            glm::vec3(-MAX_MAP_EXTENT, MAX_MAP_EXTENT, MAX_MAP_EXTENT),
            glm::vec3(-MAX_MAP_EXTENT, -MAX_MAP_EXTENT, -MAX_MAP_EXTENT),
            glm::vec3(-MAX_MAP_EXTENT, MAX_MAP_EXTENT, -MAX_MAP_EXTENT),
            glm::vec3(MAX_MAP_EXTENT, -MAX_MAP_EXTENT, MAX_MAP_EXTENT),
            glm::vec3(MAX_MAP_EXTENT, MAX_MAP_EXTENT, MAX_MAP_EXTENT),
            glm::vec3(MAX_MAP_EXTENT, -MAX_MAP_EXTENT, -MAX_MAP_EXTENT),
            glm::vec3(MAX_MAP_EXTENT, MAX_MAP_EXTENT, -MAX_MAP_EXTENT),
        ],
        faces: vec![
            (None, vec![0, 1, 3, 2]),
            (None, vec![2, 3, 7, 6]),
            (None, vec![6, 7, 5, 4]),
            (None, vec![4, 5, 1, 0]),
            (None, vec![2, 6, 4, 0]),
            (None, vec![7, 3, 1, 5]),
        ],
    };
    for face in faces {
        let normal = glm::normalize(&glm::cross(
            &(face.plane.2 - face.plane.0),
            &(face.plane.1 - face.plane.0),
        ));
        math::clip_polyhedron_to_plane(&mut poly, &face.plane.0, &normal, Some(face.clone()));
    }
    poly
}
