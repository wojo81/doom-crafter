use tinywad::{models::operation::WadOp, wad::Wad};

pub fn consume_images(
    name: &str,
    sprite: &str,
    mugshot: &str,
    wad: &mut Wad,
) -> anyhow::Result<()> {
    let mut crouch_sprite = sprite.to_string();
    crouch_sprite.replace_range(3..4, "[");
    wad.add_lump_raw(tinywad::lump::LumpAdd::new(
        tinywad::lump::LumpAddKind::Back,
        &format!(
            "name = \"{name}\"\nsprite = {sprite}\ncrouchsprite = {crouch_sprite}\nface = {mugshot}\nscale = 0.5\ngender = male"
        )
        .as_bytes()
        .to_vec(),
        "S_SKIN",
    ))?;

    let mut paths = std::fs::read_dir(std::path::Path::new("temp").join("sprites"))
        .map(|d| d.map(|p| p.unwrap().path().to_str().unwrap().to_string()))?
        .collect::<Vec<_>>();
    paths.sort();
    putpng::grab::grab_all(
        paths.iter().map(|s| s.clone()),
        "w / 2".to_string(),
        "h - 15".to_string(),
    )
    .unwrap();
    putpng::crop::apply_crop(paths.iter().map(|s| s.clone())).unwrap();
    for path in paths {
        wad.add_lump_raw(tinywad::lump::LumpAdd::new(
            tinywad::lump::LumpAddKind::Back,
            &std::fs::read(&path)?,
            &std::path::Path::new(&path)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap(),
        ))?;
    }

    let mut paths = std::fs::read_dir(std::path::Path::new("temp").join("crouch_sprites"))
        .map(|d| d.map(|p| p.unwrap().path().to_str().unwrap().to_string()))?
        .collect::<Vec<_>>();
    paths.sort();
    putpng::grab::grab_all(
        paths.iter().map(|s| s.clone()),
        "w / 2".to_string(),
        "h - 15".to_string(),
    )
    .unwrap();
    putpng::crop::apply_crop(paths.iter().map(|s| s.clone())).unwrap();
    for path in paths {
        wad.add_lump_raw(tinywad::lump::LumpAdd::new(
            tinywad::lump::LumpAddKind::Back,
            &std::fs::read(&path)?,
            &std::path::Path::new(&path)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap(),
        ))?;
    }

    let mut paths = std::fs::read_dir(std::path::Path::new("temp").join("mugshots"))
        .map(|d| d.map(|p| p.unwrap().path().to_str().unwrap().to_string()))?
        .collect::<Vec<_>>();
    paths.sort();
    putpng::grab::grab_all(
        paths.iter().map(|s| s.clone()),
        "w / 2 - 18".to_string(),
        "h / 2 - 17".to_string(),
    )
    .unwrap();
    putpng::crop::apply_crop(paths.iter().map(|s| s.clone())).unwrap();
    for path in paths {
        wad.add_lump_raw(tinywad::lump::LumpAdd::new(
            tinywad::lump::LumpAddKind::Back,
            &std::fs::read(&path)?,
            &std::path::Path::new(&path)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap(),
        ))?;
    }

    Ok(())
}
