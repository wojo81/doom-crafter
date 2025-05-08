#![allow(dead_code)]

use three_d::*;

use std::f32::consts::PI;

pub fn generate_images(path: &str, name: &str, viewport: &Viewport, context: &Context, camera: &mut Camera) -> anyhow::Result<()> {
    let mut skin = Skin::load(image::open(&path)?, name, context);

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

    for frame_index in 'A' ..= 'W' {
        match frame_index {
            'A' | 'C' => skin.reset(),
            'B'  => skin.flap_right_arm_and_leg(),
            'D' => skin.flap_left_arm_and_leg(),
            'E' | 'F' => skin.punch(frame_index),
            'G' => skin.apply_red(),
            'H' ..= 'N' => skin.rotate_around(vec3(0.0, -11.0, 0.0), -Vec3::unit_z(), (90.0 / 7.0) * (frame_index.offset_from('H') + 1) as f32),
            'O' ..= 'W' => skin.rotate_around(vec3(0.0, -11.0, 0.0), -Vec3::unit_z(), (90.0 / 9.0) * (frame_index.offset_from('O') + 1) as f32),
            _ => unreachable!()
        }

        match frame_index {
            'A' ..= 'G' => for rotation in 1 ..= 8 {
                let pixels = RenderTarget::new(target_texture.as_color_target(None), depth_texture.as_depth_target())
                    .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 0.0, 1.0))
                    .render(&camera, skin.0.iter().flat_map(|p| &p.faces).map(|f| &f.model), &[])
                    .read_color();

                use three_d_asset::io::Serialize;

                three_d_asset::io::save(
                    &CpuTexture {
                        data: TextureData::RgbaU8(pixels),
                        width: viewport.width,
                        height: viewport.height,
                        ..Default::default()
                    }
                    .serialize(format!("out/{name}{frame_index}{rotation}.png"))?,
                )?;

                camera.rotate_around_with_fixed_up(Vec3::zero(), -PI / 4.0, 0.0);
            },
            'H' ..= 'W' => {
                let rotation = 0;

                let pixels = RenderTarget::new(target_texture.as_color_target(None), depth_texture.as_depth_target())
                    .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 0.0, 1.0))
                    .render(&camera, skin.0.iter().flat_map(|p| &p.faces).map(|f| &f.model), &[])
                    .read_color();

                use three_d_asset::io::Serialize;

                three_d_asset::io::save(
                    &CpuTexture {
                        data: TextureData::RgbaU8(pixels),
                        width: viewport.width,
                        height: viewport.height,
                        ..Default::default()
                    }
                    .serialize(format!("out/{name}{frame_index}{rotation}.png"))?,
                )?;
            },
            _ => unreachable!()
        }
    }

    Ok(())
}

struct Skin([Prism; 6]);

impl Skin {
    const HEAD: usize = 0;
    const BODY: usize = 1;
    const RIGHT_LEG: usize = 2;
    const RIGHT_ARM: usize = 3;
    const LEFT_LEG: usize = 4;
    const LEFT_ARM: usize = 5;

    fn load(atlas: image::DynamicImage, name: &str, context: &Context) -> Self {
        Self([
            Prism::load(&atlas, name.to_string() + "Head",      Patch::HEAD,       vec3(0.0, 11.0, 0.0),   context),
            Prism::load(&atlas, name.to_string() + "Body",      Patch::BODY,       vec3(0.0, 2.0, 0.0),    context),
            Prism::load(&atlas, name.to_string() + "RightLeg",  Patch::RIGHT_LEG,  vec3(-2.0, -10.0, 0.0), context),
            Prism::load(&atlas, name.to_string() + "RightArm",  Patch::RIGHT_ARM,  vec3(-6.0, 2.0, 0.0),   context),
            Prism::load(&atlas, name.to_string() + "LeftLeg",   Patch::LEFT_LEG,   vec3(2.0, -10.0, 0.0),  context),
            Prism::load(&atlas, name.to_string() + "LeftArm",   Patch::LEFT_ARM,   vec3(6.0, 2.0, 0.0),    context)
        ])
    }

    fn reset(&mut self) {
        for prism in self.0.iter_mut() {
            prism.set_transformation(Mat4::identity());
        }
    }

    fn flap_right_arm_and_leg(&mut self) {
        self.0[Self::RIGHT_ARM].rotate_around(vec3(-6.0, 4.0, 0.0), &[(Vec3::unit_x(), -30.0)]);
        self.0[Self::RIGHT_LEG].rotate_around(vec3(-6.0, -4.0, 0.0), &[(Vec3::unit_x(), 30.0)]);
        self.0[Self::LEFT_ARM].rotate_around(vec3(-6.0, 4.0, 0.0), &[(Vec3::unit_x(), 30.0)]);
        self.0[Self::LEFT_LEG].rotate_around(vec3(-6.0, -4.0, 0.0), &[(Vec3::unit_x(), -30.0)]);
    }

    fn flap_left_arm_and_leg(&mut self) {
        self.0[Self::RIGHT_ARM].rotate_around(vec3(-6.0, 4.0, 0.0), &[(Vec3::unit_x(), 30.0)]);
        self.0[Self::RIGHT_LEG].rotate_around(vec3(-6.0, -4.0, 0.0), &[(Vec3::unit_x(), -30.0)]);
        self.0[Self::LEFT_ARM].rotate_around(vec3(-6.0, 4.0, 0.0), &[(Vec3::unit_x(), -30.0)]);
        self.0[Self::LEFT_LEG].rotate_around(vec3(-6.0, -4.0, 0.0), &[(Vec3::unit_x(), 30.0)]);
    }

    fn punch(&mut self, frame_index: char) {
        self.reset();
        match frame_index {
            'E' => self.0[Self::RIGHT_ARM].rotate_around(vec3(-6.0, 4.0, 0.0), &[(Vec3::unit_x(), -40.0), (Vec3::unit_z(), -13.0)]),
            'F' => self.0[Self::RIGHT_ARM].rotate_around(vec3(-6.0, 6.0, 0.0), &[(Vec3::unit_x(), -80.0), (Vec3::unit_z(), -5.0)]),
            _ => unreachable!()
        }
    }

    fn apply_red(&mut self) {
        self.reset();
        for prism in self.0.iter_mut() {
            for face in prism.faces.iter_mut() {
                face.model.material.color = Srgba::new(255, 70, 70, 255);
            }
        }
    }

    fn rotate_around(&mut self, pivot: Vec3, axis: Vec3, angle: f32) {
        for prism in self.0.iter_mut() {
            let rotation = Mat4::from_translation(pivot)
                * Mat4::from_axis_angle(axis, degrees(angle))
                * Mat4::from_translation(-pivot);

            prism.set_transformation(rotation);
        }
    }
}

struct Prism {
    faces: [Face; 6],
    width: u32,
    height: u32,
    depth: u32,
    matrix: Mat4,
}

impl Prism {
    fn load(atlas: &image::DynamicImage, name: String, patch: Patch, translation: Vec3, context: &Context) -> Self {
        let matrix = Mat4::from_translation(translation);
        let mut prism = Self {
            faces: [
                Face::new_front(&name, &atlas, patch,  context),
                Face::new_right(&name, &atlas, patch, context),
                Face::new_back(&name, &atlas, patch, context),
                Face::new_left(&name, &atlas, patch, context),
                Face::new_top(&name, &atlas, patch, context),
                Face::new_bottom(&name, &atlas, patch, context)
            ], width: patch.width, height: patch.height, depth: patch.depth, matrix
        };
        for face in prism.faces.iter_mut() {
            face.model.set_transformation(matrix);
        }
        prism
    }

    fn set_transformation(&mut self, transformation: Mat4) {
        for face in self.faces.iter_mut() {
            face.model.set_transformation(transformation * self.matrix);
        }
    }

    fn rotate_around(&mut self, pivot: Vec3, axes_angles: &[(Vec3, f32)]) {
        let rotation = axes_angles
            .into_iter()
            .fold(Mat4::from_translation(pivot),
                |matrix, (axis, angle)| matrix * Mat4::from_axis_angle(*axis, degrees(*angle))
            )
            * Mat4::from_translation(-pivot);
        self.set_transformation(rotation);
    }
}

struct Face {
    model: Gm<Mesh, ColorMaterial>,

    width: u32,
    height: u32,
    depth: u32,
}

impl Face {
    fn new_front(name: &str, atlas: &image::DynamicImage, patch: Patch, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(-x, -y, z),
            vec3(x, -y, z),
            vec3(x, y, z),
            vec3(-x, y, z)
        ];
        Self::new(name, atlas, patch, positions, 0, context)
    }

    fn new_right(name: &str, atlas: &image::DynamicImage, patch: Patch, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(x, -y, z),
            vec3(x, -y, -z),
            vec3(x, y, -z),
            vec3(x, y, z)
        ];
        Self::new(name, atlas, patch.as_right(), positions, 1, context)
    }

    fn new_back(name: &str, atlas: &image::DynamicImage, patch: Patch, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(x, -y, -z),
            vec3(-x, -y, -z),
            vec3(-x, y, -z),
            vec3(x, y, -z)
        ];
        Self::new(name, atlas, patch.as_back(), positions, 2, context)
    }

    fn new_left(name: &str, atlas: &image::DynamicImage, patch: Patch, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(-x, -y, -z),
            vec3(-x, -y, z),
            vec3(-x, y, z),
            vec3(-x, y, -z)
        ];
        Self::new(name, atlas, patch.as_left(), positions, 2, context)
    }

    fn new_top(name: &str, atlas: &image::DynamicImage, patch: Patch, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(-x, y, z),
            vec3(x, y, z),
            vec3(x, y, -z),
            vec3(-x, y, -z)
        ];
        Self::new(name, atlas, patch.as_top(), positions, 2, context)
    }

    fn new_bottom(name: &str, atlas: &image::DynamicImage, patch: Patch, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(-x, -y, -z),
            vec3(x, -y, -z),
            vec3(x, -y, z),
            vec3(-x, -y, z)
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

trait LetterOffset {
    fn letter_offset(self) -> u32;
    fn offset_from(self, other: Self) -> u32;
}

impl LetterOffset for char {
    fn letter_offset(self) -> u32 {
        self as u32 - 65
    }

    fn offset_from(self, other: Self) -> u32 {
        self.letter_offset() - other.letter_offset()
    }
}