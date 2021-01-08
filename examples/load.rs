use anyhow::{Context, Result, bail};
use klystron::{
    runtime_3d::{launch, App},
    DrawType, Engine, FramePacket, Object, UNLIT_FRAG, UNLIT_VERT, Sampling,
};
use klystron_obj::{parse_obj, triangles, wireframe, QuadMode};
use nalgebra::{Matrix4, Vector3};
use std::fs::{File, read};
use std::io::BufReader;

struct MyApp {
    objects: Vec<Object>,
    time: f32,
}

impl App for MyApp {
    const NAME: &'static str = "MyApp";

    type Args = Vec<(String, String)>;

    fn new(engine: &mut dyn Engine, models: Self::Args) -> Result<Self> {
        let mut objects = Vec::new();

        for (obj_path, texture_path) in models {
            let file = BufReader::new(File::open(obj_path)?);
            let obj = parse_obj(file).context("Error parsing OBJ")?;

            // Read important image data
            let img = png::Decoder::new(File::open(texture_path)?);
            let (info, mut reader) = img.read_info()?;
            assert!(info.color_type == png::ColorType::RGBA);
            assert!(info.bit_depth == png::BitDepth::Eight);
            let mut img_buffer = vec![0; info.buffer_size()];
            assert_eq!(info.buffer_size(), (info.width * info.height * 4) as _);
            reader.next_frame(&mut img_buffer)?;
            let texture = engine.add_texture(&img_buffer, info.width, Sampling::Nearest)?;

            // Triangles
            let (vertices, indices) = triangles(&obj)?;
            let mesh = engine.add_mesh(&vertices, &indices)?;
            let tri_material = engine.add_material(
                &read("./shaders/unlit.vert.spv")?, 
                &read("./shaders/unlit.frag.spv")?, 
                DrawType::Triangles
            )?;
            objects.push(Object {
                transform: Matrix4::identity(),
                mesh,
                material: tri_material,
                texture,
            });
        }

        Ok(Self {
            objects,
            time: 0.0,
        })
    }

    fn next_frame(&mut self, engine: &mut dyn Engine) -> Result<FramePacket> {
        // Update time
        engine.update_time_value(self.time)?;
        self.time += 0.01;

        Ok(FramePacket {
            objects: self.objects.clone(),
        })
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);

    let vr = match args.next().as_ref().map(|s| s.as_str()) {
        Some("vr") => true,
        Some("win") => false,
        _ => bail!("Requires mode either \"vr\" or \"win\""),
    };

    let mut models = Vec::new();

    loop {
        let obj_path = match args.next() {
            Some(o) => o,
            None => break,
        };
        let texture_path = args.next().context("Requires texture path.")?;
        models.push((obj_path, texture_path))
    }
    launch::<MyApp>(vr, models)
}
