use crate::converting::{Rendering, SpritePrefix};
use anyhow::Context as WithContext;
use image::{DynamicImage, GenericImageView};
use std::{f32::consts::PI, path::Path};
use three_d::*;

pub struct TargetTexture {
    texture: Texture2D,
    depth: DepthTexture2D,
}

impl TargetTexture {
    pub fn new(rendering: &Rendering) -> Self {
        let texture = Texture2D::new_empty::<[u8; 4]>(
            &rendering.context,
            rendering.viewport.width,
            rendering.viewport.height,
            Interpolation::Nearest,
            Interpolation::Nearest,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        let depth = DepthTexture2D::new::<f32>(
            &rendering.context,
            rendering.viewport.width,
            rendering.viewport.height,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );

        Self { texture, depth }
    }
}

fn create_subdir(rendered_dir: &Path, subdir: &str, index: usize) -> anyhow::Result<()> {
    std::fs::create_dir(rendered_dir.join(&format!("{subdir}{index}")))
        .with_context(|| format!("subdirectory {subdir}{index}"))
}

pub fn render_skin(
    atlas: &DynamicImage,
    rendered_dir: &Path,
    sprite_prefix: &str,
    rendering: &mut Rendering,
    index: usize,
) -> anyhow::Result<()> {
    let sprite = sprite_prefix.to_skin_sprite();
    let mut target = TargetTexture::new(&rendering);
    let mut skin = Skin::load(atlas, &sprite, &rendering.context);

    create_subdir(rendered_dir, "sprites", index)?;
    for frame_index in 'A'..='W' {
        match frame_index {
            'A' | 'C' => skin.reset(),
            'B' => skin.flap_right_arm_and_leg(),
            'D' => skin.flap_left_arm_and_leg(),
            'E' | 'F' => skin.punch(frame_index),
            'G' => skin.apply_red(70),
            'H'..='N' => skin.rotate_around(
                vec3(0.0, -11.0, 0.0),
                -Vec3::unit_z(),
                (90.0 / 7.0) * (frame_index.offset_from('H') + 1) as f32,
            ),
            'O'..='W' => skin.rotate_around(
                vec3(0.0, -11.0, 0.0),
                -Vec3::unit_z(),
                (90.0 / 9.0) * (frame_index.offset_from('O') + 1) as f32,
            ),
            _ => unreachable!(),
        }

        match frame_index {
            'A'..='G' => {
                for rotation in 1..=8 {
                    render_skin_frame(
                        &skin,
                        &sprite,
                        frame_index,
                        rotation,
                        rendering,
                        &mut target,
                        rendered_dir,
                        index,
                    )?;
                    rendering
                        .camera
                        .rotate_around_with_fixed_up(Vec3::zero(), -PI / 4.0, 0.0);
                }
            }
            'H'..='W' => {
                let rotation = 0;

                render_skin_frame(
                    &skin,
                    &sprite,
                    frame_index,
                    rotation,
                    rendering,
                    &mut target,
                    rendered_dir,
                    index,
                )?;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}

pub fn render_skin_with_crouch(
    atlas: &DynamicImage,
    rendered_dir: &Path,
    sprite_prefix: &str,
    rendering: &mut Rendering,
    index: usize,
) -> anyhow::Result<()> {
    render_skin(atlas, rendered_dir, sprite_prefix, rendering, index)?;

    let sprite = sprite_prefix.to_crouched_skin_sprite();
    let mut target = TargetTexture::new(&rendering);
    let mut skin = Skin::load_crouched(atlas, &sprite, &rendering.context);

    create_subdir(rendered_dir, "crouch-sprites", index)?;
    for frame_index in 'A'..='G' {
        match frame_index {
            'A' | 'C' => skin.reset_crouched(),
            'B' => skin.flap_right_arm_and_leg_crouched(),
            'D' => skin.flap_left_arm_and_leg_crouched(),
            'E' | 'F' => skin.punch_crouched(frame_index),
            'G' => skin.apply_red_crouched(70),
            _ => unreachable!(),
        }

        for rotation in 1..=8 {
            render_crouched_skin_frame(
                &skin,
                &sprite,
                frame_index,
                rotation,
                rendering,
                &mut target,
                rendered_dir,
                index,
            )?;
            rendering
                .camera
                .rotate_around_with_fixed_up(Vec3::zero(), -PI / 4.0, 0.0);
        }
    }

    Ok(())
}

pub fn render_mugshot(
    atlas: &DynamicImage,
    rendered_dir: &Path,
    sprite_prefix: &str,
    rendering: &mut Rendering,
    index: usize,
) -> anyhow::Result<()> {
    rendering.camera.translate(Vec3::unit_z() * 10.0);
    let sprite = sprite_prefix.to_mugshot_sprite();
    let mut target = TargetTexture::new(&rendering);
    let mut head = Limb::load(
        atlas,
        "head".into(),
        Patch::HEAD,
        Vec3::zero(),
        &rendering.context,
    );
    let mut helmet = Trim::load(
        atlas,
        "helmet".into(),
        Patch::HELMET,
        Vec3::zero(),
        &rendering.context,
    );
    let suffixes = ["DEAD", "EVL", "GOD", "KILL", "OUCH", "ST", "TL", "TR"];

    create_subdir(rendered_dir, "mugshot", index)?;
    for suffix in suffixes {
        match suffix {
            "DEAD" => {
                let color = Srgba::new(64, 64, 64, 255);
                head.apply_color(color);
                helmet.apply_color(color);
            }
            "EVL" | "KILL" => {
                let angle = -15.0;
                head.rotate_around(Vec3::zero(), &[(Vec3::unit_x(), angle)]);
                helmet.rotate_around(Vec3::zero(), &[(Vec3::unit_x(), angle)]);
            }
            "GOD" => {
                let color = Srgba::new(255, 215, 0, 255);
                head.apply_color(color);
                helmet.apply_color(color);
            }
            "OUCH" => {
                let angle = 15.0;
                head.rotate_around(Vec3::zero(), &[(Vec3::unit_x(), angle)]);
                helmet.rotate_around(Vec3::zero(), &[(Vec3::unit_x(), angle)]);
            }
            "ST" => (),
            "TL" => {
                let angle = -30.0;
                head.rotate_around(Vec3::zero(), &[(Vec3::unit_y(), angle)]);
                helmet.rotate_around(Vec3::zero(), &[(Vec3::unit_y(), angle)]);
            }
            "TR" => {
                let angle = 30.0;
                head.rotate_around(Vec3::zero(), &[(Vec3::unit_y(), angle)]);
                helmet.rotate_around(Vec3::zero(), &[(Vec3::unit_y(), angle)]);
            }
            _ => unreachable!(),
        }

        match suffix {
            "DEAD" | "GOD" => render_mugshot_frame(
                &head,
                &helmet,
                &sprite,
                suffix,
                0,
                rendering,
                &mut target,
                rendered_dir,
                index,
            )?,
            "EVL" | "KILL" | "OUCH" => {
                for i in 0..5 {
                    let saturation = 255 / (i + 1);
                    head.apply_red(saturation);
                    helmet.apply_red(saturation);
                    render_mugshot_frame(
                        &head,
                        &helmet,
                        &sprite,
                        suffix,
                        i,
                        rendering,
                        &mut target,
                        rendered_dir,
                        index,
                    )?;
                }
                let axis_angle = [(Vec3::unit_x(), 0.0)];
                head.rotate_around(Vec3::zero(), &axis_angle);
                helmet.rotate_around(Vec3::zero(), &axis_angle);
            }
            "ST" => {
                for x in 0..5 {
                    let saturation = 255 / (x + 1);
                    head.apply_red(saturation);
                    helmet.apply_red(saturation);
                    for y in 0..3 {
                        let angle = match y {
                            0 => 15.0,
                            1 => 0.0,
                            2 => -15.0,
                            _ => unreachable!(),
                        };
                        let axis_angle = [(Vec3::unit_y(), angle)];
                        head.rotate_around(Vec3::zero(), &axis_angle);
                        helmet.rotate_around(Vec3::zero(), &axis_angle);
                        render_mugshot_frame(
                            &head,
                            &helmet,
                            &sprite,
                            &format!("{suffix}{x}"),
                            y,
                            rendering,
                            &mut target,
                            rendered_dir,
                            index,
                        )?;
                    }
                }
            }
            "TL" | "TR" => {
                for i in 0..5 {
                    let saturation = 255 / (i + 1);
                    head.apply_red(saturation);
                    helmet.apply_red(saturation);
                    render_mugshot_frame(
                        &head,
                        &helmet,
                        &sprite,
                        &format!("{suffix}{i}"),
                        0,
                        rendering,
                        &mut target,
                        rendered_dir,
                        index,
                    )?;
                }
            }
            _ => unreachable!(),
        }
    }
    Ok(())
}

pub fn render_fist(
    atlas: &DynamicImage,
    rendered_dir: &Path,
    sprite_prefix: &str,
    rendering: &mut Rendering,
    index: usize,
) -> anyhow::Result<()> {
    let sprite = sprite_prefix.to_fist_sprite();
    let mut target = TargetTexture::new(&rendering);
    let position = Vec3::unit_x() * 3.5;
    let mut arm = Limb::load(
        atlas,
        "arm".into(),
        Patch::RIGHT_ARM,
        position,
        &rendering.context,
    );
    let mut sleeve = Trim::load(
        atlas,
        "arm".into(),
        Patch::RIGHT_SLEEVE,
        position,
        &rendering.context,
    );

    let delta = 28.0;

    rendering.camera.rotate_around(Vec3::zero(), PI, 0.0);
    rendering.camera.translate(Vec3::unit_z() * -delta);

    create_subdir(rendered_dir, "fist", index)?;
    for frame_index in 'A'..='I' {
        let rotation = [
            (
                Vec3::unit_x(),
                match frame_index {
                    'A' => 130.0,
                    'B' => 135.0,
                    'C' => 125.0,
                    'D' => 115.0,
                    'E' => 105.0,
                    'F' => 110.0,
                    'G' => 115.0,
                    'H' => 120.0,
                    'I' => 125.0,
                    _ => unreachable!(),
                },
            ),
            (Vec3::unit_y(), -115.0),
        ];
        let pivot = Vec3::zero();
        arm.rotate_around(pivot, &rotation);
        sleeve.rotate_around(pivot, &rotation);

        render_fist_frame(
            &arm,
            &sleeve,
            &sprite,
            frame_index,
            rendering,
            &mut target,
            rendered_dir,
            index,
        )?;
    }

    Ok(())
}

fn render_skin_frame(
    skin: &Skin,
    sprite: &str,
    frame_index: char,
    rotation: i32,
    rendering: &Rendering,
    target: &mut TargetTexture,
    rendered_dir: &Path,
    index: usize,
) -> anyhow::Result<()> {
    let pixels = RenderTarget::new(
        target.texture.as_color_target(None),
        target.depth.as_depth_target(),
    )
    .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 0.0, 1.0))
    .render(
        &rendering.camera,
        skin.limbs
            .iter()
            .flat_map(|p| &p.faces)
            .map(|f| &f.model)
            .chain(skin.trim.iter().flat_map(|p| &p.texels).map(|t| &t.model)),
        &[],
    )
    .read_color();

    use three_d_asset::io::Serialize;

    three_d_asset::io::save(
        &CpuTexture {
            data: TextureData::RgbaU8(pixels),
            width: rendering.viewport.width,
            height: rendering.viewport.height,
            ..Default::default()
        }
        .serialize(
            rendered_dir
                .join(format!("sprites{index}"))
                .join(format!("{sprite}{frame_index}{rotation}.png")),
        )?,
    )?;

    Ok(())
}

fn render_crouched_skin_frame(
    skin: &Skin,
    sprite: &str,
    frame_index: char,
    rotation: i32,
    rendering: &mut Rendering,
    target: &mut TargetTexture,
    rendered_dir: &Path,
    index: usize,
) -> anyhow::Result<()> {
    let pixels = RenderTarget::new(
        target.texture.as_color_target(None),
        target.depth.as_depth_target(),
    )
    .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 0.0, 1.0))
    .render(
        &rendering.camera,
        skin.limbs
            .iter()
            .flat_map(|p| &p.faces)
            .map(|f| &f.model)
            .chain(skin.trim.iter().flat_map(|p| &p.texels).map(|t| &t.model)),
        &[],
    )
    .read_color();

    use three_d_asset::io::Serialize;

    three_d_asset::io::save(
        &CpuTexture {
            data: TextureData::RgbaU8(pixels),
            width: rendering.viewport.width,
            height: rendering.viewport.height,
            ..Default::default()
        }
        .serialize(
            rendered_dir
                .join(format!("crouch-sprites{index}"))
                .join(format!("{sprite}{frame_index}{rotation}.png")),
        )?,
    )?;

    Ok(())
}

fn render_mugshot_frame(
    head: &Limb,
    helmet: &Trim,
    sprite: &str,
    suffix: &str,
    frame_index: u8,
    rendering: &Rendering,
    target: &mut TargetTexture,
    rendered_dir: &Path,
    index: usize,
) -> anyhow::Result<()> {
    let pixels = RenderTarget::new(
        target.texture.as_color_target(None),
        target.depth.as_depth_target(),
    )
    .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 0.0, 1.0))
    .render(
        &rendering.camera,
        head.faces
            .iter()
            .map(|f| &f.model)
            .chain(helmet.texels.iter().map(|t| &t.model)),
        &[],
    )
    .read_color();

    use three_d_asset::io::Serialize;

    three_d_asset::io::save(
        &CpuTexture {
            data: TextureData::RgbaU8(pixels),
            width: rendering.viewport.width,
            height: rendering.viewport.height,
            ..Default::default()
        }
        .serialize(
            rendered_dir
                .join(format!("mugshot{index}"))
                .join(format!("{sprite}{suffix}{frame_index}.png")),
        )?,
    )?;

    Ok(())
}

fn render_fist_frame(
    arm: &Limb,
    sleeve: &Trim,
    sprite: &str,
    frame_index: char,
    rendering: &Rendering,
    target: &mut TargetTexture,
    rendered_dir: &Path,
    index: usize,
) -> anyhow::Result<()> {
    let sprite = sprite.replace('\\', "^");
    let pixels = RenderTarget::new(
        target.texture.as_color_target(None),
        target.depth.as_depth_target(),
    )
    .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 0.0, 1.0))
    .render(
        &rendering.camera,
        arm.faces
            .iter()
            .map(|f| &f.model)
            .chain(sleeve.texels.iter().map(|t| &t.model)),
        &[],
    )
    .read_color();

    use three_d_asset::io::Serialize;

    three_d_asset::io::save(
        &CpuTexture {
            data: TextureData::RgbaU8(pixels),
            width: rendering.viewport.width,
            height: rendering.viewport.height,
            ..Default::default()
        }
        .serialize(
            rendered_dir
                .join(format!("fist{index}"))
                .join(format!("{sprite}{frame_index}0.png")),
        )?,
    )?;
    Ok(())
}

struct Skin {
    limbs: [Limb; 6],
    trim: [Trim; 6],
}

#[allow(dead_code)]
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

    const CROUCH_RIGHT_SHOULDER: Vec3 = vec3(-6.0, 2.0, 2.0);
    const CROUCH_LEFT_SHOULDER: Vec3 = vec3(-6.0, 2.0, 2.0);

    const CROUCH_PIVOT: Vec3 = vec3(0.0, 2.0, 0.0);
    const CROUCH_ROTATION: [(Vec3, f32); 1] = [(vec3(1.0, 0.0, 0.0), 30.0)];
    const CROUCH_TORSO_OFFSET: Vec3 = vec3(0.0, 0.0, 1.8);
    const CROUCH_HEAD_OFFSET: Vec3 = vec3(0.0, -2.0, 3.0);
    const CROUCH_SLEEVE_OFFSET: Vec3 = vec3(0.0, 0.0, -2.0);

    fn load(atlas: &image::DynamicImage, name: &str, context: &Context) -> Self {
        if atlas.get_pixel(55, 20).0[3] < 10 {
            Self {
                limbs: [
                    Limb::load(
                        atlas,
                        name.to_string() + "Head",
                        Patch::HEAD,
                        vec3(0.0, 11.0, 0.0),
                        context,
                    ),
                    Limb::load(
                        atlas,
                        name.to_string() + "Torso",
                        Patch::TORSO,
                        vec3(0.0, 2.0, 0.0),
                        context,
                    ),
                    Limb::load(
                        atlas,
                        name.to_string() + "RightLeg",
                        Patch::RIGHT_LEG,
                        vec3(-2.0, -10.0, 0.0),
                        context,
                    ),
                    Limb::load(
                        atlas,
                        name.to_string() + "RightArm",
                        Patch::SLIM_RIGHT_ARM,
                        vec3(-5.5, 2.0, 0.0),
                        context,
                    ),
                    Limb::load(
                        atlas,
                        name.to_string() + "LeftLeg",
                        Patch::LEFT_LEG,
                        vec3(2.0, -10.0, 0.0),
                        context,
                    ),
                    Limb::load(
                        atlas,
                        name.to_string() + "LeftArm",
                        Patch::SLIM_LEFT_ARM,
                        vec3(5.5, 2.0, 0.0),
                        context,
                    ),
                ],
                trim: [
                    Trim::load(
                        atlas,
                        name.to_string() + "Helmet",
                        Patch::HELMET,
                        vec3(0.0, 11.0, 0.0),
                        context,
                    ),
                    Trim::load(
                        atlas,
                        name.to_string() + "Shirt",
                        Patch::SHIRT,
                        vec3(0.0, 2.0, 0.0),
                        context,
                    ),
                    Trim::load(
                        atlas,
                        name.to_string() + "RightPants",
                        Patch::RIGHT_PANTS,
                        vec3(-2.0, -10.0, 0.0),
                        context,
                    ),
                    Trim::load(
                        atlas,
                        name.to_string() + "RightSleeve",
                        Patch::SLIM_RIGHT_SLEEVE,
                        vec3(-6.0, 2.0, 0.0),
                        context,
                    ),
                    Trim::load(
                        atlas,
                        name.to_string() + "LeftPants",
                        Patch::LEFT_PANTS,
                        vec3(2.0, -10.0, 0.0),
                        context,
                    ),
                    Trim::load(
                        atlas,
                        name.to_string() + "LeftSleeve",
                        Patch::SLIM_LEFT_SLEEVE,
                        vec3(6.0, 2.0, 0.0),
                        context,
                    ),
                ],
            }
        } else {
            Self {
                limbs: [
                    Limb::load(
                        atlas,
                        name.to_string() + "Head",
                        Patch::HEAD,
                        vec3(0.0, 11.0, 0.0),
                        context,
                    ),
                    Limb::load(
                        atlas,
                        name.to_string() + "Torso",
                        Patch::TORSO,
                        vec3(0.0, 2.0, 0.0),
                        context,
                    ),
                    Limb::load(
                        atlas,
                        name.to_string() + "RightLeg",
                        Patch::RIGHT_LEG,
                        vec3(-2.0, -10.0, 0.0),
                        context,
                    ),
                    Limb::load(
                        atlas,
                        name.to_string() + "RightArm",
                        Patch::RIGHT_ARM,
                        vec3(-6.0, 2.0, 0.0),
                        context,
                    ),
                    Limb::load(
                        atlas,
                        name.to_string() + "LeftLeg",
                        Patch::LEFT_LEG,
                        vec3(2.0, -10.0, 0.0),
                        context,
                    ),
                    Limb::load(
                        atlas,
                        name.to_string() + "LeftArm",
                        Patch::LEFT_ARM,
                        vec3(6.0, 2.0, 0.0),
                        context,
                    ),
                ],
                trim: [
                    Trim::load(
                        atlas,
                        name.to_string() + "Helmet",
                        Patch::HELMET,
                        vec3(0.0, 11.0, 0.0),
                        context,
                    ),
                    Trim::load(
                        atlas,
                        name.to_string() + "Shirt",
                        Patch::SHIRT,
                        vec3(0.0, 2.0, 0.0),
                        context,
                    ),
                    Trim::load(
                        atlas,
                        name.to_string() + "RightPants",
                        Patch::RIGHT_PANTS,
                        vec3(-2.0, -10.0, 0.0),
                        context,
                    ),
                    Trim::load(
                        atlas,
                        name.to_string() + "RightSleeve",
                        Patch::RIGHT_SLEEVE,
                        vec3(-6.0, 2.0, 0.0),
                        context,
                    ),
                    Trim::load(
                        atlas,
                        name.to_string() + "LeftPants",
                        Patch::LEFT_PANTS,
                        vec3(2.0, -10.0, 0.0),
                        context,
                    ),
                    Trim::load(
                        atlas,
                        name.to_string() + "LeftSleeve",
                        Patch::LEFT_SLEEVE,
                        vec3(6.0, 2.0, 0.0),
                        context,
                    ),
                ],
            }
        }
    }

    fn load_crouched(atlas: &image::DynamicImage, name: &str, context: &Context) -> Skin {
        let mut skin = Self::load(atlas, name, context);
        let torso = &mut skin.limbs[Skin::TORSO];
        let shirt = &mut skin.trim[Skin::SHIRT];

        torso.matrix = Mat4::from_translation(Self::CROUCH_TORSO_OFFSET) * torso.matrix;
        shirt.matrix = Mat4::from_translation(Self::CROUCH_TORSO_OFFSET) * shirt.matrix;

        skin.limbs[Skin::LEFT_ARM].matrix =
            Mat4::from_translation(Self::CROUCH_TORSO_OFFSET) * skin.limbs[Skin::LEFT_ARM].matrix;
        skin.limbs[Skin::RIGHT_ARM].matrix =
            Mat4::from_translation(Self::CROUCH_TORSO_OFFSET) * skin.limbs[Skin::RIGHT_ARM].matrix;
        skin.trim[Skin::LEFT_SLEEVE].matrix =
            Mat4::from_translation(Self::CROUCH_TORSO_OFFSET + Self::CROUCH_SLEEVE_OFFSET)
                * skin.limbs[Skin::LEFT_SLEEVE].matrix;
        skin.trim[Skin::RIGHT_SLEEVE].matrix =
            Mat4::from_translation(Self::CROUCH_TORSO_OFFSET + Self::CROUCH_SLEEVE_OFFSET)
                * skin.limbs[Skin::RIGHT_SLEEVE].matrix;

        skin.limbs[Skin::HEAD].matrix =
            Mat4::from_translation(Self::CROUCH_HEAD_OFFSET) * skin.limbs[Skin::HEAD].matrix;
        skin.trim[Skin::HELMET].matrix =
            Mat4::from_translation(Self::CROUCH_HEAD_OFFSET) * skin.trim[Skin::HELMET].matrix;
        skin.limbs[Skin::HEAD].set_transformation(Mat4::identity());
        skin.trim[Skin::HELMET].set_transformation(Mat4::identity());

        skin
    }

    fn reset(&mut self) {
        for limb in self.limbs.iter_mut() {
            limb.set_transformation(Mat4::identity());
        }
        for trim in self.trim.iter_mut() {
            trim.set_transformation(Mat4::identity());
        }
    }

    fn reset_crouched(&mut self) {
        for (i, limb) in self.limbs.iter_mut().enumerate() {
            match i {
                Self::TORSO | Self::RIGHT_ARM | Self::LEFT_ARM => {
                    limb.rotate_around(Self::CROUCH_PIVOT, &Self::CROUCH_ROTATION)
                }
                _ => limb.set_transformation(Mat4::identity()),
            }
        }
        for (i, limb) in self.trim.iter_mut().enumerate() {
            match i {
                Self::SHIRT | Self::RIGHT_SLEEVE | Self::LEFT_SLEEVE => {
                    limb.rotate_around(Self::CROUCH_PIVOT, &Self::CROUCH_ROTATION)
                }
                _ => limb.set_transformation(Mat4::identity()),
            }
        }
    }

    fn flap_right_arm_and_leg(&mut self) {
        self.limbs[Self::RIGHT_LEG].rotate_around(Self::RIGHT_JOINT, &[(Vec3::unit_x(), 30.0)]);
        self.limbs[Self::RIGHT_ARM].rotate_around(Self::RIGHT_SHOULDER, &[(Vec3::unit_x(), -30.0)]);
        self.limbs[Self::LEFT_LEG].rotate_around(Self::LEFT_JOINT, &[(Vec3::unit_x(), -30.0)]);
        self.limbs[Self::LEFT_ARM].rotate_around(Self::LEFT_SHOULDER, &[(Vec3::unit_x(), 30.0)]);

        self.trim[Self::RIGHT_PANTS].rotate_around(Self::RIGHT_JOINT, &[(Vec3::unit_x(), 30.0)]);
        self.trim[Self::RIGHT_SLEEVE]
            .rotate_around(Self::RIGHT_SHOULDER, &[(Vec3::unit_x(), -30.0)]);
        self.trim[Self::LEFT_PANTS].rotate_around(Self::LEFT_JOINT, &[(Vec3::unit_x(), -30.0)]);
        self.trim[Self::LEFT_SLEEVE].rotate_around(Self::LEFT_SHOULDER, &[(Vec3::unit_x(), 30.0)]);
    }

    fn flap_left_arm_and_leg(&mut self) {
        self.limbs[Self::RIGHT_ARM].rotate_around(Self::RIGHT_SHOULDER, &[(Vec3::unit_x(), 30.0)]);
        self.limbs[Self::RIGHT_LEG].rotate_around(Self::RIGHT_JOINT, &[(Vec3::unit_x(), -30.0)]);
        self.limbs[Self::LEFT_ARM].rotate_around(Self::LEFT_SHOULDER, &[(Vec3::unit_x(), -30.0)]);
        self.limbs[Self::LEFT_LEG].rotate_around(Self::LEFT_JOINT, &[(Vec3::unit_x(), 30.0)]);

        self.trim[Self::RIGHT_SLEEVE]
            .rotate_around(Self::RIGHT_SHOULDER, &[(Vec3::unit_x(), 30.0)]);
        self.trim[Self::RIGHT_PANTS].rotate_around(Self::RIGHT_JOINT, &[(Vec3::unit_x(), -30.0)]);
        self.trim[Self::LEFT_SLEEVE].rotate_around(Self::LEFT_SHOULDER, &[(Vec3::unit_x(), -30.0)]);
        self.trim[Self::LEFT_PANTS].rotate_around(Self::LEFT_JOINT, &[(Vec3::unit_x(), 30.0)]);
    }

    fn flap_right_arm_and_leg_crouched(&mut self) {
        self.limbs[Self::RIGHT_LEG].rotate_around(Self::RIGHT_JOINT, &[(Vec3::unit_x(), 30.0)]);
        self.limbs[Self::RIGHT_ARM]
            .rotate_around(Self::CROUCH_RIGHT_SHOULDER, &[(Vec3::unit_x(), -30.0)]);
        self.limbs[Self::LEFT_LEG].rotate_around(Self::LEFT_JOINT, &[(Vec3::unit_x(), -30.0)]);
        self.limbs[Self::LEFT_ARM]
            .rotate_around(Self::CROUCH_LEFT_SHOULDER, &[(Vec3::unit_x(), 30.0)]);

        self.trim[Self::RIGHT_PANTS].rotate_around(Self::RIGHT_JOINT, &[(Vec3::unit_x(), 30.0)]);
        self.trim[Self::RIGHT_SLEEVE]
            .rotate_around(Self::CROUCH_RIGHT_SHOULDER, &[(Vec3::unit_x(), -30.0)]);
        self.trim[Self::LEFT_PANTS].rotate_around(Self::LEFT_JOINT, &[(Vec3::unit_x(), -30.0)]);
        self.trim[Self::LEFT_SLEEVE]
            .rotate_around(Self::CROUCH_LEFT_SHOULDER, &[(Vec3::unit_x(), 30.0)]);
    }

    fn flap_left_arm_and_leg_crouched(&mut self) {
        self.limbs[Self::RIGHT_ARM]
            .rotate_around(Self::CROUCH_RIGHT_SHOULDER, &[(Vec3::unit_x(), 30.0)]);
        self.limbs[Self::RIGHT_LEG].rotate_around(Self::RIGHT_JOINT, &[(Vec3::unit_x(), -30.0)]);
        self.limbs[Self::LEFT_ARM]
            .rotate_around(Self::CROUCH_LEFT_SHOULDER, &[(Vec3::unit_x(), -30.0)]);
        self.limbs[Self::LEFT_LEG].rotate_around(Self::LEFT_JOINT, &[(Vec3::unit_x(), 30.0)]);

        self.trim[Self::RIGHT_SLEEVE]
            .rotate_around(Self::CROUCH_RIGHT_SHOULDER, &[(Vec3::unit_x(), 30.0)]);
        self.trim[Self::RIGHT_PANTS].rotate_around(Self::RIGHT_JOINT, &[(Vec3::unit_x(), -30.0)]);
        self.trim[Self::LEFT_SLEEVE]
            .rotate_around(Self::CROUCH_LEFT_SHOULDER, &[(Vec3::unit_x(), -30.0)]);
        self.trim[Self::LEFT_PANTS].rotate_around(Self::LEFT_JOINT, &[(Vec3::unit_x(), 30.0)]);
    }

    fn punch(&mut self, frame_index: char) {
        self.reset();
        let axes_angles = match frame_index {
            'E' => [(Vec3::unit_x(), -40.0), (Vec3::unit_z(), -13.0)],
            'F' => [(Vec3::unit_x(), -80.0), (Vec3::unit_z(), -5.0)],
            _ => unreachable!(),
        };
        self.limbs[Self::RIGHT_ARM].rotate_around(Self::RIGHT_SHOULDER, &axes_angles);
        self.trim[Self::RIGHT_ARM].rotate_around(Self::RIGHT_SHOULDER, &axes_angles);
    }

    fn punch_crouched(&mut self, frame_index: char) {
        self.reset_crouched();
        let axes_angles = match frame_index {
            'E' => [(Vec3::unit_x(), -40.0), (Vec3::unit_z(), -13.0)],
            'F' => [(Vec3::unit_x(), -80.0), (Vec3::unit_z(), -5.0)],
            _ => unreachable!(),
        };
        self.limbs[Self::RIGHT_ARM].rotate_around(Self::CROUCH_RIGHT_SHOULDER, &axes_angles);
        self.trim[Self::RIGHT_ARM].rotate_around(Self::CROUCH_RIGHT_SHOULDER, &axes_angles);
    }

    fn apply_red(&mut self, saturation: u8) {
        self.reset();
        for limb in self.limbs.iter_mut() {
            limb.apply_red(saturation);
        }
        for trim in self.trim.iter_mut() {
            trim.apply_red(saturation);
        }
    }

    fn apply_red_crouched(&mut self, saturation: u8) {
        self.reset_crouched();
        for limb in self.limbs.iter_mut() {
            limb.apply_red(saturation);
        }
        for trim in self.trim.iter_mut() {
            trim.apply_red(saturation);
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

pub struct Limb {
    pub faces: [Face; 6],
    pub matrix: Mat4,
}

impl Limb {
    pub fn load(
        atlas: &image::DynamicImage,
        name: String,
        patch: Patch,
        translation: Vec3,
        context: &Context,
    ) -> Self {
        let matrix = Mat4::from_translation(translation);
        let mut limb = Self {
            faces: [
                Face::new_front(&name, &atlas, patch, context),
                Face::new_right(&name, &atlas, patch, context),
                Face::new_back(&name, &atlas, patch, context),
                Face::new_left(&name, &atlas, patch, context),
                Face::new_top(&name, &atlas, patch, context),
                Face::new_bottom(&name, &atlas, patch, context),
            ],
            matrix,
        };
        limb.set_transformation(Mat4::identity());
        limb
    }

    fn set_transformation(&mut self, transformation: Mat4) {
        for face in self.faces.iter_mut() {
            face.model.set_transformation(transformation * self.matrix);
        }
    }

    pub fn rotate_around(&mut self, pivot: Vec3, axes_angles: &[(Vec3, f32)]) {
        let rotation = axes_angles
            .into_iter()
            .fold(Mat4::from_translation(pivot), |matrix, (axis, angle)| {
                matrix * Mat4::from_axis_angle(*axis, degrees(*angle))
            })
            * Mat4::from_translation(-pivot);
        self.set_transformation(rotation);
    }

    fn apply_red(&mut self, saturation: u8) {
        self.apply_color(Srgba::new(255, saturation, saturation, 255))
    }

    fn apply_color(&mut self, color: Srgba) {
        for face in self.faces.iter_mut() {
            face.model.material.color = color;
        }
    }
}

pub struct Face {
    pub model: Gm<Mesh, ColorMaterial>,
}

impl Face {
    fn new_front(name: &str, atlas: &image::DynamicImage, patch: Patch, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(-x, -y, z),
            vec3(x, -y, z),
            vec3(x, y, z),
            vec3(-x, y, z),
        ];
        Self::new(name, atlas, patch, positions, 0, context)
    }

    fn new_right(name: &str, atlas: &image::DynamicImage, patch: Patch, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(x, -y, z),
            vec3(x, -y, -z),
            vec3(x, y, -z),
            vec3(x, y, z),
        ];
        Self::new(name, atlas, patch.as_right(), positions, 1, context)
    }

    fn new_back(name: &str, atlas: &image::DynamicImage, patch: Patch, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(x, -y, -z),
            vec3(-x, -y, -z),
            vec3(-x, y, -z),
            vec3(x, y, -z),
        ];
        Self::new(name, atlas, patch.as_back(), positions, 2, context)
    }

    fn new_left(name: &str, atlas: &image::DynamicImage, patch: Patch, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(-x, -y, -z),
            vec3(-x, -y, z),
            vec3(-x, y, z),
            vec3(-x, y, -z),
        ];
        Self::new(name, atlas, patch.as_left(), positions, 3, context)
    }

    fn new_top(name: &str, atlas: &image::DynamicImage, patch: Patch, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(-x, y, z),
            vec3(x, y, z),
            vec3(x, y, -z),
            vec3(-x, y, -z),
        ];
        Self::new(name, atlas, patch.as_top(), positions, 4, context)
    }

    fn new_bottom(
        name: &str,
        atlas: &image::DynamicImage,
        patch: Patch,
        context: &Context,
    ) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(-x, -y, -z),
            vec3(x, -y, -z),
            vec3(x, -y, z),
            vec3(-x, -y, z),
        ];
        Self::new(name, atlas, patch.as_bottom(), positions, 5, context)
    }

    fn new(
        name: &str,
        atlas: &image::DynamicImage,
        patch: Patch,
        positions: Vec<Vec3>,
        index: u8,
        context: &Context,
    ) -> Self {
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

        let sub_image =
            image::imageops::crop_imm(atlas, patch.x, patch.y, patch.width, patch.height)
                .to_image();
        let mut linear_data = Vec::with_capacity((sub_image.width() * sub_image.height()) as usize);
        for pixel in sub_image.pixels() {
            let r = correct_gamma(pixel[0]);
            let g = correct_gamma(pixel[1]);
            let b = correct_gamma(pixel[2]);
            let a = pixel[3];
            linear_data.push([r, g, b, a]);
        }
        let data = TextureData::RgbaU8(linear_data);
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
            render_states: RenderStates {
                cull: Cull::None,
                ..Default::default()
            },
            ..Default::default()
        };

        let model = Gm::new(Mesh::new(context, &mesh), material);

        Self { model }
    }
}

fn correct_gamma(channel: u8) -> u8 {
    let channel = channel as f32 / 255.0;
    let corrected_channel = if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    };
    (corrected_channel * 255.0) as u8
}

pub struct Trim {
    pub texels: Vec<Texel>,
    pub matrix: Mat4,
}

impl Trim {
    const UNIT: f32 = 1.1;

    pub fn load(
        atlas: &image::DynamicImage,
        name: String,
        patch: Patch,
        translation: Vec3,
        context: &Context,
    ) -> Self {
        let mut texels = vec![];

        const ALPHA_MIN: u8 = 255;

        for px in 0..patch.width {
            for py in 0..patch.height {
                let tx = (px as i32 - patch.width as i32 / 2) as f32 * Self::UNIT;
                let ty = (patch.height as i32 / 2 - 1 - py as i32) as f32 * Self::UNIT;
                let tz = Self::UNIT / 2.0 * patch.depth as f32;
                let pixel = atlas.get_pixel(px + patch.x, py + patch.y).0;

                if pixel[3] >= ALPHA_MIN {
                    texels.push(Texel::new(
                        &name,
                        pixel,
                        vec3(tx, ty, tz),
                        Direction::Front,
                        context,
                    ));
                }
            }
        }

        let right_patch = patch.as_right();
        for px in 0..right_patch.width {
            for py in 0..right_patch.height {
                let tx = Self::UNIT / 2.0 * right_patch.depth as f32;
                let ty = (right_patch.height as i32 / 2 - 1 - py as i32) as f32 * Self::UNIT;
                let tz = (right_patch.width as i32 / 2 - 1 - px as i32) as f32 * Self::UNIT;
                let pixel = atlas.get_pixel(px + right_patch.x, py + right_patch.y).0;

                if pixel[3] >= ALPHA_MIN {
                    texels.push(Texel::new(
                        &name,
                        pixel,
                        vec3(tx, ty, tz),
                        Direction::Right,
                        context,
                    ));
                }
            }
        }

        let back_patch = patch.as_back();
        for px in 0..back_patch.width {
            for py in 0..back_patch.height {
                let tx = (back_patch.width as i32 / 2 - 1 - px as i32) as f32 * Self::UNIT;
                let ty = (back_patch.height as i32 / 2 - 1 - py as i32) as f32 * Self::UNIT;
                let tz = -Self::UNIT / 2.0 * back_patch.depth as f32;
                let pixel = atlas.get_pixel(px + back_patch.x, py + back_patch.y).0;

                if pixel[3] >= ALPHA_MIN {
                    texels.push(Texel::new(
                        &name,
                        pixel,
                        vec3(tx, ty, tz),
                        Direction::Back,
                        context,
                    ));
                }
            }
        }

        let left_patch = patch.as_left();
        for px in 0..left_patch.width {
            for py in 0..left_patch.height {
                let tx = -Self::UNIT / 2.0 * left_patch.depth as f32;
                let ty = (left_patch.height as i32 / 2 - 1 - py as i32) as f32 * Self::UNIT;
                let tz = (px as i32 - left_patch.width as i32 / 2) as f32 * Self::UNIT;
                let pixel = atlas.get_pixel(px + left_patch.x, py + left_patch.y).0;

                if pixel[3] >= ALPHA_MIN {
                    texels.push(Texel::new(
                        &name,
                        pixel,
                        vec3(tx, ty, tz),
                        Direction::Left,
                        context,
                    ));
                }
            }
        }

        let top_patch = patch.as_top();
        for px in 0..top_patch.width {
            for py in 0..top_patch.height {
                let tx = (px as i32 - top_patch.width as i32 / 2) as f32 * Self::UNIT;
                let ty = Self::UNIT / 2.0 * top_patch.depth as f32;
                let tz = (py as i32 - top_patch.height as i32 / 2) as f32 * Self::UNIT;
                let pixel = atlas.get_pixel(px + top_patch.x, py + top_patch.y).0;

                if pixel[3] >= ALPHA_MIN {
                    texels.push(Texel::new(
                        &name,
                        pixel,
                        vec3(tx, ty, tz),
                        Direction::Top,
                        context,
                    ));
                }
            }
        }

        let bottom_patch = patch.as_bottom();
        for px in 0..bottom_patch.width {
            for py in 0..bottom_patch.height {
                let tx = (px as i32 - bottom_patch.width as i32 / 2) as f32 * Self::UNIT;
                let ty = -Self::UNIT / 2.0 * bottom_patch.depth as f32;
                let tz = (bottom_patch.height as i32 / 2 - 1 - py as i32) as f32 * Self::UNIT;
                let pixel = atlas.get_pixel(px + bottom_patch.x, py + bottom_patch.y).0;

                if pixel[3] >= ALPHA_MIN {
                    texels.push(Texel::new(
                        &name,
                        pixel,
                        vec3(tx, ty, tz),
                        Direction::Bottom,
                        context,
                    ));
                }
            }
        }

        Self {
            texels,
            matrix: Mat4::from_translation(translation),
        }
    }

    fn set_transformation(&mut self, transformation: Mat4) {
        for texel in self.texels.iter_mut() {
            texel.model.set_transformation(transformation * self.matrix);
        }
    }

    pub fn rotate_around(&mut self, pivot: Vec3, axes_angles: &[(Vec3, f32)]) {
        let rotation = axes_angles
            .into_iter()
            .fold(Mat4::from_translation(pivot), |matrix, (axis, angle)| {
                matrix * Mat4::from_axis_angle(*axis, degrees(*angle))
            })
            * Mat4::from_translation(-pivot);
        self.set_transformation(rotation);
    }

    fn apply_red(&mut self, saturation: u8) {
        self.apply_color(Srgba::new(255, saturation, saturation, 255))
    }

    fn apply_color(&mut self, color: Srgba) {
        for texel in self.texels.iter_mut() {
            texel.model.material.color = color;
        }
    }
}

pub struct Texel {
    pub model: Gm<Mesh, ColorMaterial>,
}

impl Texel {
    fn new(
        name: &str,
        pixel: [u8; 4],
        position: Vec3,
        direction: Direction,
        context: &Context,
    ) -> Self {
        use Direction::*;
        let positions = match direction {
            Front => vec![
                vec3(position.x, position.y, position.z),
                vec3(position.x + Trim::UNIT, position.y, position.z),
                vec3(position.x + Trim::UNIT, position.y + Trim::UNIT, position.z),
                vec3(position.x, position.y + Trim::UNIT, position.z),
            ],
            Right => vec![
                vec3(position.x, position.y, position.z + Trim::UNIT),
                vec3(position.x, position.y, position.z),
                vec3(position.x, position.y + Trim::UNIT, position.z),
                vec3(position.x, position.y + Trim::UNIT, position.z + Trim::UNIT),
            ],
            Back => vec![
                vec3(position.x + Trim::UNIT, position.y, position.z),
                vec3(position.x, position.y, position.z),
                vec3(position.x, position.y + Trim::UNIT, position.z),
                vec3(position.x + Trim::UNIT, position.y + Trim::UNIT, position.z),
            ],
            Left => vec![
                vec3(position.x, position.y, position.z),
                vec3(position.x, position.y, position.z + Trim::UNIT),
                vec3(position.x, position.y + Trim::UNIT, position.z + Trim::UNIT),
                vec3(position.x, position.y + Trim::UNIT, position.z),
            ],
            Top => vec![
                vec3(position.x, position.y, position.z + Trim::UNIT),
                vec3(position.x + Trim::UNIT, position.y, position.z + Trim::UNIT),
                vec3(position.x + Trim::UNIT, position.y, position.z),
                vec3(position.x, position.y, position.z),
            ],
            Bottom => vec![
                vec3(position.x, position.y, position.z),
                vec3(position.x + Trim::UNIT, position.y, position.z),
                vec3(position.x + Trim::UNIT, position.y, position.z + Trim::UNIT),
                vec3(position.x, position.y, position.z + Trim::UNIT),
            ],
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
        let pixel = {
            let r = correct_gamma(pixel[0]);
            let g = correct_gamma(pixel[1]);
            let b = correct_gamma(pixel[2]);
            let a = pixel[3];
            [r, g, b, a]
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
            render_states: RenderStates {
                cull: Cull::None,
                ..Default::default()
            },
            ..Default::default()
        };

        let model = Gm::new(Mesh::new(context, &mesh), material);

        Self { model }
    }
}

enum Direction {
    Front,
    Right,
    Back,
    Left,
    Top,
    Bottom,
}

#[derive(Clone, Copy)]
pub struct Patch {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    depth: u32,
}

impl Patch {
    pub const HEAD: Patch = Patch::new(8, 8, 8, 8, 8);
    pub const TORSO: Patch = Patch::new(20, 20, 8, 12, 4);
    pub const RIGHT_LEG: Patch = Patch::new(4, 20, 4, 12, 4);
    pub const RIGHT_ARM: Patch = Patch::new(44, 20, 4, 12, 4);
    pub const SLIM_RIGHT_ARM: Patch = Patch::new(44, 20, 3, 12, 4);
    pub const LEFT_LEG: Patch = Patch::new(20, 52, 4, 12, 4);
    pub const LEFT_ARM: Patch = Patch::new(36, 52, 4, 12, 4);
    pub const SLIM_LEFT_ARM: Patch = Patch::new(36, 52, 3, 12, 4);

    pub const HELMET: Patch = Patch::new(40, 8, 8, 8, 8);
    pub const SHIRT: Patch = Patch::new(20, 36, 8, 12, 4);
    pub const RIGHT_PANTS: Patch = Patch::new(4, 36, 4, 12, 4);
    pub const RIGHT_SLEEVE: Patch = Patch::new(44, 36, 4, 12, 4);
    pub const SLIM_RIGHT_SLEEVE: Patch = Patch::new(44, 36, 3, 12, 4);
    pub const LEFT_PANTS: Patch = Patch::new(4, 52, 4, 12, 4);
    pub const LEFT_SLEEVE: Patch = Patch::new(52, 52, 4, 12, 4);
    pub const SLIM_LEFT_SLEEVE: Patch = Patch::new(52, 52, 3, 12, 4);

    const fn new(x: u32, y: u32, width: u32, height: u32, depth: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            depth,
        }
    }

    fn half_size(&self) -> Vec3 {
        vec3(
            self.width as f32 / 2.0,
            self.height as f32 / 2.0,
            self.depth as f32 / 2.0,
        )
    }

    fn as_right(&self) -> Self {
        Self::new(
            self.x + self.width,
            self.y,
            self.depth,
            self.height,
            self.width,
        )
    }

    fn as_back(&self) -> Self {
        Self::new(
            self.x + self.width + self.depth,
            self.y,
            self.width,
            self.height,
            self.depth,
        )
    }

    fn as_left(&self) -> Self {
        Self::new(
            self.x - self.depth,
            self.y,
            self.depth,
            self.height,
            self.width,
        )
    }

    fn as_top(&self) -> Self {
        Self::new(
            self.x,
            self.y - self.depth,
            self.width,
            self.depth,
            self.height,
        )
    }

    fn as_bottom(&self) -> Self {
        Self::new(
            self.x + self.width,
            self.y - self.depth,
            self.width,
            self.depth,
            self.height,
        )
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
