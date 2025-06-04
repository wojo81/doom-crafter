use std::f32::consts::PI;

use crate::{convert::SkinItem, minecraft::*};
use three_d::*;
use tinywad::models::operation::WadOp;

pub fn get_acc() -> Option<std::path::PathBuf> {
    if let Ok(acc) = which::which("acc") {
        Some(acc)
    } else if cfg!(target_os = "linux") && std::path::Path::new("acc").exists() {
        Some("./acc".into())
    } else {
        None
    }
}

pub fn convert(
    path: &str,
    mut sprite: String,
    fists_wad: &mut tinywad::wad::Wad,
    viewport: &Viewport,
    context: &Context,
    camera: &mut Camera,
) -> anyhow::Result<()> {
    sprite.replace_range(3..4, "]");
    render_fists(path, &sprite, viewport, context, camera)?;
    consume_fists(fists_wad)?;
    Ok(())
}

fn render_fists(
    path: &str,
    sprite: &str,
    viewport: &Viewport,
    context: &Context,
    camera: &mut Camera,
) -> anyhow::Result<()> {
    let mut target = Texture2D::new_empty::<[u8; 4]>(
        &context,
        viewport.width,
        viewport.height,
        Interpolation::Nearest,
        Interpolation::Nearest,
        None,
        Wrapping::ClampToEdge,
        Wrapping::ClampToEdge,
    );
    let mut depth = DepthTexture2D::new::<f32>(
        &context,
        viewport.width,
        viewport.height,
        Wrapping::ClampToEdge,
        Wrapping::ClampToEdge,
    );
    let light = AmbientLight::new(context, 1.0, Srgba::WHITE);

    let position = Vec3::unit_x() * 3.5;
    let mut arm = Limb::load(
        &image::open(path)?,
        "arm".into(),
        Patch::RIGHT_ARM,
        position,
        context,
    );
    let mut sleeve = Trim::load(
        &image::open(path)?,
        "arm".into(),
        Patch::RIGHT_SLEEVE,
        position,
        context,
    );

    let delta = 22.5;

    camera.rotate_around(Vec3::zero(), PI, 0.0);
    camera.translate(Vec3::unit_z() * -delta);

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

        render_fist(
            &arm,
            &sleeve,
            sprite,
            frame_index,
            viewport,
            &mut target,
            &mut depth,
            camera,
            &light,
        )?;
    }

    camera.translate(Vec3::unit_z() * delta);
    camera.rotate_around(Vec3::zero(), PI, 0.0);

    Ok(())
}

fn render_fist(
    arm: &Limb,
    sleeve: &Trim,
    sprite: &str,
    frame_index: char,
    viewport: &Viewport,
    target: &mut Texture2D,
    depth: &mut DepthTexture2D,
    camera: &Camera,
    light: &AmbientLight,
) -> anyhow::Result<()> {
    let pixels = RenderTarget::new(target.as_color_target(None), depth.as_depth_target())
        .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 0.0, 1.0))
        .render(
            &camera,
            arm.faces
                .iter()
                .map(|f| &f.model)
                .chain(sleeve.texels.iter().map(|t| &t.model)),
            &[&light],
        )
        .read_color();

    use three_d_asset::io::Serialize;

    three_d_asset::io::save(
        &CpuTexture {
            data: TextureData::RgbaU8(pixels),
            width: viewport.width,
            height: viewport.height,
            ..Default::default()
        }
        .serialize(
            std::path::Path::new("temp")
                .join("fists")
                .join(format!("{sprite}{frame_index}0.png")),
        )?,
    )?;
    Ok(())
}

fn consume_fists(fists_wad: &mut tinywad::wad::Wad) -> anyhow::Result<()> {
    let mut paths = std::fs::read_dir(std::path::Path::new("temp").join("fists"))
        .map(|d| d.map(|p| p.unwrap().path().to_str().unwrap().to_string()))?
        .collect::<Vec<_>>();
    paths.sort();
    putpng::grab::grab_all(
        paths.iter().map(|s| s.clone()),
        "-w / 2 - 15".into(),
        "-h / 2 + 3".into(),
    )
    .unwrap();
    putpng::crop::apply_crop(paths.iter().map(|s| s.clone())).unwrap();

    for path in paths {
        fists_wad.add_lump_raw(tinywad::lump::LumpAdd::new(
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

pub fn finalize(
    mut fists_wad: tinywad::wad::Wad,
    acc: std::path::PathBuf,
    items: &Vec<SkinItem>,
) -> anyhow::Result<tinywad::wad::Wad> {
    fists_wad.add_lump_raw(tinywad::lump::LumpAdd::new(
        tinywad::lump::LumpAddKind::Back,
        &vec![],
        "S_END",
    ))?;
    {
        let mut s = String::new();
        for (i, item) in items.iter().enumerate() {
            let i = i + 1;
            let mut sprite = format!("\"{}\"", item.sprite);
            sprite.replace_range(4..5, "]");
            s.push_str(&indoc::formatdoc!(
                r#"
            actor Fist_{i} : Weapon replaces Fist {{
                Weapon.SlotNumber 1
                Weapon.SelectionOrder 1
                +Weapon.NOAUTOFIRE
                +Weapon.MELEEWEAPON

                States {{
                Ready:
                    {sprite} A 1 A_WeaponReady
                    loop
                Select:
                    {sprite} A 1 A_Raise
                    loop
                Deselect:
                    {sprite} A 1 A_Lower
                    loop
                Fire:
                    {sprite} B 2 A_Punch
                    {sprite} CDEFGHI 2
                    TNT1 A 0 A_Refire
                    goto Ready
                }}
            }}
            "#
            ))
        }
        fists_wad.add_lump_raw(tinywad::lump::LumpAdd::new(
            tinywad::lump::LumpAddKind::Back,
            &s.into_bytes(),
            "DECORATE",
        ))?;
    }

    fists_wad.add_lump_raw(tinywad::lump::LumpAdd::new(
        tinywad::lump::LumpAddKind::Back,
        &"pickfist".to_string().into_bytes(),
        "LOADACS",
    ))?;

    fists_wad.add_lump_raw(tinywad::lump::LumpAdd::new(
        tinywad::lump::LumpAddKind::Back,
        &vec![],
        "A_START",
    ))?;

    {
        let removal = indoc::formatdoc!(
            r#"
                if (lastSkin == 0) takeInventory("Fist", 1);
                {}
            "#,
            (1..=items.len())
                .map(|i| format!("else if (lastSkin == {i}) takeInventory(\"Fist_{i}\", 1);\n"))
                .collect::<String>()
        );
        let retrieval = indoc::formatdoc!(
            r#"
               if (skin == 0) giveInventory("Fist", 1);
               {} 
            "#,
            (1..=items.len())
                .map(|i| format!("else if (skin == {i}) giveInventory(\"Fist_{i}\", 1);\n"))
                .collect::<String>()
        );
        let acs = indoc::formatdoc!(
            r#"
            #library "pickfist"
            #include "zcommon.acs"

            script "PickFist" ENTER {{
                int lastSkin = -1;
                takeInventory("Fist", 1);
                while (true) {{
                    int skin = getUserCVar(playerNumber(), "skin");
                    if (skin != lastSkin) {{
                        {removal}
                        {retrieval}
                        lastSkin = skin;
                    }}
                    delay(35);
                }}
            }}
        "#
        );
        std::fs::write("PICKFIST.acs", acs).unwrap();
        std::process::Command::new(acc)
            .arg("PICKFIST")
            .output()
            .unwrap();
    }
    fists_wad.add_lump_raw(tinywad::lump::LumpAdd::new(
        tinywad::lump::LumpAddKind::Back,
        &std::fs::read("PICKFIST.o").unwrap(),
        "PICKFIST",
    ))?;

    fists_wad.add_lump_raw(tinywad::lump::LumpAdd::new(
        tinywad::lump::LumpAddKind::Back,
        &vec![],
        "A_END",
    ))?;

    std::fs::remove_file("PICKFIST.acs").unwrap();
    std::fs::remove_file("PICKFIST.o").unwrap();

    Ok(fists_wad)
}
