use std::{
    fmt::{self, Display},
    io::{Cursor, Read},
    iter::{self, Peekable},
};

use anyhow::{Context, Result, bail};
use audioadapter_buffers::direct::InterleavedSlice;
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use itertools::Itertools;
use log::{debug, info};
use rubato::{Fft, Resampler};

pub const DEFAULT_OUT_RATE: u32 = 44100;
pub const DEFAULT_SILENCE_THRESHOLD: f32 = 0.005;
pub const DEFAULT_BITS_PER_SAMPLE: u16 = 16;

fn mono(left: f32, right: f32) -> f32 {
    (left + right) / 2.
}

#[derive(Debug, Clone)]
struct HoundErrorStr(String);

impl From<hound::Error> for HoundErrorStr {
    fn from(value: hound::Error) -> Self {
        HoundErrorStr(format!("WAV processing error: {value}"))
    }
}

impl Display for HoundErrorStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for HoundErrorStr {}

fn make_samples_iter(
    wav_reader: WavReader<impl Read + 'static>,
) -> Box<dyn Iterator<Item = Result<f32, HoundErrorStr>>> {
    let spec = wav_reader.spec();
    match spec.sample_format {
        SampleFormat::Float => Box::new(
            wav_reader
                .into_samples::<f32>()
                .map(|s| s.map_err(HoundErrorStr::from)),
        ),
        SampleFormat::Int => {
            let samples = wav_reader.into_samples::<i32>();
            Box::new(
                samples
                    .map_ok(move |i| i as f32 / 2i64.pow(spec.bits_per_sample as u32) as f32)
                    .map(|s| s.map_err(HoundErrorStr::from)),
            )
        }
    }
}

fn trim_start(
    silence_threshold: f32,
    mut samples_iter: Peekable<impl Iterator<Item = Result<f32, HoundErrorStr>>>,
) -> Result<impl Iterator<Item = Result<f32, HoundErrorStr>>, HoundErrorStr> {
    while let Some(sample_res) = samples_iter.peek() {
        match sample_res {
            Ok(sample) => {
                if sample.abs() < silence_threshold {
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

fn trim_end(silence_threshold: f32, mut samples: Vec<f32>) -> Vec<f32> {
    let silence_end_n = samples
        .iter()
        .rev()
        .take_while(|s| **s < silence_threshold)
        .count();
    debug!("silent samples at the end: {silence_end_n}");

    samples.truncate(samples.len() - silence_end_n);

    samples
}

fn resample(config: &Config, input_rate: usize, samples: &[f32]) -> Result<Vec<f32>> {
    let channels = if config.stereo { 2 } else { 1 };
    let mut resampler = Fft::new(
        input_rate,
        config.rate as usize,
        1024,
        1,
        channels,
        rubato::FixedSync::Both,
    )?;

    let n_input_frames = samples.len() / channels;
    let input_adapter = InterleavedSlice::new(samples, channels, n_input_frames)?;

    let n_output_frames = resampler.process_all_needed_output_len(n_input_frames);
    let mut out_buf = vec![0.; n_output_frames * channels];
    let mut output_adapter = InterleavedSlice::new_mut(&mut out_buf, channels, n_output_frames)?;

    resampler.process_all_into_buffer(&input_adapter, &mut output_adapter, n_input_frames, None)?;

    Ok(out_buf)
}

/// Normalizes WAV sample bytes for uniform handling: trim, gain, mono and fixed
/// sample rate.
fn process_file(config: &Config, bytes: impl Read + 'static) -> Result<Vec<f32>> {
    let wav_reader = WavReader::new(bytes).context("reading wav bytes")?;
    let spec = wav_reader.spec();

    info!("{spec:?}");

    // f32 samples iterator adjusted by bitrate:
    let samples_iter = make_samples_iter(wav_reader);

    // Converts stereo to mono or vise versa depending on the config and source
    // data:
    let samples_iter = match (config.stereo, spec.channels) {
        (true, 1) => {
            // Processing mono to stereo
            Box::new(samples_iter.flat_map(|s| [s.clone(), s])) as Box<dyn Iterator<Item = _>>
        }
        (false, 2) => {
            // Processing stereo to mono
            Box::new(
                samples_iter
                    .batching(|it| match it.next() {
                        None => None,
                        Some(x) => match it.next() {
                            None => None,
                            Some(y) => Some((x, y)),
                        },
                    })
                    .map(|(left, right)| Ok(mono(left?, right?))),
            )
        }
        (true, 2) | (false, 1) => samples_iter,
        _ => bail!("WAV files with only up to two channels are supported"),
    };

    // Trim silence in the start:
    let mut samples_iter = trim_start(config.silence_threshold, samples_iter.peekable())?;

    // Find peak value, evaluating the iterator, so getting rid of `Result`:
    let (peak, samples) =
        samples_iter.try_fold((0., Vec::new()), |(peak, mut samples), s| -> Result<_> {
            let sample = s?;
            samples.push(sample);

            Ok((f32::max(peak, sample.abs()), samples))
        })?;

    // Trim silence in the end:
    let mut samples = trim_end(config.silence_threshold, samples);

    // Normalize volume:
    let scale = 1. / peak;
    samples.iter_mut().for_each(|s: &mut f32| {
        *s *= scale;
    });

    debug!("input samples count: {}", samples.len());

    let out = if spec.sample_rate != config.rate {
        resample(config, spec.sample_rate as usize, &samples)?
    } else {
        samples
    };

    debug!("output samples count: {}", out.len());

    Ok(out)
}

pub struct Config {
    pub silence_threshold: f32,
    pub stereo: bool,
    pub rate: u32,
    pub bits_per_sample: u16,
}

impl Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "silence threshold: {}, ", self.silence_threshold)?;
        write!(f, "sampling rate: {}, ", self.rate)?;
        write!(f, "bit depth: {} bits, ", self.bits_per_sample)?;
        f.write_str(if self.stereo { "stereo" } else { "mono" })?;

        Ok(())
    }
}

pub fn step_sample<R: Read + 'static>(
    config: Config,
    input: impl IntoIterator<Item = R>,
) -> Result<Vec<u8>> {
    let processed_samples: Vec<Vec<f32>> = input
        .into_iter()
        .map(|file| process_file(&config, file))
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
            channels: if config.stereo { 2 } else { 1 },
            sample_rate: config.rate,
            bits_per_sample: config.bits_per_sample,
            sample_format: SampleFormat::Int,
        },
    )
    .context("initializing wav writer")?;

    match config.bits_per_sample {
        16 => {
            for s in concat_sample {
                wav_writer.write_sample::<i16>((s * i16::MAX as f32) as i16)?;
            }
        }
        24 => {
            for s in concat_sample {
                // For 24-bit, hound uses i32 with upper 8 bits unused
                let mut sample = (s * 8388607.0) as i32; // 2^23 - 1
                sample = sample.clamp(-(1 << 23), (1 << 23) - 1);
                wav_writer.write_sample::<i32>(sample)?;
            }
        }
        _ => bail!(
            "Unsupported bit depth: {}. Only 16 and 24 bits are supported.",
            config.bits_per_sample
        ),
    }

    drop(wav_writer);

    Ok(cursor.into_inner())
}
