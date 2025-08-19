use putpng::crc::Crc32;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use three_d::*;
use tinywad::models::operation::WadOp;

use crate::doom::PushLump;

#[derive(Default, Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct SkinItem {
    pub name: String,
    pub path: String,
    pub sprite: String,
    pub mugshot: String,
}

impl SkinItem {
    pub const fn ref_array(&self) -> [&String; 4] {
        [&self.name, &self.path, &self.sprite, &self.mugshot]
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

pub fn convert_all(
    items: &Vec<SkinItem>,
    file_name: String,
    acc: Option<PathBuf>,
    as_skins: bool,
) -> anyhow::Result<()> {
    let mut rendering = Rendering::new()?;
    let crc = Crc32::new();

    let mut wad = tinywad::wad::Wad::new();
    wad.set_kind(tinywad::wad::WadKind::Pwad);
    let mut fists_wad = tinywad::wad::Wad::new();
    fists_wad.set_kind(tinywad::wad::WadKind::Pwad);
    fists_wad.push_lump(&vec![], "S_START")?;

    let mut index = 1;
    for SkinItem {
        name,
        path,
        sprite,
        mugshot,
    } in items.iter()
    {
        let temp = tempdir().unwrap();
        std::fs::create_dir(temp.path().join("sprites"))?;
        std::fs::create_dir(temp.path().join("mugshots"))?;
        std::fs::create_dir(temp.path().join("crouch_sprites"))?;
        std::fs::create_dir(temp.path().join("fists"))?;
        if !as_skins {
            let mut sprite = sprite.to_string();
            sprite.replace_range(3..4, "]");
            crate::fists::render_fists(&path, &sprite, &mut rendering, temp.path())?;
        }
        convert(
            &name,
            &path,
            &sprite,
            &mugshot,
            &mut wad,
            &mut rendering,
            &crc,
            &temp.path(),
            as_skins,
            index,
        )?;
        if acc.is_some() {
            crate::fists::convert(
                &path,
                sprite.clone(),
                &mut fists_wad,
                &mut rendering,
                &crc,
                &temp.path(),
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
        index += 1;
    }

    if !as_skins {
        let mut decorate = String::new();
        let mut mapinfo = "GameInfo {\n    PlayerClasses = ".to_string();
        for i in 1..index {
            decorate += &format!("#include \"DEC_M{i}\"\n");
            mapinfo += &format!("\"Crafter_{i}\",");
        }
        mapinfo.remove(mapinfo.len() - 1);
        mapinfo += "\n}\n";
        wad.push_lump(&decorate.as_bytes().to_vec(), "DECORATE")?;
        wad.push_lump(&mapinfo.as_bytes().to_vec(), "MAPINFO")?;
    }

    wad.save(&file_name);
    if let Some(acc) = acc {
        crate::fists::finalize(fists_wad, acc, items)?.save(file_name.replace('.', "_fists."));
    }

    Ok(())
}

fn convert(
    name: &str,
    path: &str,
    sprite: &str,
    mugshot: &str,
    wad: &mut tinywad::wad::Wad,
    rendering: &mut Rendering,
    crc: &Crc32,
    temp: &Path,
    as_skins: bool,
    index: usize,
) -> anyhow::Result<()> {
    crate::minecraft::render_images(path, sprite, mugshot, rendering, temp)?;
    crate::doom::consume_images(name, sprite, mugshot, wad, crc, temp, as_skins, index)?;
    Ok(())
}
