use std::path::Path;

use anyhow::Result;

use brr_conv_lib::convert;

fn main() -> Result<()> {
    println!("BRR Conversion Tool");

    convert(Path::new("audio.wav"), true)?;

    Ok(())
}
