use bevy::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct TwoSided {
    pub size: f32,
}

impl Default for TwoSided {
    fn default() -> TwoSided {
        TwoSided { size: 1.0 }
    }
}

impl From<TwoSided> for Mesh {
    fn from(two_sided: TwoSided) -> Mesh {
        let extent = two_sided.size / 2.0;

        let vertices = [
            ([extent, 0.0, -extent], [0.0, 1.0, 0.0], [0.5, 1.0]),
            ([extent, 0.0, extent], [0.0, 1.0, 0.0], [0.5, 0.0]),
            ([-extent, 0.0, extent], [0.0, 1.0, 0.0], [0.0, 0.0]),
            ([-extent, 0.0, -extent], [0.0, 1.0, 0.0], [0.0, 1.0]),
            ([extent, 0.0, -extent], [0.0, -1.0, 0.0], [1.0, 1.0]),
            ([extent, 0.0, extent], [0.0, -1.0, 0.0], [1.0, 0.0]),
            ([-extent, 0.0, extent], [0.0, -1.0, 0.0], [0.5, 0.0]),
            ([-extent, 0.0, -extent], [0.0, -1.0, 0.0], [0.5, 1.0]),
        ];

        let indices = bevy::render::mesh::Indices::U32(vec![
            0, 2, 1,
            0, 3, 2,
            5, 6, 4,
            6, 7, 4,
        ]);

        let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
        let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
        let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

        let mut mesh = Mesh::new(
            bevy::render::mesh::PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}
