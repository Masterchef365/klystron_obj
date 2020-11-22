use anyhow::{bail, Result};
use klystron::Vertex;
use obj::raw::object::Polygon;
pub use obj::raw::{parse_obj, RawObj};
use std::collections::HashMap;
// Special case for quads. Any higher order polygon will throw an error!
// High-level wrapper, then some lower level functions
// Don't forget the examples/ !

// TODO: Replace me with something more... fixed function
fn polygon_vert_norm_pairs<'a>(
    poly: &'a Polygon,
) -> Box<dyn Iterator<Item = (usize, Option<usize>)> + 'a> {
    match poly {
        Polygon::P(v) => Box::new(v.iter().copied().map(|p| (p, None))),
        Polygon::PN(v) => Box::new(v.iter().copied().map(|(p, _)| (p, None))),
        Polygon::PT(v) => Box::new(v.iter().copied().map(|(p, t)| (p, Some(t)))),
        Polygon::PTN(v) => Box::new(v.iter().copied().map(|(p, t, _)| (p, Some(t)))),
    }
}

fn deref_vertex((p, t): (usize, Option<usize>), obj: &RawObj) -> Vertex {
    let (u, v, w) = match t {
        Some(t) => obj.tex_coords[t],
        None => (1., 1., 1.),
    };
    let (x, y, z, _) = obj.positions[p];
    Vertex {
        pos: [x, y, z],
        color: [u, v, w],
    }
}

pub fn triangles(obj: &RawObj) -> Result<(Vec<Vertex>, Vec<u16>)> {
    pub fn poly_triangles(indices: &mut Vec<u16>, current_polygon_indices: &[u16]) -> Result<()> {
        match current_polygon_indices.len() {
            3 => Ok(indices.extend(current_polygon_indices.iter().copied())),
            4 => {
                let c = &current_polygon_indices;
                indices.extend_from_slice(&[c[0], c[1], c[2]]);
                indices.extend_from_slice(&[c[0], c[2], c[3]]);
                Ok(())
            }
            _ => bail!("Polygon is not a triangle or quad"),
        }
    }

    gen_mesh(obj, poly_triangles)
}

// Dumb, repeats eery triangle like 3 times
pub fn gen_mesh(
    obj: &RawObj,
    mut f: impl FnMut(&mut Vec<u16>, &[u16]) -> Result<()>,
) -> Result<(Vec<Vertex>, Vec<u16>)> {
    let mut indices: Vec<u16> = Vec::new();
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut current_polygon_indices = Vec::with_capacity(4); // Enough space for one quad

    for polyon in &obj.polygons {
        current_polygon_indices.clear();

        // Deduplicate position/normal index pairs into vertices
        for pair in polygon_vert_norm_pairs(polyon) {
            let idx = vertices.len() as u16;
            vertices.push(deref_vertex(pair, obj));
            current_polygon_indices.push(idx);
        }

        f(&mut indices, &current_polygon_indices)?;
        // Special cases for quads and triangles
    }

    Ok((vertices, indices))
}

/// Actual lines represented in the OBJ
pub fn lines(obj: &RawObj) -> (Vec<Vertex>, Vec<u16>) {
    let mut indices: Vec<u16> = Vec::new();
    let mut vertices: Vec<Vertex> = Vec::new();
    todo!()
}

/// Load a mesh, the `color` parameter of each vertex will be the UV coordinates if they exist, and
/// ~1.0` otherwise
pub fn wireframe(obj: &RawObj) -> (Vec<Vertex>, Vec<u16>) {
    todo!()
}