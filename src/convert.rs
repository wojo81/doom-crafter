use putpng::crc::Crc32;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use three_d::*;
use tinywad::models::operation::WadOp;

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

pub fn convert_all(
    items: &Vec<SkinItem>,
    file_name: String,
    acc: Option<PathBuf>,
) -> anyhow::Result<()> {
    let viewport = Viewport::new_at_origo(204, 128);
    let context = HeadlessContext::new()?;
    let depth = 35.0;
    let mut camera = Camera::new_perspective(
        viewport,
        Vec3::unit_z() * depth,
        Vec3::zero(),
        Vec3::unit_y(),
        degrees(60.0),
        0.1,
        100.0,
    );
    let crc = Crc32::new();

    let mut wad = tinywad::wad::Wad::new();
    wad.set_kind(tinywad::wad::WadKind::Pwad);
    let mut fists_wad = tinywad::wad::Wad::new();
    fists_wad.set_kind(tinywad::wad::WadKind::Pwad);
    fists_wad.add_lump_raw(tinywad::lump::LumpAdd::new(
        tinywad::lump::LumpAddKind::Back,
        &vec![],
        "S_START",
    ))?;

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
        convert(
            &name,
            &path,
            &sprite,
            &mugshot,
            &mut wad,
            &viewport,
            &context,
            &mut camera,
            &crc,
            &temp.path(),
        )?;
        if acc.is_some() {
            crate::fists::convert(
                &path,
                sprite.clone(),
                &mut fists_wad,
                &viewport,
                &context,
                &mut camera,
                &crc,
                &temp.path(),
            )?;
        }
        camera = Camera::new_perspective(
            viewport,
            Vec3::unit_z() * depth,
            Vec3::zero(),
            Vec3::unit_y(),
            degrees(60.0),
            0.1,
            100.0,
        );
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
    viewport: &Viewport,
    context: &Context,
    camera: &mut Camera,
    crc: &Crc32,
    temp: &Path,
) -> anyhow::Result<()> {
    crate::minecraft::render_images(path, sprite, mugshot, viewport, context, camera, temp)?;
    crate::doom::consume_images(name, sprite, mugshot, wad, crc, temp)?;
    Ok(())
}
