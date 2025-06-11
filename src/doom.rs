use putpng::crc::Crc32;
use tinywad::{models::operation::WadOp, wad::Wad};

pub fn consume_images(
    name: &str,
    sprite: &str,
    mugshot: &str,
    wad: &mut Wad,
    crc: &Crc32,
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

    grab_write_into("sprites", "w / 2", "h - 15", crc, wad)?;
    grab_write_into("crouch_sprites", "w / 2", "h - 15", crc, wad)?;
    grab_write_into("mugshots", "w / 2 - 18", "h / 2 - 17", crc, wad)?;

    Ok(())
}

fn grab_write_into(
    subpath: &str,
    x: &str,
    y: &str,
    crc: &Crc32,
    wad: &mut Wad,
) -> anyhow::Result<()> {
    let mut paths = std::fs::read_dir(std::path::Path::new("temp").join(subpath))
        .map(|d| d.map(|p| p.unwrap().path().to_str().unwrap().to_string()))?
        .collect::<Vec<_>>();
    paths.sort();
    putpng::grab::grab_all(
        paths.iter().map(|s| s.clone()),
        &crc,
        x.to_string(),
        y.to_string(),
    )
    .unwrap();
    putpng::crop::crop_all(paths.iter().map(|s| s.clone()), &crc).unwrap();
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
