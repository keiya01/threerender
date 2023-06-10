use std::{cell::RefCell, fmt::Debug, rc::Rc};

use threerender_math::{Vec2, Vec3};

use super::{types::Topology, utils::Vertex};

/// Define an entity. Entity will be used to draw mesh.
pub trait Mesh: Debug {
    /// Required to return vertices.
    fn vertex(&self) -> Rc<RefCell<Vec<Vertex>>>;

    /// Define indices to draw an entity more efficiently.
    fn index(&self) -> Option<&[u16]>;

    /// Set topology type. Default is `TriangleList`.
    fn topology(&self) -> Topology {
        Default::default()
    }

    /// Make the tangent space for the normal mapping.
    fn as_tangent_space(&self) -> Rc<RefCell<Vec<Vertex>>> {
        let vertex = self.vertex();
        let mut vertices = vertex.borrow_mut();
        let mut triangles_included = vec![0; vertices.len()];

        let mut default_index = vec![];
        let index = self.index().map_or_else(
            || {
                for (i, _) in vertices.iter().enumerate() {
                    default_index.push(i as u16);
                }
                &default_index[..]
            },
            |v| v,
        );

        // Calculate tangents and bitangets.
        // Loop index by each triangle
        // Refer the following resources
        // - https://sotrh.github.io/learn-wgpu/intermediate/tutorial11-normals/#tangent-space-to-world-space
        // - http://www.opengl-tutorial.org/intermediate-tutorials/tutorial-13-normal-mapping/
        for c in index.chunks(3) {
            let v0 = vertices[c[0] as usize];
            let v1 = vertices[c[1] as usize];
            let v2 = vertices[c[2] as usize];

            let pos0 = Vec3::from_array(&[v0.pos[0], v0.pos[1], v0.pos[2]]);
            let pos1 = Vec3::from_array(&[v1.pos[0], v1.pos[1], v1.pos[2]]);
            let pos2 = Vec3::from_array(&[v2.pos[0], v2.pos[1], v2.pos[2]]);

            let uv0 = Vec2::from_slice(&v0.tex);
            let uv1 = Vec2::from_slice(&v1.tex);
            let uv2 = Vec2::from_slice(&v2.tex);

            // Calculate the edges of the triangle
            let delta_pos1 = pos1 - pos0;
            let delta_pos2 = pos2 - pos0;

            // This will give us a direction to calculate the
            // tangent and bitangent
            let delta_uv1 = uv1 - uv0;
            let delta_uv2 = uv2 - uv0;

            // Solving the following system of equations will
            // give us the tangent and bitangent.
            //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
            //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
            // Luckily, the place I found this equation provided
            // the solution!
            let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
            let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
            // We flip the bitangent to enable right-handed normal
            // maps with wgpu texture coordinate system
            let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

            // We'll use the same tangent/bitangent for each vertex in the triangle
            vertices[c[0] as usize].tangent =
                (tangent + Vec3::from_array(&vertices[c[0] as usize].tangent)).into();
            vertices[c[0] as usize].bitangent =
                (bitangent + Vec3::from_array(&vertices[c[0] as usize].bitangent)).into();
            vertices[c[1] as usize].tangent =
                (tangent + Vec3::from_array(&vertices[c[1] as usize].tangent)).into();
            vertices[c[1] as usize].bitangent =
                (bitangent + Vec3::from_array(&vertices[c[1] as usize].bitangent)).into();
            vertices[c[2] as usize].tangent =
                (tangent + Vec3::from_array(&vertices[c[2] as usize].tangent)).into();
            vertices[c[2] as usize].bitangent =
                (bitangent + Vec3::from_array(&vertices[c[2] as usize].bitangent)).into();

            // Used to average the tangents/bitangents
            triangles_included[c[0] as usize] += 1;
            triangles_included[c[1] as usize] += 1;
            triangles_included[c[2] as usize] += 1;
        }

        // Average the tangents/bitangents
        for (i, n) in triangles_included.into_iter().enumerate() {
            let denom = 1.0 / n as f32;
            let mut v = &mut vertices[i];
            v.tangent = (Vec3::from_array(&v.tangent) * denom).into();
            v.bitangent = (Vec3::from_array(&v.bitangent) * denom).into();
        }

        vertex.clone()
    }
}
