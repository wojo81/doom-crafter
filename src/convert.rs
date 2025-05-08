use three_d::*;
use tinywad::wad::Wad;

pub fn convert_all(paths_and_names: impl Iterator<Item = (String, String)>) -> anyhow::Result<()> {
    let viewport = Viewport::new_at_origo(192, 128);
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

    let mut wad = Wad::new();
    wad.set_kind(tinywad::wad::WadKind::Pwad);
    for (path, name) in paths_and_names {
        std::fs::create_dir("out")?;
        convert(&path, &name, &mut wad, &viewport, &context, &mut camera)?;
        std::fs::remove_dir_all("out")?;
    }

    Ok(())
}

fn convert(path: &str, name: &str, wad: &mut Wad, viewport: &Viewport, context: &Context, camera: &mut Camera) -> anyhow::Result<()> {
    crate::minecraft::generate_images(path, name, viewport, context, camera)?;
    crate::doom::consume_images(path, name, wad)?;
    Ok(())
}

