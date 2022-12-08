use anyhow::{Context, Result};
use std::{env, fs};

pub fn load() -> Result<(Vec<u8>, u32)> {
    let bin_path =
        fs::canonicalize(env::current_exe().with_context(|| "failed to get current binary path")?)
            .with_context(|| "non canonical current binary path")?;
    let bin_dir = bin_path
        .parent()
        .with_context(|| "invalid current binary dir")?;
    let babel_bin = fs::read(bin_dir.join("../../babel/bin/babel"))
        .with_context(|| "failed to load babel binary")?;
    let crc = crc::Crc::<u32>::new(&crc::CRC_32_BZIP2);
    let mut digest = crc.digest();
    digest.update(&babel_bin);
    Ok((babel_bin, digest.finalize()))
}
