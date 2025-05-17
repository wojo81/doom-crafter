use tinywad::{models::operation::WadOp, wad::Wad};

pub fn consume_images(name: &str, sprite: &str, wad: &mut Wad) -> anyhow::Result<()> {
    wad.add_lump_raw(tinywad::lump::LumpAdd::new(tinywad::lump::LumpAddKind::Back,
        &format!("name = \"{name}\"\nsprite = {sprite}\nscale = 0.5\ngender = male").as_bytes().to_vec(), "S_SKIN"))?;

    let paths = std::fs::read_dir("temp").map(|d| d.map(|p| p.unwrap().path().to_str().unwrap().to_string()))?.collect::<Vec<_>>();
    putpng::grab::grab_all(paths.iter().map(|s| s.clone()), "w / 2".to_string(), "h - 5".to_string()).unwrap();
    putpng::crop::apply_crop(paths.iter().map(|s| s.clone())).unwrap();
    for path in paths {
        wad.add_lump_raw(tinywad::lump::LumpAdd::new(tinywad::lump::LumpAddKind::Back,
            &std::fs::read(&path)?, &std::path::Path::new(&path).file_name().unwrap().to_str().unwrap()[0..6]
        ))?;
    }

    Ok(())
}