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

    pub fn has_displacement(&self) -> bool {
        for (face, _) in &self.shape.faces {
            if let Some(face) = face {
                if face.dispinfo.is_some() {
                    return true;
                }
            }
        }
        false
    }
}

/// Represents a Face of a Brush
/// In the VMF, this is called a "side"
#[derive(Debug, Clone)]
pub struct Face {
    pub id: i32,
    plane: (glm::Vec3, glm::Vec3, glm::Vec3),
    pub material: String,
    pub uaxis: UVAxis,
    pub vaxis: UVAxis,
    lightmapscale: i32,
    smoothing_groups: i32,
    pub dispinfo: Option<Dispinfo>,
}

impl Face {
    fn parse(kv: &KeyValues) -> Option<Self> {
        //println!("{}", kv.get("id")?.get_value()?);
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
        let plane = (
            glm::vec3(x1, y1, z1),
            glm::vec3(x2, y2, z2),
            glm::vec3(x3, y3, z3),
        );
        Some(Self {
            id: kv.get("id")?.get_value()?.parse().ok()?,
            plane,
            material: kv.get("material")?.get_value()?.to_string(),
            uaxis: UVAxis::parse(kv.get("uaxis")?.get_value()?)?,
            vaxis: UVAxis::parse(kv.get("vaxis")?.get_value()?)?,
            lightmapscale: kv.get("lightmapscale")?.get_value()?.parse().ok()?,
            smoothing_groups: kv.get("smoothing_groups")?.get_value()?.parse().ok()?,
            dispinfo: match kv.get("dispinfo") {
                Some(info) => Some(Dispinfo::parse(
                    info,
                    &glm::normalize(&glm::cross(&(plane.2 - plane.0), &(plane.1 - plane.0))),
                )?),
                None => None,
            },
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
    pub power: u8,
    pub startpos: glm::Vec3,
    pub elevation: f32,
    pub subdiv: bool,
    pub normals: Vec<Vec<glm::Vec3>>,
    pub distances: Vec<Vec<f32>>,
    pub offsets: Vec<Vec<glm::Vec3>>,
    // pub offset_normals: Vec<Vec<glm::Vec3>>,
    pub alphas: Vec<Vec<u8>>,
}

impl Dispinfo {
    fn parse(kv: &KeyValues, face_normal: &glm::Vec3) -> Option<Self> {
        let power = kv.get("power")?.get_value()?.parse::<u8>().ok()?;
        let startpos = {
            let mut x = 0.0;
            let mut y = 0.0;
            let mut z = 0.0;
            sscanf!(kv.get("startposition")?.get_value()?, "[{} {} {}]", x, y, z).ok()?;
            glm::vec3(x, y, z)
        };
        let mut normals = get_dispdata_3(power, kv.get("normals")?)?;
        let mut distances = vec![];
        for row in 0..((1 << power) + 1) {
            let mut row_data = vec![];
            let mut values: Vec<f32> = vec![];
            for value in kv
                .get("distances")?
                .get(&format!("row{}", row))?
                .get_value()?
                .split(' ')
            {
                values.push(value.parse().ok()?);
            }
            for column in 0..((1 << power) + 1) {
                row_data.push(values[column as usize]);
            }
            distances.push(row_data);
        }
        let offsets =
            (|| -> Option<Vec<Vec<glm::Vec3>>> { get_dispdata_3(power, kv.get("offsets")?) })()
                .unwrap_or(vec![
                    vec![glm::vec3(0.0, 0.0, 0.0); (1 << power) + 1];
                    (1 << power) + 1
                ]);
        let mut alphas = vec![];
        for row in 0..((1 << power) + 1) {
            let mut row_data = vec![];
            let mut values: Vec<u8> = vec![];
            for value in kv
                .get("alphas")?
                .get(&format!("row{}", row))?
                .get_value()?
                .split(' ')
            {
                // Why can these be decimal values????
                values.push(value.parse::<f32>().ok()? as u8);
            }
            for column in 0..((1 << power) + 1) {
                row_data.push(values[column as usize]);
            }
            alphas.push(row_data);
        }
        normalize_normals(power, &mut normals, &mut distances, face_normal);
        Some(Self {
            power,
            startpos,
            elevation: kv.get("elevation")?.get_value()?.parse().ok()?,
            subdiv: kv.get("subdiv")?.get_value()? != "0",
            normals,
            distances,
            offsets,
            // offset_normals: get_dispdata_3(power, kv.get("offset_normals")?)?,
            alphas,
        })
    }
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

fn get_dispdata_3(power: u8, kv: &KeyValues) -> Option<Vec<Vec<glm::Vec3>>> {
    let mut data = vec![];
    for row in 0..((1 << power) + 1) {
        let mut row_data = vec![];
        let mut values: Vec<f32> = vec![];
        for value in kv.get(&format!("row{}", row))?.get_value()?.split(' ') {
            values.push(value.parse().ok()?);
        }
        for column in 0..((1 << power) + 1) {
            row_data.push(glm::vec3(
                values[(column * 3) as usize],
                values[(column * 3 + 1) as usize],
                values[(column * 3 + 2) as usize],
            ));
        }
        data.push(row_data);
    }
    Some(data)
}

fn normalize_normals(
    power: u8,
    normals: &mut [Vec<glm::Vec3>],
    distances: &mut [Vec<f32>],
    face_normal: &glm::Vec3,
) {
    for row in 0..((1 << power) + 1) as usize {
        for column in 0..((1 << power) + 1) as usize {
            if normals[row][column].norm_squared() == 0.0 {
                // No normal set
                normals[row][column] = *face_normal;
                distances[row][column] = 0.0;
            }
            if glm::dot(&normals[row][column], face_normal) < 0.0 {
                // Displacement normal points inside the face
                normals[row][column] *= -1.0;
                distances[row][column] *= -1.0;
            }
            if normals[row][column].norm_squared() != 1.0 {
                // Not normalized
                let lenght = normals[row][column].norm();
                normals[row][column] /= lenght;
                distances[row][column] *= lenght;
            }
        }
    }
}
