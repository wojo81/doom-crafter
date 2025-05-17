#![allow(dead_code)]

use image::GenericImageView;
use three_d::*;

use std::f32::consts::PI;

pub fn render_images(path: &str, sprite: &str, viewport: &Viewport, context: &Context, camera: &mut Camera) -> anyhow::Result<()> {
    let mut skin = Skin::load(image::open(&path)?, sprite, context);

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

    let ambient = AmbientLight::new(context, 0.6, Srgba::WHITE);

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
                    .render(&camera, skin.limbs.iter().flat_map(|p| &p.faces).map(|f| &f.model)
                        .chain(skin.trim.iter().flat_map(|p| &p.texels).map(|t| &t.model)), &[&ambient])
                    .read_color();

                use three_d_asset::io::Serialize;

                three_d_asset::io::save(
                    &CpuTexture {
                        data: TextureData::RgbaU8(pixels),
                        width: viewport.width,
                        height: viewport.height,
                        ..Default::default()
                    }
                    .serialize(format!("temp/{sprite}{frame_index}{rotation}.png"))?,
                )?;

                camera.rotate_around_with_fixed_up(Vec3::zero(), -PI / 4.0, 0.0);
            },
            'H' ..= 'W' => {
                let rotation = 0;

                let pixels = RenderTarget::new(target_texture.as_color_target(None), depth_texture.as_depth_target())
                    .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 0.0, 1.0))
                    .render(&camera, skin.limbs.iter().flat_map(|p| &p.faces).map(|f| &f.model)
                        .chain(skin.trim.iter().flat_map(|p| &p.texels).map(|t| &t.model)), &[&ambient])
                    .read_color();

                use three_d_asset::io::Serialize;

                three_d_asset::io::save(
                    &CpuTexture {
                        data: TextureData::RgbaU8(pixels),
                        width: viewport.width,
                        height: viewport.height,
                        ..Default::default()
                    }
                    .serialize(format!("temp/{sprite}{frame_index}{rotation}.png"))?,
                )?;
            },
            _ => unreachable!()
        }
    }

    Ok(())
}

struct Skin {
    limbs: [Limb; 6],
    trim: [Trim; 6]
}

impl Skin {
    const HEAD: usize = 0;
    const TORSO: usize = 1;
    const RIGHT_LEG: usize = 2;
    const RIGHT_ARM: usize = 3;
    const LEFT_LEG: usize = 4;
    const LEFT_ARM: usize = 5;

    const HELMET: usize = 0;
    const SHIRT: usize = 1;
    const RIGHT_PANTS: usize = 2;
    const RIGHT_SLEEVE: usize = 3;
    const LEFT_PANTS: usize = 4;
    const LEFT_SLEEVE: usize = 5;

    const RIGHT_JOINT: Vec3 = vec3(-6.0, -4.0, 0.0);
    const RIGHT_SHOULDER: Vec3 = vec3(-6.0, 4.0, 0.0);
    const LEFT_JOINT: Vec3 = vec3(-6.0, -4.0, 0.0);
    const LEFT_SHOULDER: Vec3 = vec3(-6.0, 4.0, 0.0);

    fn load(atlas: image::DynamicImage, name: &str, context: &Context) -> Self {
        Self {
            limbs: [
                Limb::load(&atlas, name.to_string() + "Head",        Patch::HEAD,         vec3(0.0, 11.0, 0.0),   context),
                Limb::load(&atlas, name.to_string() + "Torso",       Patch::TORSO,        vec3(0.0, 2.0, 0.0),    context),
                Limb::load(&atlas, name.to_string() + "RightLeg",    Patch::RIGHT_LEG,    vec3(-2.0, -10.0, 0.0), context),
                Limb::load(&atlas, name.to_string() + "RightArm",    Patch::RIGHT_ARM,    vec3(-6.0, 2.0, 0.0),   context),
                Limb::load(&atlas, name.to_string() + "LeftLeg",     Patch::LEFT_LEG,     vec3(2.0, -10.0, 0.0),  context),
                Limb::load(&atlas, name.to_string() + "LeftArm",     Patch::LEFT_ARM,     vec3(6.0, 2.0, 0.0),    context),
            ],
            trim: [
                Trim::load(&atlas, name.to_string() + "Helmet",      Patch::HELMET,       vec3(0.0, 11.0, 0.0),   context),
                Trim::load(&atlas, name.to_string() + "Shirt",       Patch::SHIRT,        vec3(0.0, 2.0, 0.0),    context),
                Trim::load(&atlas, name.to_string() + "RightPants",  Patch::RIGHT_PANTS,  vec3(-2.0, -10.0, 0.0), context),
                Trim::load(&atlas, name.to_string() + "RightSleeve", Patch::RIGHT_SLEEVE, vec3(-6.0, 2.0, 0.0),   context),
                Trim::load(&atlas, name.to_string() + "LeftPants",   Patch::LEFT_PANTS,   vec3(2.0, -10.0, 0.0),  context),
                Trim::load(&atlas, name.to_string() + "LeftSleeve",  Patch::LEFT_SLEEVE,  vec3(6.0, 2.0, 0.0),    context),
            ],
        }
    }

    fn reset(&mut self) {
        for limb in self.limbs.iter_mut() {
            limb.set_transformation(Mat4::identity());
        }
        for trim in self.trim.iter_mut() {
            trim.set_transformation(Mat4::identity());
        }
    }

    fn flap_right_arm_and_leg(&mut self) {
        self.limbs[Self::RIGHT_LEG].rotate_around(Self::RIGHT_JOINT, &[(Vec3::unit_x(), 30.0)]);
        self.limbs[Self::RIGHT_ARM].rotate_around(Self::RIGHT_SHOULDER, &[(Vec3::unit_x(), -30.0)]);
        self.limbs[Self::LEFT_LEG].rotate_around(Self::LEFT_JOINT, &[(Vec3::unit_x(), -30.0)]);
        self.limbs[Self::LEFT_ARM].rotate_around(Self::LEFT_SHOULDER, &[(Vec3::unit_x(), 30.0)]);

        self.trim[Self::RIGHT_PANTS].rotate_around(Self::RIGHT_JOINT, &[(Vec3::unit_x(), 30.0)]);
        self.trim[Self::RIGHT_SLEEVE].rotate_around(Self::RIGHT_SHOULDER, &[(Vec3::unit_x(), -30.0)]);
        self.trim[Self::LEFT_PANTS].rotate_around(Self::LEFT_JOINT, &[(Vec3::unit_x(), -30.0)]);
        self.trim[Self::LEFT_SLEEVE].rotate_around(Self::LEFT_SHOULDER, &[(Vec3::unit_x(), 30.0)]);
    }

    fn flap_left_arm_and_leg(&mut self) {
        self.limbs[Self::RIGHT_ARM].rotate_around(Self::RIGHT_SHOULDER, &[(Vec3::unit_x(), 30.0)]);
        self.limbs[Self::RIGHT_LEG].rotate_around(Self::RIGHT_JOINT, &[(Vec3::unit_x(), -30.0)]);
        self.limbs[Self::LEFT_ARM].rotate_around(Self::LEFT_SHOULDER, &[(Vec3::unit_x(), -30.0)]);
        self.limbs[Self::LEFT_LEG].rotate_around(Self::LEFT_JOINT, &[(Vec3::unit_x(), 30.0)]);

        self.trim[Self::RIGHT_SLEEVE].rotate_around(Self::RIGHT_SHOULDER, &[(Vec3::unit_x(), 30.0)]);
        self.trim[Self::RIGHT_PANTS].rotate_around(Self::RIGHT_JOINT, &[(Vec3::unit_x(), -30.0)]);
        self.trim[Self::LEFT_SLEEVE].rotate_around(Self::LEFT_SHOULDER, &[(Vec3::unit_x(), -30.0)]);
        self.trim[Self::LEFT_PANTS].rotate_around(Self::LEFT_JOINT, &[(Vec3::unit_x(), 30.0)]);
    }

    fn punch(&mut self, frame_index: char) {
        self.reset();
        let axes_angles = match frame_index {
            'E' => [(Vec3::unit_x(), -40.0), (Vec3::unit_z(), -13.0)],
            'F' => [(Vec3::unit_x(), -80.0), (Vec3::unit_z(), -5.0)],
            _ => unreachable!()
        };
        self.limbs[Self::RIGHT_ARM].rotate_around(Self::RIGHT_SHOULDER, &axes_angles);
        self.trim[Self::RIGHT_ARM].rotate_around(Self::RIGHT_SHOULDER, &axes_angles);
    }

    fn apply_red(&mut self) {
        self.reset();
        for limb in self.limbs.iter_mut() {
            for face in limb.faces.iter_mut() {
                face.model.material.color = Srgba::new(255, 70, 70, 255);
            }
        }
        for trim in self.trim.iter_mut() {
            for texel in trim.texels.iter_mut() {
                texel.model.material.color = Srgba::new(255, 70, 70, 255);
            }
        }
    }

    fn rotate_around(&mut self, pivot: Vec3, axis: Vec3, angle: f32) {
        for limb in self.limbs.iter_mut() {
            let rotation = Mat4::from_translation(pivot)
                * Mat4::from_axis_angle(axis, degrees(angle))
                * Mat4::from_translation(-pivot);

            limb.set_transformation(rotation);
        }
        for trim in self.trim.iter_mut() {
            let rotation = Mat4::from_translation(pivot)
                * Mat4::from_axis_angle(axis, degrees(angle))
                * Mat4::from_translation(-pivot);

            trim.set_transformation(rotation);
        }
    }
}

struct Limb {
    faces: [Face; 6],
    matrix: Mat4,
}

impl Limb {
    fn load(atlas: &image::DynamicImage, name: String, patch: Patch, translation: Vec3, context: &Context) -> Self {
        let matrix = Mat4::from_translation(translation);
        let mut limb = Self {
            faces: [
                Face::new_front(&name, &atlas, patch,  context),
                Face::new_right(&name, &atlas, patch, context),
                Face::new_back(&name, &atlas, patch, context),
                Face::new_left(&name, &atlas, patch, context),
                Face::new_top(&name, &atlas, patch, context),
                Face::new_bottom(&name, &atlas, patch, context)
            ],  matrix
        };
        limb.set_transformation(Mat4::identity());
        limb
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
        Self::new(name, atlas, patch.as_left(), positions, 3, context)
    }

    fn new_top(name: &str, atlas: &image::DynamicImage, patch: Patch, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(-x, y, z),
            vec3(x, y, z),
            vec3(x, y, -z),
            vec3(-x, y, -z)
        ];
        Self::new(name, atlas, patch.as_top(), positions, 4, context)
    }

    fn new_bottom(name: &str, atlas: &image::DynamicImage, patch: Patch, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(-x, -y, -z),
            vec3(x, -y, -z),
            vec3(x, -y, z),
            vec3(-x, -y, z)
        ];
        Self::new(name, atlas, patch.as_bottom(), positions, 5, context)
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

        let material = ColorMaterial {
            texture: Some(Texture2DRef::from_cpu_texture(&context, &texture)),
            render_states: RenderStates { cull: Cull::None, ..Default::default() },
            ..Default::default()
        };

        let model = Gm::new(Mesh::new(context, &mesh), material);

        Self { model }
    }
}

struct Trim {
    texels: Vec<Texel>,
    matrix: Mat4,
}

impl Trim {
    const UNIT: f32 = 1.1;
    const SCALE: Vec3 = vec3(Self::UNIT, Self::UNIT, Self::UNIT);

    fn load(atlas: &image::DynamicImage, name: String, patch: Patch, translation: Vec3, context: &Context) -> Self {
        let mut texels = vec![];

        const ALPHA_MIN: u8 = 255;

        for px in 0 .. patch.width {
            for py in 0 .. patch.height {
                let tx = (px as i32 - patch.width as i32 / 2) as f32 * Self::UNIT;
                let ty = (patch.height as i32 / 2 - 1 - py as i32) as f32 * Self::UNIT;
                let tz = Self::UNIT / 2.0 * patch.depth as f32;
                let pixel = atlas.get_pixel(px + patch.x, py + patch.y).0;

                if pixel[3] >= ALPHA_MIN {
                    texels.push(Texel::new(&name, pixel, vec3(tx, ty, tz), Direction::Front, context));
                }
            }
        }

        let right_patch = patch.as_right();
        for px in 0 .. right_patch.width {
            for py in 0 .. right_patch.height {
                let tx = Self::UNIT / 2.0 * right_patch.depth as f32;
                let ty = (right_patch.height as i32 / 2 - 1 - py as i32) as f32 * Self::UNIT;
                let tz = (right_patch.width as i32 / 2 - 1 - px as i32) as f32 * Self::UNIT;
                let pixel = atlas.get_pixel(px + right_patch.x, py + right_patch.y).0;

                if pixel[3] >= ALPHA_MIN {
                    texels.push(Texel::new(&name, pixel, vec3(tx, ty, tz), Direction::Right, context));
                }
            }
        }

        let back_patch = patch.as_back();
        for px in 0 .. back_patch.width {
            for py in 0 .. back_patch.height {
                let tx = (back_patch.width as i32 / 2 - 1 - px as i32) as f32 * Self::UNIT;
                let ty = (back_patch.height as i32 / 2 - 1 - py as i32) as f32 * Self::UNIT;
                let tz = -Self::UNIT / 2.0 * back_patch.depth as f32;
                let pixel = atlas.get_pixel(px + back_patch.x, py + back_patch.y).0;

                if pixel[3] >= ALPHA_MIN {
                    texels.push(Texel::new(&name, pixel, vec3(tx, ty, tz), Direction::Back, context));
                }
            }
        }

        let left_patch = patch.as_left();
        for px in 0 .. left_patch.width {
            for py in 0 .. left_patch.height {
                let tx = -Self::UNIT / 2.0 * left_patch.depth as f32;
                let ty = (left_patch.height as i32 / 2 - 1 - py as i32) as f32 * Self::UNIT;
                let tz = (px as i32 - left_patch.width as i32 / 2) as f32 * Self::UNIT;
                let pixel = atlas.get_pixel(px + left_patch.x, py + left_patch.y).0;

                if pixel[3] >= ALPHA_MIN {
                    texels.push(Texel::new(&name, pixel, vec3(tx, ty, tz), Direction::Left, context));
                }
            }
        }

        let top_patch = patch.as_top();
        for px in 0 .. top_patch.width {
            for py in 0 .. top_patch.height {
                let tx = (px as i32 - top_patch.width as i32 / 2) as f32 * Self::UNIT;
                let ty = Self::UNIT / 2.0 * top_patch.depth as f32;
                let tz = (py as i32 - top_patch.height as i32 / 2) as f32 * Self::UNIT;
                let pixel = atlas.get_pixel(px + top_patch.x, py + top_patch.y).0;

                if pixel[3] >= ALPHA_MIN {
                    texels.push(Texel::new(&name, pixel, vec3(tx, ty, tz), Direction::Top, context));
                }
            }
        }

        let bottom_patch = patch.as_bottom();
        for px in 0 .. bottom_patch.width {
            for py in 0 .. bottom_patch.height {
                let tx = (px as i32 - bottom_patch.width as i32 / 2) as f32 * Self::UNIT;
                let ty = -Self::UNIT / 2.0 * bottom_patch.depth as f32;
                let tz = (bottom_patch.height as i32 / 2 - 1 - py as i32) as f32 * Self::UNIT;
                let pixel = atlas.get_pixel(px + bottom_patch.x, py + bottom_patch.y).0;

                if pixel[3] >= ALPHA_MIN {
                    texels.push(Texel::new(&name, pixel, vec3(tx, ty, tz), Direction::Bottom, context));
                }
            }
        }

        Self { texels, matrix: Mat4::from_translation(translation) }
    }

    fn set_transformation(&mut self, transformation: Mat4) {
        for texel in self.texels.iter_mut() {
            texel.model.set_transformation(transformation * self.matrix);
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

struct Texel {
    model: Gm<Mesh, ColorMaterial>,
}

impl Texel {
    fn new(name: &str, pixel: [u8; 4], position: Vec3, direction: Direction, context: &Context) -> Self {
        use Direction::*;
        let positions = match direction {
            Front => vec![
                vec3(position.x, position.y, position.z),
                vec3(position.x + Trim::UNIT, position.y, position.z),
                vec3(position.x + Trim::UNIT, position.y + Trim::UNIT, position.z),
                vec3(position.x, position.y + Trim::UNIT, position.z)
            ],
            Right => vec![
                vec3(position.x, position.y, position.z + Trim::UNIT),
                vec3(position.x, position.y, position.z),
                vec3(position.x, position.y + Trim::UNIT, position.z),
                vec3(position.x, position.y + Trim::UNIT, position.z + Trim::UNIT)
            ],
            Back => vec![
                vec3(position.x + Trim::UNIT, position.y, position.z),
                vec3(position.x, position.y, position.z),
                vec3(position.x, position.y + Trim::UNIT, position.z),
                vec3(position.x + Trim::UNIT, position.y + Trim::UNIT, position.z)
            ],
            Left => vec![
                vec3(position.x, position.y, position.z),
                vec3(position.x, position.y, position.z + Trim::UNIT),
                vec3(position.x, position.y + Trim::UNIT, position.z + Trim::UNIT),
                vec3(position.x, position.y + Trim::UNIT, position.z)
            ],
            Top => vec![
                vec3(position.x, position.y, position.z + Trim::UNIT),
                vec3(position.x + Trim::UNIT, position.y, position.z + Trim::UNIT),
                vec3(position.x + Trim::UNIT, position.y, position.z),
                vec3(position.x, position.y, position.z)
            ],
            Bottom => vec![
                vec3(position.x, position.y, position.z),
                vec3(position.x + Trim::UNIT, position.y, position.z),
                vec3(position.x + Trim::UNIT, position.y, position.z + Trim::UNIT),
                vec3(position.x, position.y, position.z + Trim::UNIT)
            ]
        };

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

        let data = TextureData::RgbaU8(vec![pixel]);
        let texture = CpuTexture {
            name: format!("{name}"),
            data,
            width: 1,
            height: 1,
            min_filter: Interpolation::Nearest,
            mag_filter: Interpolation::Nearest,
            ..Default::default()
        };

        let material = ColorMaterial {
            texture: Some(Texture2DRef::from_cpu_texture(&context, &texture)),
            render_states: RenderStates { cull: Cull::None, ..Default::default() },
            ..Default::default()
        };

        let model = Gm::new(Mesh::new(context, &mesh), material);

        Self { model }
    }
}

enum Direction {
    Front, Right, Back, Left, Top, Bottom
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
    const TORSO: Patch = Patch::new(20, 20, 8, 12, 4);
    const RIGHT_LEG: Patch = Patch::new(4, 20, 4, 12, 4);
    const RIGHT_ARM: Patch = Patch::new(44, 20, 4, 12, 4);
    const LEFT_LEG: Patch = Patch::new(20, 52, 4, 12, 4);
    const LEFT_ARM: Patch = Patch::new(36, 52, 4, 12, 4);

    const HELMET: Patch = Patch::new(40, 8, 8, 8, 8);
    const SHIRT: Patch = Patch::new(20, 36, 8, 12, 4);
    const RIGHT_PANTS: Patch = Patch::new(4, 36, 4, 12, 4);
    const RIGHT_SLEEVE: Patch = Patch::new(44, 36, 4, 12, 4);
    const LEFT_PANTS: Patch = Patch::new(4, 52, 4, 12, 4);
    const LEFT_SLEEVE: Patch = Patch::new(52, 52, 4, 12, 4);

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