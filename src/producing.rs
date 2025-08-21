use crate::converting::{SpritePrefix, get_acc};
use anyhow::Context;
use putpng::crc::Crc32;
use std::path::Path;
use tinywad::{
    lump::{LumpAdd, LumpAddKind},
    models::operation::WadOp,
    wad::{Wad, WadKind},
};

trait Archive {
    fn new_archive() -> Self;

    fn push_lump(&mut self, buffer: &Vec<u8>, name: &str) -> anyhow::Result<()>;

    fn grab_from(
        &mut self,
        rendered_dir: &Path,
        subdir: &str,
        index: usize,
        x: &str,
        y: &str,
        crc: &Crc32,
    ) -> anyhow::Result<()> {
        let mut paths = std::fs::read_dir(rendered_dir.join(&format!("{subdir}{index}")))
            .map(|d| d.map(|p| p.unwrap().path().to_str().unwrap().to_string()))
            .with_context(|| format!("subdirectory {subdir}{index}"))?
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
            self.push_lump(
                &std::fs::read(&path).with_context(|| path.as_str().to_string())?,
                &Path::new(&path).file_stem().unwrap().to_str().unwrap(),
            )?;
        }
        Ok(())
    }

    fn populate_s_skin(
        &mut self,
        rendered_dir: &Path,
        name: &str,
        sprite_prefix: &str,
        index: usize,
        crc: &Crc32,
    ) -> anyhow::Result<()> {
        let sprite = sprite_prefix.to_skin_sprite();
        let crouch_sprite = sprite_prefix.to_crouched_skin_sprite();
        let mugshot = sprite_prefix.to_mugshot_sprite();
        self.push_lump(
            &format!(
                "name = \"{name}\"\nsprite = {sprite}\ncrouchsprite = {crouch_sprite}\nface = {mugshot}\nscale = 0.5"
            )
            .into_bytes(),
            "S_SKIN",
        )?;
        self.grab_from(rendered_dir, "sprites", index, "w / 2", "h - 15", crc)?;
        self.grab_from(
            rendered_dir,
            "crouch-sprites",
            index,
            "w / 2",
            "h - 15",
            crc,
        )?;
        self.grab_from(
            rendered_dir,
            "mugshot",
            index,
            "w / 2 - 18",
            "h / 2 - 17",
            crc,
        )?;

        Ok(())
    }
}

impl Archive for Wad {
    fn new_archive() -> Self {
        let mut wad = Wad::new();
        wad.set_kind(WadKind::Pwad);
        wad
    }

    fn push_lump(&mut self, buffer: &Vec<u8>, name: &str) -> anyhow::Result<()> {
        let name = name.replace("^", "\\");
        self.add_lump_raw(LumpAdd::new(LumpAddKind::Back, &buffer, &name))?;
        Ok(())
    }
}

pub fn produce_s_skin_wad(
    rendered_dir: &Path,
    produced_file: &Path,
    names_and_sprite_prefixes: Vec<(String, String)>,
    crc: &Crc32,
) -> anyhow::Result<()> {
    let mut wad = Wad::new_archive();
    for (index, (name, sprite_prefix)) in names_and_sprite_prefixes.into_iter().enumerate() {
        wad.populate_s_skin(rendered_dir, &sprite_prefix, &name, index, crc)?;
    }
    wad.save(produced_file);
    Ok(())
}

pub fn produce_s_skin_and_fist_wads(
    rendered_dir: &Path,
    produced_file: &Path,
    names_and_sprite_prefixes: Vec<(String, String)>,
    crc: &Crc32,
) -> anyhow::Result<()> {
    let mut wad = Wad::new_archive();
    let mut fist_wad = Wad::new_archive();
    let mut decorate = String::new();
    let mut index = 0;
    fist_wad.push_lump(&vec![], "S_START")?;
    for (name, sprite_prefix) in names_and_sprite_prefixes {
        wad.populate_s_skin(rendered_dir, &name, &sprite_prefix, index, crc)?;
        fist_wad.grab_from(
            rendered_dir,
            "fist",
            index,
            "-w / 2 - 15",
            "-h / 2 + 3",
            crc,
        )?;
        decorate += &generate_fist_decorate(&sprite_prefix.to_fist_sprite(), index);
        index += 1;
    }
    decorate.pop();
    fist_wad.push_lump(&vec![], "S_END")?;
    fist_wad.push_lump(&decorate.into_bytes(), "DECORATE")?;
    let acs = indoc::formatdoc!(
        r#"
        #library "pickfist"
        #include "zcommon.acs"

        script "PickFist" ENTER {{
            int lastSkin = 0;
            while (true) {{
                int skin = getUserCVar(playerNumber(), "skin");
                if (skin != lastSkin) {{
                    if (lastSkin != 0 && lastSkin <= {index})
                        takeInventory(strParam(s: "Fist", d: lastSkin - 1), 1);
                    else
                        takeInventory("Fist", 1);

                    if (skin != 0 && skin <= {index})
                        giveInventory(strParam(s: "Fist", d: skin - 1), 1);
                    else
                        giveInventory("Fist", 1);
                
                    lastSkin = skin;
                }}
                delay(35);
            }}
        }}
        "#
    );
    std::fs::write("pick-fist.acs", acs.clone()).unwrap();
    std::process::Command::new(get_acc().unwrap())
        .arg("pick-fist")
        .output()
        .unwrap();
    fist_wad.push_lump(&acs.into_bytes(), "PICKFIST")?;
    fist_wad.push_lump(&"PICKFIST".as_bytes().to_vec(), "LOADACS")?;
    fist_wad.push_lump(&vec![], "A_START")?;
    fist_wad.push_lump(&std::fs::read("pick-fist.o").unwrap(), "PICKFIST")?;
    fist_wad.push_lump(&vec![], "A_END")?;
    fist_wad.push_lump(
        &"[enu default]\nFIST = \"Fist\";\0".as_bytes().to_vec(),
        "LANGUAGE",
    )?;
    std::fs::remove_file("pick-fist.acs")?;
    std::fs::remove_file("pick-fist.o")?;
    wad.save(produced_file);
    fist_wad.save(produced_file.to_str().unwrap().replace('.', "-fist."));
    Ok(())
}

pub fn produce_decorate_wad(
    rendered_dir: &Path,
    produced_file: &Path,
    names_and_sprite_prefixes: Vec<(String, String)>,
    crc: &Crc32,
) -> anyhow::Result<()> {
    let mut wad = Wad::new_archive();
    let mut decorate = String::new();
    let mut mapinfo = "GameInfo {\n    PlayerClasses = ".to_string();
    wad.push_lump(&vec![], "S_START")?;
    for (index, (name, sprite_prefix)) in names_and_sprite_prefixes.into_iter().enumerate() {
        let sprite = sprite_prefix.to_skin_sprite().quoted();
        let crouch_sprite = sprite_prefix.to_crouched_skin_sprite();
        let mugshot = sprite_prefix.to_mugshot_sprite();
        let fist = sprite_prefix.to_fist_sprite();
        decorate += &indoc::formatdoc!(
            r#"
            ACTOR Crafter{index} : DoomPlayer {{
                Player.DisplayName "{name}"
                Player.Face "{mugshot}"
                Player.CrouchSprite "{crouch_sprite}"
                Player.StartItem "Pistol"
                Player.StartItem "Fist{index}"
                Player.StartItem "Clip", 50
                Player.WeaponSlot 1, Fist{index}, Chainsaw
                Player.WeaponSlot 2, Pistol
                Player.WeaponSlot 3, Shotgun, SuperShotgun
                Player.WeaponSlot 4, Chaingun
                Player.WeaponSlot 5, RocketLauncher
                Player.WeaponSlot 6, PlasmaRifle
                Player.WeaponSlot 7, BFG9000
                Scale 0.5

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
        decorate += &generate_fist_decorate(&fist, index);
        mapinfo += &format!("\"Crafter{index}\", ");
        wad.grab_from(rendered_dir, "sprites", index, "w / 2", "h - 15", crc)?;
        wad.grab_from(
            rendered_dir,
            "crouch-sprites",
            index,
            "w / 2",
            "h - 15",
            crc,
        )?;
        wad.grab_from(
            rendered_dir,
            "mugshot",
            index,
            "w / 2 - 18",
            "h / 2 - 17",
            crc,
        )?;
        wad.grab_from(
            rendered_dir,
            "fist",
            index,
            "-w / 2 - 15",
            "-h / 2 + 3",
            crc,
        )?;
    }
    decorate.pop();
    mapinfo.pop();
    mapinfo.pop();
    mapinfo += "\n}";
    wad.push_lump(&vec![], "S_END")?;
    wad.push_lump(&decorate.into_bytes(), "DECORATE")?;
    wad.push_lump(&mapinfo.into_bytes(), "MAPINFO")?;

    wad.save(produced_file);
    Ok(())
}

fn generate_fist_decorate(sprite: &str, index: usize) -> String {
    let sprite = format!("\"{sprite}\"");
    indoc::formatdoc!(
        r#"
        ACTOR Fist{index} : Weapon replaces Fist {{
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
