use crate::{
    convert::{Rendering, SkinItem},
    doom::PushLump,
    minecraft::*,
};
use putpng::crc::Crc32;
use std::{f32::consts::PI, path::Path};
use three_d::*;

pub fn get_acc() -> Option<std::path::PathBuf> {
    if let Ok(acc) = which::which("acc") {
        Some(acc)
    } else if cfg!(target_os = "linux") && Path::new("acc").exists() {
        Some("./acc".into())
    } else {
        None
    }
}

pub fn convert(
    path: &str,
    mut sprite: String,
    fists_wad: &mut tinywad::wad::Wad,
    rendering: &mut Rendering,
    crc: &Crc32,
    temp: &Path,
) -> anyhow::Result<()> {
    sprite.replace_range(3..4, "]");
    render_fists(path, &sprite, rendering, temp)?;
    consume_fists(fists_wad, crc, temp)?;
    Ok(())
}

pub fn render_fists(
    path: &str,
    sprite: &str,
    rendering: &mut Rendering,
    temp: &Path,
) -> anyhow::Result<()> {
    let mut target = Texture2D::new_empty::<[u8; 4]>(
        &rendering.context,
        rendering.viewport.width,
        rendering.viewport.height,
        Interpolation::Nearest,
        Interpolation::Nearest,
        None,
        Wrapping::ClampToEdge,
        Wrapping::ClampToEdge,
    );
    let mut depth = DepthTexture2D::new::<f32>(
        &rendering.context,
        rendering.viewport.width,
        rendering.viewport.height,
        Wrapping::ClampToEdge,
        Wrapping::ClampToEdge,
    );

    let position = Vec3::unit_x() * 3.5;
    let mut arm = Limb::load(
        &image::open(path)?,
        "arm".into(),
        Patch::RIGHT_ARM,
        position,
        &rendering.context,
    );
    let mut sleeve = Trim::load(
        &image::open(path)?,
        "arm".into(),
        Patch::RIGHT_SLEEVE,
        position,
        &rendering.context,
    );

    let delta = 22.5;

    rendering.camera.rotate_around(Vec3::zero(), PI, 0.0);
    rendering.camera.translate(Vec3::unit_z() * -delta);

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
            &rendering.viewport,
            &mut target,
            &mut depth,
            &rendering.camera,
            temp,
        )?;
    }

    rendering.camera.translate(Vec3::unit_z() * delta);
    rendering.camera.rotate_around(Vec3::zero(), PI, 0.0);

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
    temp: &Path,
) -> anyhow::Result<()> {
    let pixels = RenderTarget::new(target.as_color_target(None), depth.as_depth_target())
        .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 0.0, 1.0))
        .render(
            &camera,
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
            width: viewport.width,
            height: viewport.height,
            ..Default::default()
        }
        .serialize(
            temp.join("fists")
                .join(format!("{sprite}{frame_index}0.png")),
        )?,
    )?;
    Ok(())
}

pub fn consume_fists(
    fists_wad: &mut tinywad::wad::Wad,
    crc: &Crc32,
    temp: &Path,
) -> anyhow::Result<()> {
    let mut paths = std::fs::read_dir(temp.join("fists"))
        .map(|d| d.map(|p| p.unwrap().path().to_str().unwrap().to_string()))?
        .collect::<Vec<_>>();
    paths.sort();
    putpng::grab::grab_all(
        paths.iter().map(|s| s.clone()),
        crc,
        "-w / 2 - 15".into(),
        "-h / 2 + 3".into(),
        true,
    )
    .unwrap();
    putpng::crop::crop_all(paths.iter().map(|s| s.clone()), crc).unwrap();

    for path in paths {
        fists_wad.push_lump(
            &std::fs::read(&path)?,
            &std::path::Path::new(&path)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap(),
        )?;
    }
    Ok(())
}

pub fn generate_fist(sprite: &str, index: usize) -> String {
    indoc::formatdoc!(
        r#"
        ACTOR Fist_{index} : Weapon replaces Fist {{
            Tag "$FIST"
            Weapon.SlotNumber 1
            Weapon.SelectionOrder 1
            +Weapon.NOAUTOFIRE
            +Weapon.MELEEWEAPON

            States {{
            Ready:
                {sprite} A 1 A_WeaponReady
                Loop
            Select:
                {sprite} A 1 A_Raise
                Loop
            Deselect:
                {sprite} A 1 A_Lower
                Loop
            Fire:
                {sprite} B 2 A_Punch
                {sprite} CDEFGHI 2
                TNT1 A 0 A_Refire
                Goto Ready
            }}
        }}
        "#
    )
}

pub fn finalize(
    mut fists_wad: tinywad::wad::Wad,
    acc: std::path::PathBuf,
    items: &Vec<SkinItem>,
) -> anyhow::Result<tinywad::wad::Wad> {
    fists_wad.push_lump(&vec![], "S_END")?;
    {
        let mut code = String::new();
        for (i, item) in items.iter().enumerate() {
            let i = i + 1;
            let mut sprite = format!("\"{}\"", item.sprite);
            sprite.replace_range(4..5, "]");
            code += &generate_fist(&sprite, i);
        }
        fists_wad.push_lump(&code.into_bytes(), "DECORATE")?;
    }

    fists_wad.push_lump(
        &"[enu default]\nFIST = \"Fist\";\0".as_bytes().to_vec(),
        "LANGUAGE",
    )?;
    fists_wad.push_lump(&"pickfist".as_bytes().to_vec(), "LOADACS")?;
    fists_wad.push_lump(&vec![], "A_START")?;
    {
        let removal = indoc::formatdoc!(
            r#"
                if (lastSkin == 0) takeInventory("Fist", 1);
            {}
            "#,
            (1..=items.len())
                .map(|i| format!("\telse if (lastSkin == {i}) takeInventory(\"Fist_{i}\", 1);\n"))
                .collect::<String>()
        );
        let retrieval = indoc::formatdoc!(
            r#"
               if (skin == 0) giveInventory("Fist", 1);
            {}
            "#,
            (1..=items.len())
                .map(|i| format!("\telse if (skin == {i}) giveInventory(\"Fist_{i}\", 1);\n"))
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
    fists_wad.push_lump(&std::fs::read("PICKFIST.o").unwrap(), "PICKFIST")?;
    fists_wad.push_lump(&vec![], "A_END")?;

    std::fs::remove_file("PICKFIST.acs").unwrap();
    std::fs::remove_file("PICKFIST.o").unwrap();

    Ok(fists_wad)
}
