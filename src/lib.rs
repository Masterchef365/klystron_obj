use anyhow::{bail, Result};
use klystron::Vertex;
use obj::raw::object::Polygon;
pub use obj::raw::{parse_obj, RawObj};
use std::collections::HashSet;
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
    pub fn poly_triangles(indices: &mut Vec<u16>, poly: &[u16]) -> Result<()> {
        match poly.len() {
            3 => Ok(indices.extend(poly.iter().copied())),
            4 => {
                indices.extend_from_slice(&[poly[0], poly[1], poly[2]]);
                indices.extend_from_slice(&[poly[0], poly[2], poly[3]]);
                Ok(())
            }
            _ => bail!("Polygon is not a triangle or quad"),
        }
    }

    polygon_indices(obj, poly_triangles)
}

// Dumb, repeats eery triangle like 3 times
pub fn polygon_indices(
    obj: &RawObj,
    mut f: impl FnMut(&mut Vec<u16>, &[u16]) -> Result<()>,
) -> Result<(Vec<Vertex>, Vec<u16>)> {
    let mut indices: Vec<u16> = Vec::new();
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut poly = Vec::with_capacity(4); // Enough space for one quad

    for polyon in &obj.polygons {
        poly.clear();

        // Deduplicate position/normal index pairs into vertices
        for pair in polygon_vert_norm_pairs(polyon) {
            let idx = vertices.len() as u16;
            vertices.push(deref_vertex(pair, obj));
            poly.push(idx);
        }

        f(&mut indices, &poly)?;
        // Special cases for quads and triangles
    }

    Ok((vertices, indices))
}

/// Actual lines represented in the OBJ
pub fn lines(obj: &RawObj) -> (Vec<Vertex>, Vec<u16>) {
    todo!()
}

pub enum QuadMode {
    Keep,
    Tessellate,
}

pub fn wireframe(obj: &RawObj, quad_mode: QuadMode) -> Result<(Vec<Vertex>, Vec<u16>)> {
    let mut line_dedup = HashSet::new();

    let mut add_line = |indices: &mut Vec<u16>, a: u16, b: u16| {
        if !line_dedup.contains(&(a, b)) {
            indices.extend_from_slice(&[a, b]);
            line_dedup.insert((a, b));
            line_dedup.insert((b, a));
        }
    };

    fn quad_check(poly: &[u16]) -> Result<()> {
        if poly.len() < 3 || poly.len() > 4 {
            bail!("Polygon is not a triangle or quad");
        }
        Ok(())
    }

    match quad_mode {
        QuadMode::Tessellate => {
            polygon_indices(obj, |indices, poly| -> Result<()> {
                quad_check(poly)?;

                add_line(indices, poly[0], poly[1]);
                add_line(indices, poly[1], poly[2]);
                add_line(indices, poly[2], poly[0]);

                if poly.len() == 4 {
                    add_line(indices, poly[0], poly[2]);
                    add_line(indices, poly[2], poly[3]);
                    add_line(indices, poly[3], poly[0]);
                }

                Ok(())
            })
        }
        QuadMode::Keep => {
            polygon_indices(obj, |indices, poly| -> Result<()> {
                quad_check(poly)?;

                add_line(indices, poly[0], poly[1]);
                add_line(indices, poly[1], poly[2]);

                if poly.len() == 4 {
                    add_line(indices, poly[2], poly[3]);
                    add_line(indices, poly[3], poly[0]);
                } else {
                    add_line(indices, poly[2], poly[0]);
                }

                Ok(())
            })
        }
    }
}
