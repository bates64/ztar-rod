use cgmath::{Deg, Matrix3, Matrix4, Vector3};
use glium::backend::Facade;
use glium::index::{NoIndices, PrimitiveType};
use glium::program::ProgramCreationInput;
use glium::texture::{ClientFormat, RawImage2d};
use glium::{
    implement_vertex, uniform, Depth, DepthTest, DrawParameters, Program, Surface, Texture2d,
    VertexBuffer,
};
use std::borrow::Cow;
use std::collections::hash_map::{Entry, HashMap};
use std::fs::{self, File};
use std::io::BufReader;

#[derive(Deserialize, Serialize)]
pub struct Map(pub Vec<Mesh>);

#[derive(Deserialize, Serialize)]
pub struct Mesh {
    pub texture: String, // TODO: maybe Option<String>
    pub triangles: Vec<[Vertex; 3]>,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct Vertex {
    pub xyz: [i32; 3],
    pub rgba: [u8; 4],
    pub uv: [i16; 2],
}

implement_vertex!(Vertex, xyz, rgba, uv);

pub struct Scene {
    textures: Vec<Texture2d>,
    meshes: Vec<(usize, VertexBuffer<Vertex>)>,
}

impl Scene {
    pub fn new<F: Facade>(facade: &F, mut map: Map) -> Self {
        let mut textures_map = HashMap::new();
        let mut textures = Vec::new();
        let mut meshes = Vec::new();

        for mut mesh in map.0.drain(..) {
            let tex_id;

            match textures_map.entry(mesh.texture) {
                Entry::Occupied(entry) => tex_id = *entry.get(),
                Entry::Vacant(entry) => {
                    tex_id = textures.len();
                    textures.push(locate_texture(facade, entry.key()));
                    entry.insert(tex_id);
                }
            }

            let mut flat_triangles = Vec::new();
            for tri in mesh.triangles.drain(..) {
                flat_triangles.extend(&tri);
            }
            let buffer = VertexBuffer::new(facade, &flat_triangles).unwrap();

            meshes.push((tex_id, buffer));
        }

        Scene { textures, meshes }
    }

    pub fn small<F: Facade>(facade: &F) -> Self {
        #[rustfmt::skip]
        let triangle = [
            Vertex { xyz: [-163, 0,  -71], rgba: [182, 182, 182, 255], uv: [0, 0] },
            Vertex { xyz: [ 162, 0,  -71], rgba: [182, 182, 182, 255], uv: [0, 0] },
            Vertex { xyz: [ 152, 5, -151], rgba: [255, 255, 255, 255], uv: [0, 0] },
        ];

        let buffer = VertexBuffer::new(facade, &triangle).unwrap();

        Scene {
            textures: vec![],
            meshes: vec![(0, buffer)],
        }
    }
}

fn locate_texture<F: Facade>(facade: &F, tex: &str) -> Texture2d {
    let image = if tex == "" {
        // a plain, blank texture: just one white pixel
        RawImage2d {
            data: Cow::Borrowed(&[255, 255, 255, 255]),
            width: 1,
            height: 1,
            format: ClientFormat::U8U8U8U8,
        }
    } else {
        let area = tex.split('_').nth(0).unwrap();
        let proper_location = format!("mod/img/tex/{}.png", &tex);
        let stolen_location = format!("steal/image/texture/{}_tex/{}.png", &area, &tex);

        let reader = File::open(&proper_location)
            .or_else(|_| {
                fs::copy(&stolen_location, &proper_location)?;
                File::open(&proper_location)
            })
            .map(BufReader::new)
            .unwrap_or_else(|_| {
                panic!(
                    r#"could not locate texture "{}"; copy the Star Rod directory "image" to "steal""#,
                    tex
                )
            });
        let decoded = image::load(reader, image::PNG).unwrap().to_rgba();
        let dimensions = decoded.dimensions();
        RawImage2d::from_raw_rgba_reversed(&decoded.into_raw(), dimensions)
    };

    Texture2d::new(facade, image).unwrap()
}

#[derive(Debug)]
pub struct Camera {
    pos: Vector3<f32>,
    yaw: Deg<f32>,
    pitch: Deg<f32>,
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            pos: Vector3::new(0.0, 200.0, 1000.0),
            yaw: Deg(0.0),
            pitch: Deg(-20.0),
        }
    }
}

impl Camera {
    pub fn pan(&mut self, amt: Deg<f32>) {
        self.yaw += amt;
    }
    pub fn tilt(&mut self, amt: Deg<f32>) {
        let clamp_top = f32::min(self.pitch.0 + amt.0, 90.0);
        let clamp_bot = f32::max(clamp_top, -90.0);
        self.pitch.0 = clamp_bot;
    }

    pub fn dolly(&mut self, amt: f32) {
        self.pos += amt
            * Matrix3::from_angle_y(self.yaw)
            * Matrix3::from_angle_x(self.pitch)
            * Vector3::unit_z();
    }
    pub fn truck(&mut self, amt: f32) {
        self.pos += amt * Matrix3::from_angle_y(self.yaw) * Vector3::unit_x();
    }

    fn perspective<S: Surface>(&self, surface: &S) -> Matrix4<f32> {
        let (width, height) = surface.get_dimensions();
        let aspect = width as f32 / height as f32;

        cgmath::perspective(Deg(45.0), aspect, 0.1, 1_000_000.0)
            * Matrix4::from_angle_x(-self.pitch)
            * Matrix4::from_angle_y(-self.yaw)
            * Matrix4::from_translation(-self.pos)
    }
}

pub struct Renderer {
    program: Program,
    params: DrawParameters<'static>,
}

impl Renderer {
    pub fn new<F: Facade>(facade: &F) -> Self {
        const VERTEX_SHADER_SRC: &str = include_str!("render/vert.glsl");
        const FRAGMENT_SHADER_SRC: &str = include_str!("render/frag.glsl");

        let program = Program::new(
            facade,
            ProgramCreationInput::SourceCode {
                vertex_shader: VERTEX_SHADER_SRC,
                tessellation_control_shader: None,
                tessellation_evaluation_shader: None,
                geometry_shader: None,
                fragment_shader: FRAGMENT_SHADER_SRC,
                transform_feedback_varyings: None,
                outputs_srgb: true,
                uses_point_size: false,
            },
        )
        .unwrap();

        let params = DrawParameters {
            depth: Depth {
                test: DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };

        Renderer { program, params }
    }

    pub fn render<S: Surface>(&self, surface: &mut S, scene: &Scene, cam: &Camera) {
        let indices = NoIndices(PrimitiveType::TrianglesList);
        let perspective: [[f32; 4]; 4] = cam.perspective(surface).into();

        for mesh in &scene.meshes {
            let texture = &scene.textures[mesh.0];

            surface
                .draw(
                    &mesh.1,
                    &indices,
                    &self.program,
                    &uniform!(perspective: perspective, tex: texture),
                    &self.params,
                )
                .unwrap();
        }
    }
}
