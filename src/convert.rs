use three_d::*;
use tinywad::models::operation::WadOp;

#[derive(Default, Clone)]
pub struct SkinInfo {
    pub name: String,
    pub path: String,
    pub sprite: String,
}

pub fn convert_all(infos: &Vec<SkinInfo>, file_name: String) -> anyhow::Result<()> {
    let viewport = Viewport::new_at_origo(204, 128);
    let context = HeadlessContext::new()?;
    let depth = 35.0;
    let mut camera = Camera::new_perspective(
        viewport,
        vec3(0.0, 0.0, depth),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(60.0),
        0.1,
        100.0,
    );

    let mut wad = tinywad::wad::Wad::new();
    wad.set_kind(tinywad::wad::WadKind::Pwad);
    for SkinInfo { name, path, sprite } in infos {
        std::fs::create_dir("temp")?;
        convert(&name, &path, &sprite, &mut wad, &viewport, &context, &mut camera)?;
        std::fs::remove_dir_all("temp")?;
    }

    wad.save(&file_name);

    Ok(())
}

fn convert(name: &str, path: &str, sprite: &str,  wad: &mut tinywad::wad::Wad, viewport: &Viewport, context: &Context, camera: &mut Camera) -> anyhow::Result<()> {
    crate::minecraft::render_images(path, sprite, viewport, context, camera)?;
    crate::doom::consume_images(name, sprite, wad)?;
    Ok(())
}

