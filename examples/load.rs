use anyhow::{Context, Result};
use klystron::{
    runtime_3d::{launch, App},
    DrawType, Engine, FramePacket, Object, UNLIT_FRAG, UNLIT_VERT,
};
use klystron_obj::{parse_obj, triangles, wireframe, QuadMode};
use nalgebra::{Matrix4, Vector3};
use std::fs::File;
use std::io::BufReader;

struct MyApp {
    quad_wires: Object,
    tess_wires: Object,
    tris: Object,
    time: f32,
}

impl App for MyApp {
    const NAME: &'static str = "MyApp";

    type Args = (String, String);

    fn new(engine: &mut dyn Engine, (obj_path, texture_path): Self::Args) -> Result<Self> {
        let file = BufReader::new(File::open(obj_path)?);
        let obj = parse_obj(file).context("Error parsing OBJ")?;

        let line_material = engine.add_material(UNLIT_VERT, UNLIT_FRAG, DrawType::Lines)?;

        // Read important image data
        let img = png::Decoder::new(File::open(texture_path)?);
        let (info, mut reader) = img.read_info()?;
        assert!(info.color_type == png::ColorType::RGB);
        assert!(info.bit_depth == png::BitDepth::Eight);
        let mut img_buffer = vec![0; info.buffer_size()];
        assert_eq!(info.buffer_size(), (info.width * info.height * 3) as _);
        reader.next_frame(&mut img_buffer)?;
        let texture = engine.add_texture(&img_buffer, info.width)?;

        // Tessellated wires
        let (vertices, indices) = wireframe(&obj, QuadMode::Tessellate)?;
        let mesh = engine.add_mesh(&vertices, &indices)?;
        let tess_wires = Object {
            transform: Matrix4::identity(),
            mesh,
            material: line_material,
            texture,
        };

        // Quad wires
        let (vertices, indices) = wireframe(&obj, QuadMode::Keep)?;
        let mesh = engine.add_mesh(&vertices, &indices)?;
        let quad_wires = Object {
            transform: Matrix4::identity(),
            mesh,
            material: line_material,
            texture,
        };

        // Triangles
        let (vertices, indices) = triangles(&obj)?;
        let mesh = engine.add_mesh(&vertices, &indices)?;
        let tri_material = engine.add_material(UNLIT_VERT, UNLIT_FRAG, DrawType::Triangles)?;
        let tris = Object {
            transform: Matrix4::identity(),
            mesh,
            material: tri_material,
            texture,
        };

        Ok(Self {
            quad_wires,
            tess_wires,
            tris,
            time: 0.0,
        })
    }

    fn next_frame(&mut self, engine: &mut dyn Engine) -> Result<FramePacket> {
        // Update positions
        let rotate = Matrix4::from_euler_angles(0.0, self.time, 0.0);
        let spacing = Vector3::new(3., 0., 0.);
        self.tris.transform = Matrix4::new_translation(&spacing) * rotate;
        self.quad_wires.transform = rotate;
        self.tess_wires.transform = Matrix4::new_translation(&-spacing) * rotate;

        // Update time
        engine.update_time_value(self.time)?;
        self.time += 0.01;

        Ok(FramePacket {
            objects: vec![self.tris, self.quad_wires, self.tess_wires],
        })
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let obj_path = args.next().context("Requires OBJ path.")?;
    let texture_path = args.next().context("Requires texture path.")?;
    let vr = args.next().is_some();
    launch::<MyApp>(vr, (obj_path, texture_path))
}
