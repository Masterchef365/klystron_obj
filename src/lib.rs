use anyhow::{bail, Result};
use klystron::Vertex;
use obj::raw::object::Polygon;
pub use obj::raw::{parse_obj, RawObj};
use std::collections::{HashMap, HashSet};
// Special case for quads. Any higher order polygon will throw an error!
// High-level wrapper, then some lower level functions
// Don't forget the examples/ !

/// Generate a triangle mesh for this
pub fn triangles(obj: &RawObj, extras: Extras) -> Result<(Vec<Vertex>, Vec<u16>)> {
    polygon_indices(obj, |indices, poly| -> Result<()> {
        match poly.len() {
            3 => Ok(indices.extend(poly.iter().copied())),
            4 => {
                indices.extend_from_slice(&[poly[0], poly[1], poly[2]]);
                indices.extend_from_slice(&[poly[0], poly[2], poly[3]]);
                Ok(())
            }
            _ => bail!("Polygon is not a triangle or quad"),
        }
    }, extras)
}

/// Whether or not to tessellate quads when generating a wireframe
pub enum QuadMode {
    Keep,
    Tessellate,
}

/// Generate a wireframe from this OBJ. Capable of preserving quads (see `[crate::QuadMode]`).
pub fn wireframe(obj: &RawObj, quad_mode: QuadMode, extras: Extras) -> Result<(Vec<Vertex>, Vec<u16>)> {
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
        QuadMode::Tessellate => polygon_indices(obj, |indices, poly| -> Result<()> {
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
        }, extras),
        QuadMode::Keep => polygon_indices(obj, |indices, poly| -> Result<()> {
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
        }, extras),
    }
}

/*
pub fn lines(obj: &RawObj) -> (Vec<Vertex>, Vec<u16>) {
    todo!()
}
*/

/// Generate indices, memoizing vertices and using the specified functino `f` to generate
/// output indices according to a rule (wireframe, triangles, quads)
fn polygon_indices(
    obj: &RawObj,
    mut f: impl FnMut(&mut Vec<u16>, &[u16]) -> Result<()>,
    extras: Extras,
) -> Result<(Vec<Vertex>, Vec<u16>)> {
    let mut indices: Vec<u16> = Vec::new();
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut poly = Vec::with_capacity(4); // Enough space for one quad
    let mut vert_compressor: HashMap<(usize, Option<usize>), u16> = HashMap::new();

    for polyon in &obj.polygons {
        poly.clear();

        // Deduplicate position/normal index pairs into vertices
        let iter = match extras {
            Extras::UVW => polygon_uvw(polyon),
            Extras::Normals => polygon_normal(polyon),
        };
        for pair in iter {
            let idx = match vert_compressor.get(&pair).copied() {
                None => {
                    let idx = vertices.len() as u16;
                    vertices.push(deref_vertex(pair, obj, extras));
                    vert_compressor.insert(pair, idx);
                    idx
                }
                Some(i) => i,
            };
            poly.push(idx);
        }

        f(&mut indices, &poly)?;
        // Special cases for quads and triangles
    }

    Ok((vertices, indices))
}

// TODO: Replace me with something more... fixed function
/// Extract (position_idx, Option<uvw_idx>) pairs from a polygone,
fn polygon_uvw<'a>(
    poly: &'a Polygon,
) -> Box<dyn Iterator<Item = (usize, Option<usize>)> + 'a> {
    match poly {
        Polygon::P(v) => Box::new(v.iter().copied().map(|p| (p, None))),
        Polygon::PN(v) => Box::new(v.iter().copied().map(|(p, _)| (p, None))),
        Polygon::PT(v) => Box::new(v.iter().copied().map(|(p, uvw)| (p, Some(uvw)))),
        Polygon::PTN(v) => Box::new(v.iter().copied().map(|(p, uvw, _)| (p, Some(uvw)))),
    }
}

// TODO: Replace me with something more... fixed function
/// Extract (position_idx, Option<normal_idx>) pairs from a polygone,
fn polygon_normal<'a>(
    poly: &'a Polygon,
) -> Box<dyn Iterator<Item = (usize, Option<usize>)> + 'a> {
    match poly {
        Polygon::P(v) => Box::new(v.iter().copied().map(|p| (p, None))),
        Polygon::PN(v) => Box::new(v.iter().copied().map(|(p, n)| (p, Some(n)))),
        Polygon::PT(v) => Box::new(v.iter().copied().map(|(p, _)| (p, None))),
        Polygon::PTN(v) => Box::new(v.iter().copied().map(|(p, _, n)| (p, Some(n)))),
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Extras {
    UVW,
    Normals,
}

/// Create a vertex from an obj given (position, Option<uvw_idx>)
fn deref_vertex((p, t): (usize, Option<usize>), obj: &RawObj, extras: Extras) -> Vertex {
    let (r, g, b) = match t {
        Some(t) => match extras {
            Extras::UVW => obj.tex_coords[t],
            Extras::Normals => obj.normals[t],
        },
        None => (0., 0., 0.),
    };
    let (x, y, z, _) = obj.positions[p];
    Vertex {
        pos: [x, y, z],
        color: [r, g, b],
    }
}
