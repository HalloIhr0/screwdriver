use std::collections::{HashMap, VecDeque};

use nalgebra_glm as glm;

pub fn line_plane_intersection(
    line_point: &glm::Vec3,
    line_normal: &glm::Vec3,
    plane_point: &glm::Vec3,
    plane_normal: &glm::Vec3,
) -> Option<glm::Vec3> {
    // Calculation from https://en.wikipedia.org/w/index.php?title=Line%E2%80%93plane_intersection&oldid=1142933404#Algebraic_form
    let dividend = glm::dot(&(plane_point - line_point), plane_normal);
    let divisor = glm::dot(line_normal, plane_normal);
    if divisor == 0.0 {
        return None;
    }
    let d = dividend / divisor;
    Some(line_point + line_normal * d)
}

pub fn line_segment_plane_intersection(
    line: (&glm::Vec3, &glm::Vec3),
    plane_point: &glm::Vec3,
    plane_normal: &glm::Vec3,
) -> Option<glm::Vec3> {
    let side1 = f32::signum(glm::dot(&(line.0 - plane_point), plane_normal));
    let side2 = f32::signum(glm::dot(&(line.1 - plane_point), plane_normal));
    if side1 == side2 {
        return None;
    }
    line_plane_intersection(line.0, &(line.1 - line.0), plane_point, plane_normal)
}

pub fn clip_polyhedron_to_plane<T: Clone>(
    polyhedron: &mut Polyhedron<T>,
    plane_point: &glm::Vec3,
    plane_normal: &glm::Vec3,
    new_face_info: T,
) {
    let mut new_faces: Vec<(T, Vec<usize>)> = vec![];
    let mut new_face_edges = HashMap::new();
    for (info, face) in &polyhedron.faces {
        let mut sides = vec![];
        for vertex in face {
            sides.push(f32::signum(glm::dot(
                &(polyhedron.vertices[*vertex] - plane_point),
                plane_normal,
            )));
        }
        if !sides.contains(&-1.0) {
            // All in front of plane, can be ignored
            continue;
        } else if !sides.contains(&1.0) {
            // All behind plane, can just be kept
            new_faces.push((info.clone(), face.clone()));
            continue;
        }
        let mut iter = sides.iter().enumerate().cycle();
        let mut inside_vertices = VecDeque::new();
        let mut last = iter
            .next()
            .expect("Invalid polyhedron: Face with no vertices");
        let mut started = false;
        for (i, side) in iter {
            if !started {
                if last.1 == &1.0 && side == &-1.0 {
                    inside_vertices.push_back(face[last.0]);
                    inside_vertices.push_back(face[i]);
                    started = true;
                }
                last = (i, side);
            } else {
                inside_vertices.push_back(face[i]);
                if side == &1.0 {
                    break;
                }
            }
        }
        let new_vertex_one = line_segment_plane_intersection(
            (
                &polyhedron.vertices[inside_vertices
                    .pop_front()
                    .expect("There needs to be at least two intersections")],
                &polyhedron.vertices[*inside_vertices
                    .front()
                    .expect("There needs to be at least two intersections")],
            ),
            plane_point,
            plane_normal,
        )
        .expect("The sides have been checked before, must intersect");
        let new_vertex_one = match polyhedron
            .vertices
            .iter()
            .position(|x| x == &new_vertex_one)
        {
            Some(i) => i,
            None => {
                polyhedron.vertices.push(new_vertex_one);
                polyhedron.vertices.len() - 1
            }
        };
        let new_vertex_two = line_segment_plane_intersection(
            (
                &polyhedron.vertices[inside_vertices
                    .pop_back()
                    .expect("There needs to be at least two intersections")],
                &polyhedron.vertices[*inside_vertices
                    .back()
                    .expect("There needs to be at least two intersections")],
            ),
            plane_point,
            plane_normal,
        )
        .expect("The sides have been checked before, must intersect");
        let new_vertex_two = match polyhedron
            .vertices
            .iter()
            .position(|x| x == &new_vertex_two)
        {
            Some(i) => i,
            None => {
                polyhedron.vertices.push(new_vertex_two);
                polyhedron.vertices.len() - 1
            }
        };
        new_face_edges.insert(new_vertex_one, new_vertex_two);
        inside_vertices.push_front(new_vertex_one);
        inside_vertices.push_front(new_vertex_two);
        new_faces.push((info.clone(), inside_vertices.into()));
    }
    if !new_face_edges.is_empty() {
        let start = *new_face_edges
            .keys()
            .next()
            .expect("Invalid polyhedron: Unconnected vertex");
        let mut edge_loop = vec![start];
        let mut current = new_face_edges[&start];
        while current != start {
            edge_loop.push(current);
            current = new_face_edges[&current];
        }
        assert_eq!(new_face_edges.len(), edge_loop.len());
        new_faces.push((new_face_info, edge_loop));
    }
    polyhedron.faces = new_faces
}

#[derive(Debug)]
pub struct Polyhedron<T> {
    pub vertices: Vec<glm::Vec3>,
    pub faces: Vec<(T, Vec<usize>)>,
}

#[cfg(test)]
mod tests {
    use glm::vec3;

    use super::*;

    #[test]
    fn test_line_plane_intersection() {
        assert_eq!(
            line_plane_intersection(
                &vec3(6.1, -5.7, -2.5),
                &vec3(9.7, -17.5, -9.5),
                &vec3(4.9, 5.7, 2.7),
                &vec3(
                    0.17176008250171795,
                    0.12437799077710607,
                    -0.9772556418201193
                )
            ),
            Some(glm::vec3(1.8213401, 2.0192318, 1.6904402))
        );
    }

    // fn test_clip_polyhedron_to_plane() {

    // }
}
