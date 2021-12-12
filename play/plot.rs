use std::{
    fs::File,
    io::{BufReader, Read},
};

use anyhow::Result;
use brr_conv_lib::brr::BrrIterator;
use hound::WavReader;
use plotters::prelude::*;

#[allow(overflowing_literals)]

pub fn plot(orig: &str, brr: &str, skip: usize) -> Result<()> {
    let mut wav_file = WavReader::open(orig)?;
    let mut wav = wav_file.samples::<i16>().step_by(2).skip(skip); //step by because channels are interleaved
    let mut brr = BrrIterator::new(
        BufReader::new(File::open(brr)?)
            .bytes()
            .map(|r| r.expect("Error parsing or smthn")),
    )
    .skip(skip);

    let x_graph_size = 4096;
    let y_graph_size = 4096;

    let root = SVGBackend::new("plot.svg", (x_graph_size, y_graph_size)).into_drawing_area();
    root.fill(&WHITE)?;

    let x_range = 0f32..1000f32;
    let y_range = (i16::MIN as f32)..(i16::MAX as f32);

    let x_mod = (x_graph_size as f32) / ((x_range.start.abs() + x_range.end) as f32);
    let y_mod = (y_graph_size as f32) / ((y_range.start.abs() + y_range.end) as f32);

    let x_mid = (x_range.start * x_mod).abs().round() as i32;
    let y_mid = (y_range.end * y_mod).round() as i32;

    //draw out areas where brr notes can be placed.
    for i in 0..=11 {
        for j in -8..=7 {
            let possible_val = j * 2i32.pow(i);
            let possible_y = y_mid - ((possible_val as f32 * y_mod).round() as i32);
            root.draw(&PathElement::new(
                [
                    ((x_range.start * x_mod).round() as i32, possible_y),
                    ((x_range.end * x_mod).round() as i32, possible_y),
                ],
                HSLColor((i as f64) * 0.083, 0.5, 0.5),
            ))?;
        }
    }

    // draw zero line
    root.draw(&PathElement::new(
        [
            ((x_range.start * x_mod).round() as i32, y_mid),
            ((x_range.end * x_mod).round() as i32, y_mid),
        ],
        &BLACK,
    ))?;

    for i in 0..1000 {
        let curr_x = ((i as f32) * x_mod).round() as i32 + x_mid;

        if (i + skip) % 16 == 0 {
            root.draw(&PathElement::new(
                [(curr_x, 0), (curr_x, y_graph_size as i32)],
                &BLACK,
            ))?;
        }

        let wav_val = match wav.next() {
            Some(w) => w,
            None => break,
        }? as f32;
        let brr_val = brr.next().expect("Brr does not match wav file length") as f32;

        let wav_pt = (curr_x, y_mid - (wav_val * y_mod).round() as i32);
        let brr_pt = (curr_x, y_mid - (brr_val * y_mod).round() as i32);

        //draw points for each
        root.draw(&Circle::new(wav_pt, 4, RGBColor(141, 133, 255).filled()))?;
        root.draw(&Circle::new(brr_pt, 4, RGBColor(188, 255, 133).filled()))?;

        //draw line showing difference
        root.draw(&PathElement::new([wav_pt, brr_pt], &RED))?;
    }

    Ok(())
}
