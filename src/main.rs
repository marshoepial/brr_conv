use std::path::Path;

use anyhow::Result;

use brr_conv_lib::convert;
use clap::App;

fn main() -> Result<()> {
    println!("BRR Conversion Tool");

    let args = App::new("brr_conv")
        .version("1.0")
        .author("marshoepial <marshoepial@gmail.com>")
        .about("Converts wav files into SPC700 brr format.")
        .args_from_usage("-i --in <FILE> 'file to be converted'")
        .get_matches();

    convert(Path::new(args.value_of("in").expect("Must have input file")), true)?;

    Ok(())
}
