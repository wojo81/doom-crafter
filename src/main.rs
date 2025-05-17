mod convert;
mod doom;
mod minecraft;

use std::io::Write;
use crate::convert::*;

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    let mut infos = vec![];

    'outer: loop {
        let mut path = String::new();
        let mut name = String::new();
        let mut sprite = String::new();

        loop {
            path.clear();
            print!("Enter path of next minecraft skin(empty to stop): ");
            stdout.flush()?;
            stdin.read_line(&mut path)?;
            path = path.trim().into();
            remove_quotes(&mut path);
            println!("{path}");
            if path.is_empty() {
                break 'outer;
            } else if validate_path(&path, &mut stdout)? {
                break;
            }
        }

        print!("Enter name of skin: ");
        stdout.flush()?;
        stdin.read_line(&mut name)?;
        name = name.trim().into();

        loop {
            sprite.clear();
            print!("Enter sprite name for skin: ");
            stdout.flush()?;
            stdin.read_line(&mut sprite)?;
            sprite = sprite.trim().into();
            if validate_sprite(&sprite) {
                break;
            }
        }

        infos.push(SkinInfo { path, name, sprite });
    }

    if !infos.is_empty() {
        let mut file_name = String::new();
        print!("Enter file name: ");
        stdout.flush()?;
        stdin.read_line(&mut file_name)?;
        file_name = file_name.trim().into();
        convert_all(&infos, file_name.clone())?;

        println!("{file_name} created successfully");
        stdout.flush().unwrap();
        let mut foo = String::new();
        stdin.read_line(&mut foo).unwrap();
    }

    Ok(())
}

fn remove_quotes(s: &mut String) {
    if s.starts_with('\'') {
        let _ = s.remove(0);
    }
    if s.ends_with('\'') {
        let _ = s.remove(s.len() - 1);
    }
}

fn validate_path(path: impl AsRef<std::path::Path>, stdout: &mut std::io::Stdout) -> std::io::Result<bool> {
    let path = path.as_ref();
    if !path.exists() {
        println!("{path:?} does not exist");
        stdout.flush()?;
        Ok(false)
    } else if path.extension().unwrap() != "png" {
        println!("path entered needs to be a png file");
        Ok(false)
    } else {
        Ok(true)
    }
}

fn validate_sprite(sprite: &str) -> bool {
    if sprite.len() != 4 {
        println!("sprite name can only be 4 characters long");
        return false;
    }
    for c in sprite.chars() {
        if !c.is_ascii() && !c.is_alphabetic() && c != '[' && c != ']' && c != '\\' {
            println!("sprite name can only consist of alphabetic characters or '[', ']', '\\'");
            return false;
        }
    }
    true
}