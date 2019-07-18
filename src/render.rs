use cgmath::{Deg, Matrix3, Matrix4, Vector3};
use glium::backend::Facade;
use glium::index::{NoIndices, PrimitiveType};
use glium::program::ProgramCreationInput;
use glium::texture::{ClientFormat, RawImage2d};
use glium::{
    implement_vertex, uniform, Depth, DepthTest, DrawParameters, Program, Surface, Texture2d,
    VertexBuffer,
};
use image::DynamicImage;
use std::borrow::Cow;
use std::collections::hash_map::{Entry, HashMap};
use std::io;

use crate::mod_dir::ModDir;

#[derive(Deserialize, Serialize)]
pub struct Map {
    pub bg_name: String,
    pub meshes: Vec<Mesh>,
}

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

#[derive(Clone, Copy)]
struct BgVertex {
    xy: [f32; 2],
    uv: [f32; 2],
}
implement_vertex!(BgVertex, xy, uv);

pub struct Scene {
    textures: Vec<Texture2d>,
    meshes: Vec<(usize, VertexBuffer<Vertex>)>,
    bg: VertexBuffer<BgVertex>,
}

impl Scene {
    pub fn new<F: Facade>(facade: &F, mod_dir: &ModDir, map: &str) -> Result<Self, io::Error> {
        let map = mod_dir.read_map(map)?;

        let mut textures_map = HashMap::new();
        let mut textures = Vec::new();
        let mut meshes = Vec::new();

        let bg_tex = mod_dir.read_bg(&map.bg_name)?;
        #[rustfmt::skip]
        let bg = VertexBuffer::new(facade, &[
            BgVertex { xy: [ 1.0, -1.0], uv: [1.0, 0.0] },
            BgVertex { xy: [-1.0, -1.0], uv: [0.0, 0.0] },
            BgVertex { xy: [ 1.0,  1.0], uv: [1.0, 1.0] },
            BgVertex { xy: [-1.0,  1.0], uv: [0.0, 1.0] },
        ]).unwrap();
        textures.push(prepare_texture(facade, bg_tex));

        textures_map.insert("", 1);
        textures.push(empty_tex(facade));

        for mesh in &map.meshes {
            let tex_id;

            match textures_map.entry(&mesh.texture) {
                Entry::Occupied(entry) => tex_id = *entry.get(),

                Entry::Vacant(entry) => {
                    let texture = prepare_texture(facade, mod_dir.read_tex(entry.key())?);

                    tex_id = textures.len();
                    textures.push(texture);
                    entry.insert(tex_id);
                }
            }

            let mut flat_triangles = Vec::new();
            for tri in &mesh.triangles {
                flat_triangles.extend(tri);
            }
            let buffer = VertexBuffer::new(facade, &flat_triangles).unwrap();

            meshes.push((tex_id, buffer));
        }

        Ok(Scene {
            textures,
            meshes,
            bg,
        })
    }

    pub fn small<F: Facade>(facade: &F) -> Self {
        #[rustfmt::skip]
        let triangle = [
            Vertex { xyz: [-163, 0,  -71], rgba: [182, 182, 182, 255], uv: [0, 0] },
            Vertex { xyz: [ 162, 0,  -71], rgba: [182, 182, 182, 255], uv: [0, 0] },
            Vertex { xyz: [ 152, 5, -151], rgba: [255, 255, 255, 255], uv: [0, 0] },
        ];
        let buffer = VertexBuffer::new(facade, &triangle).unwrap();

        let bg = VertexBuffer::new(facade, &[]).unwrap();

        Scene {
            textures: vec![empty_tex(facade)],
            meshes: vec![(0, buffer)],
            bg,
        }
    }
}

fn empty_tex<F: Facade>(facade: &F) -> Texture2d {
    let empty_image = RawImage2d::<u8> {
        data: Cow::Borrowed(&[255, 255, 255, 255]),
        width: 1,
        height: 1,
        format: ClientFormat::U8U8U8U8,
    };
    Texture2d::new(facade, empty_image).unwrap()
}

fn prepare_texture<F: Facade>(facade: &F, img: DynamicImage) -> Texture2d {
    let decoded = img.to_rgba();
    let dimensions = decoded.dimensions();
    let image = RawImage2d::from_raw_rgba_reversed(&decoded.into_raw(), dimensions);
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
    mesh_program: Program,
    mesh_params: DrawParameters<'static>,

    bg_program: Program,
    bg_params: DrawParameters<'static>,
}

impl Renderer {
    pub fn new<F: Facade>(facade: &F) -> Self {
        fn program_with_srgb<F: Facade>(facade: &F, vert: &str, frag: &str) -> Program {
            Program::new(
                facade,
                ProgramCreationInput::SourceCode {
                    vertex_shader: vert,
                    tessellation_control_shader: None,
                    tessellation_evaluation_shader: None,
                    geometry_shader: None,
                    fragment_shader: frag,
                    transform_feedback_varyings: None,
                    outputs_srgb: true,
                    uses_point_size: false,
                },
            )
            .unwrap()
        }

        const VERTEX_SHADER_SRC: &str = include_str!("render/vert.glsl");
        const FRAGMENT_SHADER_SRC: &str = include_str!("render/frag.glsl");

        let mesh_program = program_with_srgb(facade, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC);
        let mesh_params = DrawParameters {
            depth: Depth {
                test: DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };

        const BG_VERTEX: &str = "#version 140
            in vec2 xy;
            in vec2 uv;
            out vec2 tex_pos;
            void main() {
                gl_Position = vec4(xy, 0.0, 1.0);
                tex_pos = uv;
            }
        ";
        const BG_FRAG: &str = "#version 140
            in vec2 tex_pos;
            out vec4 color;
            uniform sampler2D tex;
            void main() {
                color = texture(tex, tex_pos);
            }
        ";

        let bg_program = program_with_srgb(facade, BG_VERTEX, BG_FRAG);
        let bg_params = DrawParameters::default();

        Renderer {
            mesh_program,
            mesh_params,
            bg_program,
            bg_params,
        }
    }

    pub fn render<S: Surface>(&self, surface: &mut S, scene: &Scene, cam: &Camera) {
        self.render_bg(surface, scene);
        self.render_meshes(surface, scene, cam);
    }

    pub fn render_bg<S: Surface>(&self, surface: &mut S, scene: &Scene) {
        let indices = NoIndices(PrimitiveType::TriangleStrip);
        let texture = &scene.textures[0];

        surface
            .draw(
                &scene.bg,
                &indices,
                &self.bg_program,
                &uniform!(tex: texture),
                &self.bg_params,
            )
            .unwrap();
    }

    pub fn render_meshes<S: Surface>(&self, surface: &mut S, scene: &Scene, cam: &Camera) {
        let indices = NoIndices(PrimitiveType::TrianglesList);
        let perspective: [[f32; 4]; 4] = cam.perspective(surface).into();

        for mesh in &scene.meshes {
            let texture = &scene.textures[mesh.0];

            surface
                .draw(
                    &mesh.1,
                    &indices,
                    &self.mesh_program,
                    &uniform!(perspective: perspective, tex: texture),
                    &self.mesh_params,
                )
                .unwrap();
        }
    }
}
