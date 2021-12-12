mod plot;

use plot::plot;

use std::{
    fs::File,
    io::{BufReader, Read},
};

use anyhow::Result;
use brr_conv_lib::brr::BrrIterator;
use clap::{App, SubCommand};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize, SampleRate, StreamConfig,
};

fn main() -> Result<()> {
    let args = App::new("brrPlay")
        .version("1.0")
        .author("marshoepial <marshoepial@gmail.com>")
        .about("Plays brr files generated by brrConv. No guarantees about other brr files")
        .subcommand(SubCommand::with_name("play")
            .args_from_usage("-l --left <FILE> 'Specify brr to play in the left channel'
                            -r --right <FILE> 'Specify brr to play in the right channel'")
        ).subcommand(SubCommand::with_name("plot")
            .about("plots the waveform of the original and generated file, highlighting differences. Mostly for testing")
            .args_from_usage("-o --original <FILE> 'Original wav file'
                            -b --brr <FIILE> 'Brr file to plot'
                            -s --skip [skip] 'Frames to skip'")
                        )
        .get_matches();

    match args.subcommand() {
        ("play", Some(sub_m)) => play(
            sub_m
                .value_of("left")
                .expect("File needed for left channel"),
            sub_m
                .value_of("right")
                .expect("File needed for right channel"),
        )?,
        ("plot", Some(sub_m)) => plot(
            sub_m.value_of("original").expect("Original wav needed"),
            sub_m.value_of("brr").expect("Brr file needed"),
            sub_m
                .value_of("skip")
                .map(|s| s.parse::<usize>().expect("Could not parse skip value"))
                .unwrap_or_default(),
        )?,
        _ => panic!("Subcommand needed"),
    };

    Ok(())
}

fn play(left: &str, right: &str) -> Result<()> {
    let mut brr_left = BrrIterator::new(
        BufReader::new(File::open(left)?)
            .bytes()
            .map(|r| r.expect("Error parsing or smthn")),
    );
    let mut brr_right = BrrIterator::new(
        BufReader::new(File::open(right)?)
            .bytes()
            .map(|r| r.expect("Error parsing or smthn")),
    );

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("No output devices available");
    let config = StreamConfig {
        channels: 2,
        sample_rate: SampleRate(48000),
        buffer_size: BufferSize::Default,
    };

    let mut sample_counter = 2;
    let mut prev_samples = (0, 0);
    let stream = device.build_output_stream(
        &config,
        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
            for (i, sample) in data.iter_mut().enumerate() {
                if sample_counter != 0 {
                    if i % 2 == 0 {
                        *sample = brr_left.next().unwrap_or(0);
                        prev_samples.0 = *sample;
                    } else {
                        *sample = brr_right.next().unwrap_or(0);
                        prev_samples.1 = *sample;
                        sample_counter -= 1;
                    }
                } else if i % 2 == 0 {
                    *sample = prev_samples.0;
                } else {
                    *sample = prev_samples.1;
                    sample_counter = 2;
                }
            }
        },
        |err| {},
    )?;

    stream.play().unwrap();

    std::thread::sleep(std::time::Duration::from_millis(1000000));

    Ok(())
}
