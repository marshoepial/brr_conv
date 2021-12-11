pub mod brr;

use std::{collections::HashMap, fs::File, io::Write, path::Path};

use anyhow::Result;
use hound::WavReader;
use indicatif::ProgressBar;
use nibble_vec::Nibblet;

pub fn convert(in_file: &Path, output: bool) -> Result<()> {
    let mut reader = WavReader::open(in_file)?;

    let file_stem = in_file
        .file_stem()
        .expect("Not a valid filename")
        .to_str()
        .expect("Filename is not valid unicode");
    let mut left_output = File::create(format!("{}_left.brr", file_stem))?;
    let mut right_output = File::create(format!("{}_right.brr", file_stem))?;

    let mut sample_iter = reader.samples::<i16>();
    let samples_len = sample_iter.len();

    let progress: ProgressBar;
    if output {
        println!("Started audio stream");
        progress = ProgressBar::new(samples_len.try_into()?);
    } else {
        progress = ProgressBar::hidden();
    }

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

fn get_next_nibbles<T: Iterator<Item = Result<i16, hound::Error>>>(
    samples: &mut T,
) -> (Vec<u8>, Vec<u8>) {
    let samples: Vec<i16> = samples
        .take(32)
        .map(|i| i.expect("Error while reading wav"))
        .collect();

    // Return early if there are no more samples to process
    if samples.is_empty() {
        return (Vec::new(), Vec::new());
    }
        
    let left_shifts: Vec<u32> = samples
        .iter()
        .step_by(2)
        .map(calc_shift)
        .collect();
    let right_shifts: Vec<u32> = samples
        .iter()
        .skip(1)
        .step_by(2)
        .map(calc_shift)
        .collect();
        
    let mut left_counts = HashMap::new();
    let left_opt_shift = left_shifts.iter() 
        .max_by_key(|&&shift| {
            let count = left_counts.entry(shift).or_insert(0);
            *count += 1;
            *count
        })
        .unwrap();    
    let mut right_counts = HashMap::new();
    let right_opt_shift = right_shifts.iter() 
        .max_by_key(|&&shift| {
            let count = right_counts.entry(shift).or_insert(0);
            *count += 1;
            *count
        })
        .unwrap();
    

    // Nibble vecs let us store sets of nibbles safely, and then convert them to u8 sets when done
    let mut left_nibbles = Nibblet::new();
    let mut right_nibbles = Nibblet::new();

    for i in 0..samples.len() {
        if i % 2 == 0 {
            left_nibbles.push(match left_shifts[i/2].cmp(left_opt_shift) {
                std::cmp::Ordering::Greater => {
                    if samples[i] > 0 {
                        0b00000111
                    } else {
                        0b00001000
                    }
                },
                std::cmp::Ordering::Equal => (samples[i] >> left_opt_shift) as u8,
                std::cmp::Ordering::Less => {
                    if (1 << left_opt_shift) - samples[i].abs() > samples[i].abs() {
                        0
                    } else if samples[i] < 0 {
                        0b00001111
                    } else {
                        1
                    }
                }
            });
        } else {
            right_nibbles.push(match right_shifts[i/2].cmp(right_opt_shift) {
                // shifting a number that doesn't fit into the nibble right can cause unpredictable results
                // give the maximum instead
                std::cmp::Ordering::Greater => {
                    if samples[i] > 0 {
                        0b00000111
                    } else {
                        0b00001000
                    }
                },
                std::cmp::Ordering::Equal => (samples[i] >> right_opt_shift) as u8,
                std::cmp::Ordering::Less => {
                    if (1 << right_opt_shift) - samples[i].abs() > samples[i].abs() {
                        0
                    } else if samples[i] < 0 {
                        0b00001111
                    } else {
                        1
                    }
                }
            });
        }
    }

    let mut left_brr_samples = vec![(*left_opt_shift as u8) << 4];
    left_brr_samples.append(&mut left_nibbles.into_bytes());

    let mut right_brr_samples = vec![(*right_opt_shift as u8) << 4];
    right_brr_samples.append(&mut right_nibbles.into_bytes());

    (left_brr_samples, right_brr_samples)
}

fn calc_shift(sample: &i16) -> u32 {
    // SPC uses two's complement for storing singed numbers. Complement is taken before doing this operation if number is negative
    let mut shifted_sample = *sample;
    if sample < &0 {
        shifted_sample = !shifted_sample;
    }

    // Get number of leading zeroes so top 3 MSBs are preserved
    let zeroes = shifted_sample.leading_zeros().min(13);
    13 - zeroes
}
