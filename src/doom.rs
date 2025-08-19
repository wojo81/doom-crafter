use putpng::crc::Crc32;
use std::{path::Path, time::Duration};
use tinywad::{models::operation::WadOp, wad::Wad};

use crate::fists;

pub trait PushLump {
    fn push_lump(&mut self, buffer: &Vec<u8>, name: &str) -> anyhow::Result<()>;
}

impl PushLump for Wad {
    fn push_lump(&mut self, buffer: &Vec<u8>, name: &str) -> anyhow::Result<()> {
        self.add_lump_raw(tinywad::lump::LumpAdd::new(
            tinywad::lump::LumpAddKind::Back,
            &buffer,
            name,
        ))?;
        Ok(())
    }
}

pub fn consume_images(
    name: &str,
    sprite: &str,
    mugshot: &str,
    wad: &mut Wad,
    crc: &Crc32,
    temp: &Path,
    as_skins: bool,
    index: usize,
) -> anyhow::Result<()> {
    let crouch_sprite = &{
        let mut crouch_sprite = sprite.to_string();
        crouch_sprite.replace_range(3..4, "[");
        crouch_sprite
    };

    if as_skins {
        generate_skins_wad(name, sprite, crouch_sprite, mugshot, wad)?;
    } else {
        generate_players_wad(name, sprite, crouch_sprite, mugshot, wad, index)?;
    }

    grab_into_wad("sprites", "w / 2", "h - 15", crc, wad, temp)?;
    grab_into_wad("crouch_sprites", "w / 2", "h - 15", crc, wad, temp)?;
    grab_into_wad("mugshots", "w / 2 - 18", "h / 2 - 17", crc, wad, temp)?;

    if !as_skins {
        grab_into_wad("fists", "-w / 2 - 15", "-h / 2 + 3", crc, wad, temp)?;
        wad.push_lump(&vec![], "S_END")?;
    }

    Ok(())
}

fn generate_skins_wad(
    name: &str,
    sprite: &str,
    crouch_sprite: &str,
    mugshot: &str,
    wad: &mut Wad,
) -> anyhow::Result<()> {
    wad.push_lump(
        &format!(
            "name = \"{name}\"\nsprite = {sprite}\ncrouchsprite = {crouch_sprite}\nface = {mugshot}\nscale = 0.5"
        )
        .as_bytes()
        .to_vec(),
        "S_SKIN",
    )?;

    Ok(())
}

fn generate_players_wad(
    name: &str,
    sprite: &str,
    crouch_sprite: &str,
    mugshot: &str,
    wad: &mut Wad,
    index: usize,
) -> anyhow::Result<()> {
    let mut decorate = indoc::formatdoc!(
        r#"
        ACTOR Crafter_{index} : DoomPlayer {{
            Scale 0.5
            Player.DisplayName "{name}"
            Player.Face "{mugshot}"
            Player.CrouchSprite "{crouch_sprite}"
            Player.StartItem "Pistol"
            Player.StartItem "Fist_{index}"
            Player.StartItem "Clip", 50
            Player.WeaponSlot 1, Fist_{index}, Chainsaw
            Player.WeaponSlot 2, Pistol
            Player.WeaponSlot 3, Shotgun, SuperShotgun
            Player.WeaponSlot 4, Chaingun
            Player.WeaponSlot 5, RocketLauncher
            Player.WeaponSlot 6, PlasmaRifle
            Player.WeaponSlot 7, BFG9000

            States {{
                Spawn:
                    {sprite} A -1
                    Loop
                See:
                    {sprite} ABCD 4 
                    Loop
                Missile:
                    {sprite} E 12
                    Goto Spawn
                Melee:
                    {sprite} F 6 BRIGHT
                    Goto Missile
                Pain:
                    {sprite} G 4 
                    {sprite} G 4 A_Pain
                    Goto Spawn
                Death:
                    {sprite} H 0 A_PlayerSkinCheck("AltSkinDeath")
                Death1:
                    {sprite} H 10
                    {sprite} I 10 A_PlayerScream
                    {sprite} J 10 A_NoBlocking
                    {sprite} KLM 10
                    {sprite} N -1
                    Stop
                XDeath:
                    {sprite} O 0 A_PlayerSkinCheck("AltSkinXDeath")
                XDeath1:
                    {sprite} O 5
                    {sprite} P 5 A_XScream
                    {sprite} Q 5 A_NoBlocking
                    {sprite} RSTUV 5
                    {sprite} W -1
                    Stop
                AltSkinDeath:
                    {sprite} H 6
                    {sprite} I 6 A_PlayerScream
                    {sprite} JK 6
                    {sprite} L 6 A_NoBlocking
                    {sprite} MNO 6
                    {sprite} P -1
                    Stop
                AltSkinXDeath:
                    {sprite} Q 5 A_PlayerScream
                    {sprite} R 0 A_NoBlocking
                    {sprite} R 5 A_SkullPop
                    {sprite} STUVWX 5
                    {sprite} Y -1
                Stop
            }}
        }}
        "#
    );
    decorate += "\n";
    let mut sprite = sprite.to_string();
    sprite.replace_range(3..4, "]");
    sprite = format!("\"{sprite}\"");
    decorate += &fists::generate_fist(&sprite, index);

    wad.push_lump(&decorate.as_bytes().to_vec(), &format!("DEC_M{index}"))?;
    wad.push_lump(&vec![], "S_Start")?;

    Ok(())
}

pub fn grab_into_wad(
    subpath: &str,
    x: &str,
    y: &str,
    crc: &Crc32,
    wad: &mut Wad,
    temp: &Path,
) -> anyhow::Result<()> {
    let mut paths = std::fs::read_dir(temp.join(subpath))
        .map(|d| d.map(|p| p.unwrap().path().to_str().unwrap().to_string()))?
        .collect::<Vec<_>>();
    paths.sort();
    putpng::grab::grab_all(
        paths.iter().map(|s| s.clone()),
        &crc,
        x.to_string(),
        y.to_string(),
        true,
    )
    .unwrap();
    putpng::crop::crop_all(paths.iter().map(|s| s.clone()), &crc).unwrap();
    for path in paths {
        wad.push_lump(
            &std::fs::read(&path)?,
            &Path::new(&path).file_stem().unwrap().to_str().unwrap(),
        )?;
    }

    Ok(())
}
