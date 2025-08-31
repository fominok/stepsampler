use std::{
    io::{Cursor, Read},
    iter::{self, Peekable},
};

use anyhow::{Context, Result, bail};
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use itertools::Itertools;
use log::{debug, info};
use rubato::{FftFixedInOut, Resampler};

const OUT_RATE: u32 = 44100;
const SILENCE_THRESHOLD: f32 = 0.005;

fn mono(left: f32, right: f32) -> f32 {
    (left + right) / 2.
}

fn make_samples_iter(
    wav_reader: WavReader<impl Read + 'static>,
) -> Box<dyn Iterator<Item = Result<f32, hound::Error>>> {
    let spec = wav_reader.spec();
    match spec.sample_format {
        SampleFormat::Float => Box::new(wav_reader.into_samples::<f32>()),
        SampleFormat::Int => {
            let samples = wav_reader.into_samples::<i32>();
            Box::new(
                samples.map_ok(move |i| i as f32 / 2i64.pow(spec.bits_per_sample as u32) as f32),
            )
        }
    }
}

fn trim_start(
    mut samples_iter: Peekable<impl Iterator<Item = Result<f32>>>,
) -> Result<impl Iterator<Item = Result<f32>>> {
    while let Some(sample_res) = samples_iter.peek() {
        match sample_res {
            Ok(sample) => {
                if sample.abs() < SILENCE_THRESHOLD {
                    samples_iter.next();
                } else {
                    break;
                }
            }
            Err(_) => {
                if let Some(Err(e)) = samples_iter.next() {
                    return Err(e);
                }
            }
        }
    }

    Ok(samples_iter)
}

fn trim_end(mut samples: Vec<f32>) -> Vec<f32> {
    let silence_end_n = samples
        .iter()
        .rev()
        .take_while(|s| **s < SILENCE_THRESHOLD)
        .count();
    debug!("silent samples at the end: {silence_end_n}");

    samples.truncate(samples.len() - silence_end_n);

    samples
}

fn resample(input_rate: usize, samples: &[f32]) -> Result<Vec<f32>> {
    let mut resampler = FftFixedInOut::<f32>::new(input_rate, OUT_RATE as usize, 1024, 1)
        .context("initializing resampled")?;

    let delay = resampler.output_delay();
    let mut input_buffer = &samples[..];
    let mut expecting_next = resampler.input_frames_next();
    let mut result = Vec::new();

    while input_buffer.len() >= expecting_next {
        result.extend_from_slice(
            &resampler
                .process(&[input_buffer], None)
                .context("resampling")?
                .pop()
                .expect("one channel of input was provided"),
        );
        input_buffer = &input_buffer[expecting_next..];
        expecting_next = resampler.input_frames_next();
    }

    result.extend_from_slice(
        &resampler
            .process_partial(Some(&[input_buffer]), None)
            .context("resampling")?
            .pop()
            .expect("one channel of input was provided"),
    );

    result.truncate(result.len() - delay);

    Ok(result)
}

/// Normalizes WAV sample bytes for uniform handling: trim, gain, mono and fixed
/// sample rate.
fn process_file(bytes: impl Read + 'static) -> Result<Vec<f32>> {
    let wav_reader = WavReader::new(bytes).context("reading wav bytes")?;
    let spec = wav_reader.spec();

    info!("{spec:?}");

    let channels = if spec.channels <= 2 {
        spec.channels
    } else {
        bail!("WAV files with only up to two channels are supported");
    };

    // f32 samples iterator adjusted by bitrate:
    let samples_iter = make_samples_iter(wav_reader);

    // Convert to mono by averaging left-right pairs:
    let chunks = samples_iter.chunks(channels as usize);
    let mono_samples_iter = chunks.into_iter().map(|mut chunk| -> Result<_> {
        let left = chunk
            .next()
            .expect("Itetools ensures non-empty chunks")
            .context("iterating over wav reader output")?;
        Ok(if let Some(right) = chunk.next() {
            mono(left, right.context("iterating over wav reader output")?)
        } else {
            left
        })
    });

    // Trim silence in the start:
    let mut samples_iter = trim_start(mono_samples_iter.peekable())?;

    // Find peak value, evaluating the iterator, so getting rid of `Result`:
    let (peak, samples) =
        samples_iter.try_fold((0., Vec::new()), |(peak, mut samples), s| -> Result<_> {
            let sample = s?;
            samples.push(sample);

            Ok((f32::max(peak, sample.abs()), samples))
        })?;

    // Trim silence in the end:
    let mut samples = trim_end(samples);

    // Normalize volume:
    let scale = 1. / peak;
    samples.iter_mut().for_each(|s: &mut f32| {
        *s *= scale;
    });

    debug!("input samples count: {}", samples.len());

    let out = if spec.sample_rate != OUT_RATE {
        resample(spec.sample_rate as usize, &samples)?
    } else {
        samples
    };

    debug!("output samples count: {}", out.len());

    Ok(out)
}

pub fn step_sample<R: Read + 'static>(input: impl IntoIterator<Item = R>) -> Result<Vec<u8>> {
    let processed_samples: Vec<Vec<f32>> = input
        .into_iter()
        .map(process_file)
        .collect::<Result<_>>()
        .context("processing sample")?;

    debug!("number of samples: {}", processed_samples.len());

    let max_length = processed_samples
        .iter()
        .map(|s| s.len())
        .max()
        .unwrap_or_default();

    debug!("max sample length: {max_length}");

    let concat_sample = processed_samples
        .into_iter()
        .map(|s| s.into_iter().chain(iter::repeat(0.)).take(max_length))
        .flatten();

    let mut cursor = Cursor::new(Vec::new());

    let mut wav_writer = WavWriter::new(
        &mut cursor,
        WavSpec {
            channels: 1,
            sample_rate: OUT_RATE,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        },
    )
    .context("initializing wav writer")?;

    for s in concat_sample {
        wav_writer.write_sample((s * i16::MAX as f32) as i16)?;
    }

    drop(wav_writer);

    Ok(cursor.into_inner())
}
