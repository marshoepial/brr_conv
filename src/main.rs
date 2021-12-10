use std::{fs::File, io::Write, collections::HashMap};

use anyhow::Result;
use hound::WavReader;
use indicatif::ProgressBar;
use nibble_vec::Nibblet;

fn main() -> Result<()> {
    println!("BRR Conversion Tool");

    let mut reader = WavReader::open("audio.wav")?;
    let mut left_output = File::create("audio_left.brr")?;
    let mut right_output = File::create("audio_right.brr")?;

    let mut sample_iter = reader.samples::<i16>();
    let samples_len = sample_iter.len();

    println!("Started audio stream");

    let progress = ProgressBar::new(samples_len.try_into()?);

    let mut iteration = 0;
    let mut samples = 0;
    loop {
        // println!("Loop {}, samples {}-{} out of {}", iteration, samples, samples_len.min(samples+32), &samples_len);

        let brr_chunk = get_next_nibbles(&mut sample_iter);

        if brr_chunk.0.is_empty() {
            break;
        }

        left_output.write_all(&brr_chunk.0)?;
        right_output.write_all(&brr_chunk.1)?;

        progress.inc(32);
        iteration += 1;
        samples += 32;
    }

    Ok(())
}

fn get_next_nibbles<T: Iterator<Item = Result<i16, hound::Error>>>(samples: &mut T) -> (Vec<u8>, Vec<u8>) {
    let samples: Vec<i16> = samples.take(32).map(|i| i.expect("Error while reading wav")).collect();

    // Return early if there are no more samples to process
    if samples.is_empty() {
        return (Vec::new(), Vec::new())
    }

    let mut left_counts = HashMap::new();
    let mut right_counts = HashMap::new();
    let left_opt_shift = samples.iter().step_by(2).map(|sample| calc_shift(sample)).max_by_key(|&shift| {
        let count = left_counts.entry(shift).or_insert(0);
        *count += 1;
        *count
    }).unwrap();
    let right_opt_shift = samples.iter().step_by(2).map(|sample| calc_shift(sample)).max_by_key(|&shift| {
        let count = right_counts.entry(shift).or_insert(0);
        *count += 1;
        *count
    }).unwrap();

    // Nibble vecs let us store sets of nibbles safely, and then convert them to u8 sets when done
    let mut left_nibbles= Nibblet::new();
    let mut right_nibbles = Nibblet::new();

    let mut on_left = true;
    for sample in &samples {
        if on_left {
            left_nibbles.push((sample >> left_opt_shift) as u8);
        } else {
            right_nibbles.push((sample >> right_opt_shift) as u8);
        }

        on_left = !on_left;
    }
    
    let mut left_brr_samples = vec![(left_opt_shift as u8) << 4];
    left_brr_samples.append(&mut left_nibbles.into_bytes());

    let mut right_brr_samples = vec![(right_opt_shift as u8) << 4];
    right_brr_samples.append(&mut right_nibbles.into_bytes());

    (left_brr_samples, right_brr_samples)
}

fn calc_shift(sample: &i16) -> u32 {
    // SPC uses two's complement for storing singed numbers. Complement is taken before doing this operation if number is negative
    let mut shifted_sample = sample.clone();
    if sample < &0 {
        shifted_sample = !shifted_sample;
    }

    // Get number of leading zeroes so top 3 MSBs are preserved
    let zeroes = shifted_sample.leading_zeros().min(13);
    13 - zeroes
}