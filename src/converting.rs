use crate::{
    producing::{produce_decorate_wad, produce_s_skin_and_fist_wads, produce_s_skin_wad},
    rendering::{render_fist, render_mugshot, render_skin_with_crouch},
};
use image::DynamicImage;
use putpng::crc::Crc32;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use three_d::*;

pub trait SpritePrefix {
    fn to_skin_sprite(&self) -> String;
    fn to_crouched_skin_sprite(&self) -> String;
    fn to_mugshot_sprite(&self) -> String;
    fn to_fist_sprite(&self) -> String;
    fn quoted(&self) -> String;
}

impl SpritePrefix for str {
    fn to_skin_sprite(&self) -> String {
        self.to_string() + "]"
    }

    fn to_crouched_skin_sprite(&self) -> String {
        self.to_string() + "["
    }

    fn to_mugshot_sprite(&self) -> String {
        self.to_string()
    }

    fn to_fist_sprite(&self) -> String {
        self.to_string() + "\\"
    }

    fn quoted(&self) -> String {
        format!("\"{self}\"")
    }
}

#[allow(non_camel_case_types)]
pub enum Format {
    S_SkinWad,
    S_SkinAndFistWads,
    DecorateWad,
    // DecoratePk3,
    // ZScriptPk3,
}

type Render = fn(&DynamicImage, &Path, &str, &mut Rendering, usize) -> anyhow::Result<()>;
type Produce = fn(&Path, &Path, Vec<(String, String)>, crc: &Crc32) -> anyhow::Result<()>;

impl Format {
    fn methods(&self) -> (Vec<Render>, Produce) {
        use Format::*;
        match self {
            S_SkinWad => (
                vec![render_skin_with_crouch, render_mugshot],
                produce_s_skin_wad,
            ),
            S_SkinAndFistWads => (
                vec![render_skin_with_crouch, render_mugshot, render_fist],
                produce_s_skin_and_fist_wads,
            ),
            DecorateWad => (
                vec![render_skin_with_crouch, render_mugshot, render_fist],
                produce_decorate_wad,
            ),
        }
    }
}

#[derive(Default, Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct SkinData {
    pub name: String,
    pub path: String,
    pub sprite_prefix: String,
}

impl SkinData {
    pub fn as_refs(&self) -> [&str; 3] {
        [&self.name, &self.path, &self.sprite_prefix]
    }
}

pub fn get_acc() -> Option<PathBuf> {
    if let Ok(acc) = which::which("acc") {
        Some(acc)
    } else if cfg!(target_os = "linux") && Path::new("acc").exists() {
        Some("./acc".into())
    } else {
        None
    }
}

pub struct Rendering {
    pub viewport: Viewport,
    pub context: HeadlessContext,
    pub camera: Camera,
}

impl Rendering {
    const DEPTH: f32 = 35.0;

    pub fn new() -> anyhow::Result<Self> {
        const DEPTH: f32 = 35.0;

        let viewport = Viewport::new_at_origo(204, 128);
        let context = HeadlessContext::new()?;
        let camera = Camera::new_perspective(
            viewport,
            Vec3::unit_z() * DEPTH,
            Vec3::zero(),
            Vec3::unit_y(),
            degrees(60.0),
            0.1,
            100.0,
        );

        Ok(Self {
            viewport,
            context,
            camera,
        })
    }
}

pub fn convert(data: &Vec<SkinData>, format: Format, produced_file: &Path) -> anyhow::Result<()> {
    let mut rendering = Rendering::new()?;
    let rendered_dir = tempdir().unwrap();
    let (renders, produce) = format.methods();
    let crc = Crc32::new();

    let mut names_and_sprite_prefixes = vec![];
    for (
        index,
        SkinData {
            name,
            path,
            sprite_prefix,
        },
    ) in data.iter().enumerate()
    {
        names_and_sprite_prefixes.push((name.clone(), sprite_prefix.replace("\\", "^")));
        let image = image::open(path)?;
        for render in &renders {
            render(
                &image,
                &rendered_dir.path(),
                sprite_prefix,
                &mut rendering,
                index,
            )?;
        }
        rendering.camera = Camera::new_perspective(
            rendering.viewport,
            Vec3::unit_z() * Rendering::DEPTH,
            Vec3::zero(),
            Vec3::unit_y(),
            degrees(60.0),
            0.1,
            100.0,
        );
    }

    produce(
        &rendered_dir.path(),
        produced_file,
        names_and_sprite_prefixes,
        &crc,
    )?;

    Ok(())
}
