use std::{
    collections::BTreeSet,
    env,
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq)]
struct Asset {
    name: String,
    offset: u64,
    size: u64,
    crc: u32,
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn be32(input: &mut impl Read) -> io::Result<u32> {
    let mut bytes = [0; 4];
    input.read_exact(&mut bytes)?;
    Ok(u32::from_be_bytes(bytes))
}

// In the supplied packs each directory block starts with an absolute next-block
// offset and entry count. Entry fields and block pointers are big endian.
fn read_index(input: &mut (impl Read + Seek), file_size: u64) -> io::Result<Vec<Asset>> {
    let mut assets = Vec::new();
    let mut visited = BTreeSet::new();
    let mut block = 0u64;
    loop {
        if !visited.insert(block) || block.checked_add(8).is_none_or(|end| end > file_size) {
            return Err(invalid("invalid or cyclic directory block"));
        }
        input.seek(SeekFrom::Start(block))?;
        let next = u64::from(be32(input)?);
        let count = u64::from(be32(input)?);
        if count > (file_size - block - 8) / 16 {
            return Err(invalid("invalid entry count"));
        }
        for _ in 0..count {
            let length = be32(input)? as usize;
            if length == 0 || length > 4096 {
                return Err(invalid("invalid asset name length"));
            }
            let mut bytes = vec![0; length];
            input.read_exact(&mut bytes)?;
            let name = String::from_utf8(bytes).map_err(|_| invalid("asset name is not UTF-8"))?;
            if name.chars().any(char::is_control) {
                return Err(invalid("control character in asset name"));
            }
            let offset = u64::from(be32(input)?);
            let size = u64::from(be32(input)?);
            let crc = be32(input)?;
            if offset.checked_add(size).is_none_or(|end| end > file_size) {
                return Err(invalid("asset extends past end of pack"));
            }
            assets.push(Asset {
                name,
                offset,
                size,
                crc,
            });
        }
        if next == 0 {
            break;
        }
        block = next;
    }
    Ok(assets)
}

fn collect_packs(root: &Path, packs: &mut Vec<PathBuf>) -> io::Result<()> {
    for item in fs::read_dir(root)? {
        let item = item?;
        let kind = item.file_type()?;
        if kind.is_dir() {
            collect_packs(&item.path(), packs)?;
        } else if kind.is_file()
            && item
                .path()
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("pack"))
        {
            packs.push(item.path());
        }
    }
    Ok(())
}

fn crc_table() -> [u32; 256] {
    let mut table = [0; 256];
    for (i, cell) in table.iter_mut().enumerate() {
        let mut value = i as u32;
        for _ in 0..8 {
            value = (value >> 1) ^ if value & 1 != 0 { 0xedb88320 } else { 0 };
        }
        *cell = value;
    }
    table
}

fn copy_asset(
    input: &mut File,
    asset: &Asset,
    output: &Path,
    table: &[u32; 256],
) -> io::Result<()> {
    let mut file = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output)?,
    );
    input.seek(SeekFrom::Start(asset.offset))?;
    let mut remaining = asset.size;
    let mut crc = !0u32;
    let mut buffer = [0; 65536];
    while remaining > 0 {
        let count = remaining.min(buffer.len() as u64) as usize;
        input.read_exact(&mut buffer[..count])?;
        for &byte in &buffer[..count] {
            crc = (crc >> 8) ^ table[((crc ^ u32::from(byte)) & 255) as usize];
        }
        file.write_all(&buffer[..count])?;
        remaining -= count as u64;
    }
    file.flush()?;
    if !crc != asset.crc {
        return Err(invalid(&format!(
            "CRC mismatch for {} (export is incomplete/unverified)",
            asset.name
        )));
    }
    Ok(())
}

fn export_name(index: usize, name: &str) -> String {
    // Numeric names preserve duplicate/versioned entries and cannot escape the
    // output directory. catalog.tsv retains every original logical filename.
    let suffix = name.rsplit_once('.').map(|(_, ext)| ext).unwrap_or("");
    if !suffix.is_empty() && suffix.len() <= 16 && suffix.bytes().all(|b| b.is_ascii_alphanumeric())
    {
        format!("{index:06}.{suffix}")
    } else {
        format!("{index:06}.bin")
    }
}

fn run() -> io::Result<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() != 3 || (args[0] != "catalog" && args[0] != "extract") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: osfr-tools catalog CLIENT_DIR NEW_CATALOG.tsv | extract CLIENT_DIR NEW_OUTPUT_DIR",
        ));
    }
    let extract = args[0] == "extract";
    let root = fs::canonicalize(Path::new(&args[1]))?;
    let output = PathBuf::from(&args[2]);
    let mut packs = Vec::new();
    collect_packs(&root, &mut packs)?;
    packs.sort();
    if packs.is_empty() {
        return Err(invalid("no .pack files found"));
    }
    let catalog_path = if extract {
        fs::create_dir(&output)?;
        output.join("catalog.tsv")
    } else {
        output.clone()
    };
    let mut catalog = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(catalog_path)?,
    );
    writeln!(
        catalog,
        "pack\tentry\tname\toffset\tbytes\tcrc32\textracted_relative_path"
    )?;
    let table = crc_table();
    let mut total = 0;
    for (pack_index, path) in packs.iter().enumerate() {
        let mut file = File::open(path)?;
        let size = file.metadata()?.len();
        let assets = read_index(&mut file, size)?;
        let folder = format!("pack_{pack_index:02}");
        if extract {
            fs::create_dir(output.join(&folder))?;
        }
        for (index, asset) in assets.iter().enumerate() {
            let relative = format!("{folder}/{}", export_name(index, &asset.name));
            if extract {
                copy_asset(&mut file, asset, &output.join(&relative), &table)?;
            }
            writeln!(
                catalog,
                "{}\t{}\t{}\t{}\t{}\t{:08x}\t{}",
                path.strip_prefix(&root).unwrap().display(),
                index,
                asset.name,
                asset.offset,
                asset.size,
                asset.crc,
                relative
            )?;
        }
        total += assets.len();
        eprintln!(
            "{}: {} entries{}",
            path.file_name().unwrap().to_string_lossy(),
            assets.len(),
            if extract { ", CRC verified" } else { "" }
        );
    }
    catalog.flush()?;
    eprintln!(
        "{} packs, {} entries{}",
        packs.len(),
        total,
        if extract {
            ", extraction complete"
        } else {
            ", catalog complete"
        }
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    fn fixture() -> Vec<u8> {
        let mut bytes = Vec::new();
        for word in [0u32, 1, 5] {
            bytes.extend(word.to_be_bytes());
        }
        bytes.extend(b"x.dds");
        for word in [29u32, 3, 0x352441c2] {
            bytes.extend(word.to_be_bytes());
        }
        bytes.extend(b"abc");
        bytes
    }
    #[test]
    fn reads_big_endian_directory() {
        let bytes = fixture();
        let assets = read_index(&mut Cursor::new(&bytes), bytes.len() as u64).unwrap();
        assert_eq!(
            assets,
            vec![Asset {
                name: "x.dds".into(),
                offset: 29,
                size: 3,
                crc: 0x352441c2
            }]
        );
    }
    #[test]
    fn rejects_out_of_bounds_asset_and_truncated_header() {
        let mut bytes = fixture();
        bytes[21..25].copy_from_slice(&99u32.to_be_bytes());
        assert!(read_index(&mut Cursor::new(&bytes), bytes.len() as u64).is_err());
        assert!(read_index(&mut Cursor::new([0u8; 7]), 7).is_err());
    }
    #[test]
    fn follows_linked_blocks_and_rejects_cycles() {
        let mut bytes = fixture();
        bytes[..4].copy_from_slice(&32u32.to_be_bytes());
        bytes.extend(0u32.to_be_bytes());
        bytes.extend(0u32.to_be_bytes());
        assert_eq!(
            read_index(&mut Cursor::new(&bytes), bytes.len() as u64)
                .unwrap()
                .len(),
            1
        );
        bytes[32..36].copy_from_slice(&32u32.to_be_bytes());
        assert!(read_index(&mut Cursor::new(&bytes), bytes.len() as u64).is_err());
    }
    #[test]
    fn crc32_matches_standard_vector() {
        let table = crc_table();
        let mut crc = !0u32;
        for byte in b"123456789" {
            crc = (crc >> 8) ^ table[((crc ^ u32::from(*byte)) & 255) as usize];
        }
        assert_eq!(!crc, 0xcbf43926);
    }
    #[test]
    fn export_names_never_include_asset_paths() {
        assert_eq!(export_name(3, "../../escape.dds"), "000003.dds");
        assert_eq!(export_name(4, "C:\\escape.evil/name"), "000004.bin");
    }
}
