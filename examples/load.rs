use anyhow::{Context, Result};
use klystron::{
    runtime_3d::{launch, App},
    DrawType, Engine, FramePacket, Object, UNLIT_FRAG, UNLIT_VERT,
};
use klystron_obj::{parse_obj, triangles, wireframe, QuadMode};
use nalgebra::{Matrix4, Vector3};
use std::fs::{File, read};
use std::io::BufReader;

struct MyApp {
    tris: Object,
    time: f32,
}

impl App for MyApp {
    const NAME: &'static str = "MyApp";

    type Args = (String, String);

    fn new(engine: &mut dyn Engine, (obj_path, texture_path): Self::Args) -> Result<Self> {
        let file = BufReader::new(File::open(obj_path)?);
        let obj = parse_obj(file).context("Error parsing OBJ")?;

        // Read important image data
        let img = png::Decoder::new(File::open(texture_path)?);
        let (info, mut reader) = img.read_info()?;
        assert!(info.color_type == png::ColorType::RGB);
        assert!(info.bit_depth == png::BitDepth::Eight);
        let mut img_buffer = vec![0; info.buffer_size()];
        assert_eq!(info.buffer_size(), (info.width * info.height * 3) as _);
        reader.next_frame(&mut img_buffer)?;
        let texture = engine.add_texture(&img_buffer, info.width)?;

        // Triangles
        let (vertices, indices) = triangles(&obj)?;
        let mesh = engine.add_mesh(&vertices, &indices)?;
        let tri_material = engine.add_material(
            &read("./shaders/unlit.vert.spv")?, 
            &read("./shaders/unlit.frag.spv")?, 
            DrawType::Triangles
        )?;
        let tris = Object {
            transform: Matrix4::identity(),
            mesh,
            material: tri_material,
            texture,
        };

        Ok(Self {
            tris,
            time: 0.0,
        })
    }

    fn next_frame(&mut self, engine: &mut dyn Engine) -> Result<FramePacket> {
        // Update positions
        let rotate = Matrix4::from_euler_angles(0.0, self.time, 0.0);
        self.tris.transform = rotate;

        // Update time
        engine.update_time_value(self.time)?;
        self.time += 0.01;

        Ok(FramePacket {
            objects: vec![self.tris],
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
