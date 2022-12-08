use eyre::Result;
use std::fs;
use std::path::PathBuf;

pub fn save(babel_bin: Vec<u8>, babel_bin_path: &PathBuf) -> Result<()> {
    fs::write(babel_bin_path, babel_bin)?;
    Ok(())
}

pub fn checksum(babel_bin_path: &PathBuf) -> Result<u32> {
    let babel_bin = fs::read(babel_bin_path)?;

    let crc = crc::Crc::<u32>::new(&crc::CRC_32_BZIP2);
    let mut digest = crc.digest();
    digest.update(&babel_bin);
    Ok(digest.finalize())
}
