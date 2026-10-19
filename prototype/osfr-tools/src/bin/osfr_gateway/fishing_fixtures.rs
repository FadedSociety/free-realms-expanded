//! Wire builders shared by the gateway fishing regressions. Both the gateway binary's
//! `#[cfg(test)]` module and the `fishing_entry` integration test include this file by path, so
//! it may use only the standard library and must keep every item `pub`.

use std::{fs, path::Path};

pub fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

pub fn login(name: &str) -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for (index, value) in [name, "1.910.1.530630", ""].into_iter().enumerate() {
        bytes.extend((value.len() as u32).to_le_bytes());
        bytes.extend(value.bytes());
        if index == 0 {
            bytes.extend(1u64.to_le_bytes());
        }
    }
    bytes
}

pub fn npc(sub: u16, target: u64, choice: Option<i32>) -> Vec<u8> {
    let mut bytes = [26u16.to_le_bytes(), sub.to_le_bytes()].concat();
    bytes.extend(target.to_le_bytes());
    match choice {
        Some(choice) => bytes.extend(choice.to_le_bytes()),
        None => bytes.push(0),
    }
    bytes
}

pub fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// The option ids of a 26/9 menu whose label is `string_id`.
pub fn fish_option(menu: &[Vec<u8>], string_id: u32) -> Option<i32> {
    let packet = menu
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))?;
    let count = i32::from_le_bytes(packet[13..17].try_into().ok()?) as usize;
    (0..count).find_map(|index| {
        let at = 17 + index * 28;
        let id = i32::from_le_bytes(packet[at..at + 4].try_into().ok()?);
        let label = u32::from_le_bytes(packet[at + 8..at + 12].try_into().ok()?);
        (label == string_id).then_some(id)
    })
}
