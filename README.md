# Stepsampler

A Rust-based audio processing tool that creates evenly-sized sample grids by concatenating
multiple WAV files. This approach is commonly used in hardware samplers to pack many
sounds into one slot, enabling grid-based playback without manual chopping. It's
particularly useful for percussion sample packs where you want to fit dozens of hi-hats,
kicks, or snares onto a single pad.

## How does it work?

Stepsampler takes multiple audio samples and creates a single, evenly-sized sample by:

1. **Trimming silence** from the beginning and end of each sample
2. **Normalizing volume** across all samples
3. **Converting to mono** and standardizing to 44.1kHz 16-bit format
4. **Padding with silence** to make all samples the same length
5. **Concatenating** them into a single WAV file

## Usage

### CLI Usage

The command-line tool processes multiple WAV files and outputs a single concatenated
sample:

```bash
# Basic usage
stepsampler -o output.wav input1.wav input2.wav input3.wav

# Process all WAV files in a directory
stepsampler -o drum_kit.wav *.wav
```

### Web Interface

[Browser-based](https://fominok.github.io) version is available too.
