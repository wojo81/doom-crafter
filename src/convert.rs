#![allow(dead_code)]

use std::f32::consts::PI;

use three_d::*;

pub fn convert_all(paths_and_names: impl Iterator<Item = (String, String)>) -> anyhow::Result<()> {
    let viewport = Viewport::new_at_origo(128, 128);
    let context = HeadlessContext::new()?;
    let mut camera = Camera::new_perspective(
        viewport,
        vec3(0.0, 0.0, 2.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(60.0),
        0.1,
        100.0,
    );

    for (path, name) in paths_and_names {
        convert(path, name, &viewport, &context, &mut camera)?;
    }

    Ok(())
}

fn convert(path: String, name: String, viewport: &Viewport, context: &Context, camera: &mut Camera) -> anyhow::Result<()> {
    let atlas = image::open(&path)?;

    let depth = -35.0;

    let mut prisms = [
        Prism::load(&atlas, name.clone() + "Head",      Patch::HEAD,       vec3(0.0, 11.0, depth),    vec3(0.0, 0.0, 0.0),  context),
        Prism::load(&atlas, name.clone() + "Body",      Patch::BODY,       vec3(0.0, 2.0, depth),     vec3(0.0, 0.0, 0.0),  context),
        Prism::load(&atlas, name.clone() + "RightLeg",  Patch::RIGHT_LEG,  vec3(-2.0, -6.0, depth),   vec3(0.0, -4.0, 0.0), context),
        Prism::load(&atlas, name.clone() + "RightArm",  Patch::RIGHT_ARM,  vec3(-6.0, 6.0, depth),    vec3(0.0, -4.0, 0.0), context),
        Prism::load(&atlas, name.clone() + "LeftLeg",   Patch::LEFT_LEG,   vec3(2.0, -6.0, depth),    vec3(0.0, -4.0, 0.0), context),
        Prism::load(&atlas, name.clone() + "LeftArm",   Patch::LEFT_ARM,   vec3(6.0, 6.0, depth),     vec3(0.0, -4.0, 0.0), context)
    ];

    let mut target_texture = Texture2D::new_empty::<[u8; 4]>(
        &context,
        viewport.width,
        viewport.height,
        Interpolation::Nearest,
        Interpolation::Nearest,
        None,
        Wrapping::ClampToEdge,
        Wrapping::ClampToEdge,
    );

    let mut depth_texture = DepthTexture2D::new::<f32>(
        &context,
        viewport.width,
        viewport.height,
        Wrapping::ClampToEdge,
        Wrapping::ClampToEdge,
    );

    for frame_index in 0..8 {
        prisms[2].set_transformation(Mat4::from_angle_x(degrees(-15.0 * frame_index as f32)));

        let pixels = RenderTarget::new(target_texture.as_color_target(None), depth_texture.as_depth_target())
            .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 0.0, 1.0))
            .render(&camera, prisms.iter().flat_map(|p| &p.faces).map(|f| &f.model), &[])
            .read_color();

        use three_d_asset::io::Serialize;

        three_d_asset::io::save(
            &CpuTexture {
                data: TextureData::RgbaU8(pixels),
                width: viewport.width,
                height: viewport.height,
                ..Default::default()
            }
            .serialize(format!("out/{name}-{frame_index}.png"))?,
        )?;

        camera.rotate_around_with_fixed_up(vec3(0.0, 0.0, depth), PI / 4.0, 0.0);
    }

    Ok(())
}

struct Prism {
    faces: [Face; 6],
    width: u32,
    height: u32,
    depth: u32,
    matrix: Mat4,
}

impl Prism {
    fn load(atlas: &image::DynamicImage, name: String, patch: Patch, translation: Vec3, center: Vec3, context: &Context) -> Self {
        let matrix = Mat4::from_translation(translation);
        let mut prism = Self {
            faces: [
                Face::new_front(&name, &atlas, patch, center,  context),
                Face::new_right(&name, &atlas, patch, center, context),
                Face::new_back(&name, &atlas, patch, center, context),
                Face::new_left(&name, &atlas, patch, center, context),
                Face::new_top(&name, &atlas, patch, center, context),
                Face::new_bottom(&name, &atlas, patch, center, context)
            ], width: patch.width, height: patch.height, depth: patch.depth, matrix
        };
        for face in prism.faces.iter_mut() {
            face.model.set_transformation(matrix);
        }
        prism
    }

    fn set_transformation(&mut self, transformation: Mat4) {
        for face in self.faces.iter_mut() {
            face.model.set_transformation(self.matrix * transformation);
        }
    }
}

struct Face {
    model: Gm<Mesh, ColorMaterial>,

    width: u32,
    height: u32,
    depth: u32,
}

impl Face {
    fn new_front(name: &str, atlas: &image::DynamicImage, patch: Patch, center: Vec3, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(-x, -y, z) + center,
            vec3(x, -y, z) + center,
            vec3(x, y, z) + center,
            vec3(-x, y, z) + center
        ];
        Self::new(name, atlas, patch, positions, 0, context)
    }

    fn new_right(name: &str, atlas: &image::DynamicImage, patch: Patch, center: Vec3, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(x, -y, z) + center,
            vec3(x, -y, -z) + center,
            vec3(x, y, -z) + center,
            vec3(x, y, z) + center
        ];
        Self::new(name, atlas, patch.as_right(), positions, 1, context)
    }

    fn new_back(name: &str, atlas: &image::DynamicImage, patch: Patch, center: Vec3, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(x, -y, -z) + center,
            vec3(-x, -y, -z) + center,
            vec3(-x, y, -z) + center,
            vec3(x, y, -z) + center
        ];
        Self::new(name, atlas, patch.as_back(), positions, 2, context)
    }

    fn new_left(name: &str, atlas: &image::DynamicImage, patch: Patch, center: Vec3, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(-x, -y, -z) + center,
            vec3(-x, -y, z) + center,
            vec3(-x, y, z) + center,
            vec3(-x, y, -z) + center
        ];
        Self::new(name, atlas, patch.as_left(), positions, 2, context)
    }

    fn new_top(name: &str, atlas: &image::DynamicImage, patch: Patch, center: Vec3, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(-x, y, z) + center,
            vec3(x, y, z) + center,
            vec3(x, y, -z) + center,
            vec3(-x, y, -z) + center
        ];
        Self::new(name, atlas, patch.as_top(), positions, 2, context)
    }

    fn new_bottom(name: &str, atlas: &image::DynamicImage, patch: Patch, center: Vec3, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(-x, -y, -z) + center,
            vec3(x, -y, -z) + center,
            vec3(x, -y, z) + center,
            vec3(-x, -y, z) + center
        ];
        Self::new(name, atlas, patch.as_bottom(), positions, 2, context)
    }

    fn new(name: &str, atlas: &image::DynamicImage, patch: Patch, positions: Vec<Vec3>, index: u8, context: &Context) -> Self {
        let indices = vec![0, 1, 2, 2, 3, 0];

        let uvs = vec![
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 0.0),
        ];

        let mesh = CpuMesh {
            positions: Positions::F32(positions),
            indices: Indices::U8(indices),
            uvs: Some(uvs),
            ..Default::default()
        };

        let sub_image = image::imageops::crop_imm(atlas, patch.x, patch.y, patch.width, patch.height).to_image();
        let data = TextureData::RgbaU8(
            sub_image.into_raw()
                .chunks(4)
                .map(|p| [p[0], p[1], p[2], p[3]])
                .collect()
        );
        let texture = CpuTexture {
            name: format!("{name}_{index}"),
            data,
            width: patch.width,
            height: patch.height,
            min_filter: Interpolation::Nearest,
            mag_filter: Interpolation::Nearest,
            ..Default::default()
        };

        let model = Gm::new(
            Mesh::new(context, &mesh),
            ColorMaterial {
                texture: Some(Texture2DRef::from_cpu_texture(&context, &texture)),
                render_states: RenderStates { cull: Cull::Back, ..Default::default() },
                ..Default::default()
            },
        );

        Self { model, width: patch.width, height: patch.height, depth: patch.depth }
    }
}

#[derive(Clone, Copy)]
struct Patch {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    depth: u32,
}

impl Patch {
    const HEAD: Patch = Patch::new(8, 8, 8, 8, 8);
    const BODY: Patch = Patch::new(20, 20, 8, 12, 4);
    const LEFT_LEG: Patch = Patch::new(20, 52, 4, 12, 4);
    const LEFT_ARM: Patch = Patch::new(36, 52, 4, 12, 4);
    const RIGHT_LEG: Patch = Patch::new(4, 20, 4, 12, 4);
    const RIGHT_ARM: Patch = Patch::new(44, 20, 4, 12, 4);

    const fn new(x: u32, y: u32, width: u32, height: u32, depth: u32) -> Self {
        Self { x, y, width, height, depth }
    }

    fn half_size(&self) -> Vec3 {
        vec3(self.width as f32 / 2.0, self.height as f32 / 2.0, self.depth as f32 / 2.0)
    }

    fn as_right(&self) -> Self {
        Self::new(self.x + self.width, self.y, self.depth, self.height, self.width)
    }

    fn as_back(&self) -> Self {
        Self::new(self.x + self.width + self.depth, self.y, self.width, self.height, self.depth)
    }

    fn as_left(&self) -> Self {
        Self::new(self.x - self.depth, self.y, self.depth, self.height, self.width)
    }

    fn as_top(&self) -> Self {
        Self::new(self.x, self.y - self.depth, self.width, self.depth, self.height)
    }

    fn as_bottom(&self) -> Self {
        Self::new(self.x + self.width, self.y - self.depth, self.width, self.depth, self.height)
    }
}