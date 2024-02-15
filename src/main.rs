mod utils;

use clap::Parser;
use hound::{WavReader, WavWriter};
use once_cell::sync::Lazy;
use rustfft::num_complex::{Complex, ComplexFloat};
use rustfft::FftPlanner;
use std::collections::HashMap;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    input_wav_file: String,

    #[arg(long)]
    output_wav_file: String,

    #[arg(long, default_value_t = 50)]
    /// time window in ms
    window: u32,

    #[arg(long, default_value_t = false)]
    debug: bool,
}

static DTMF: Lazy<HashMap<(u16, u16), &str>> = Lazy::new(|| {
    HashMap::from([
        ((697, 1209), "1"),
        ((697, 1336), "2"),
        ((697, 1477), "3"),
        ((770, 1209), "4"),
        ((770, 1336), "5"),
        ((770, 1477), "6"),
        ((852, 1209), "7"),
        ((852, 1336), "8"),
        ((852, 1477), "9"),
        ((941, 1209), "*"),
        ((941, 1336), "0"),
        ((941, 1477), "#"),
        ((697, 1633), "A"),
        ((770, 1633), "B"),
        ((852, 1633), "C"),
        ((941, 1633), "D"),
    ])
});

fn main() {
    let args = Args::parse();

    let mut reader = WavReader::open(args.input_wav_file).expect("input_wav");

    let spec = reader.spec();
    let duration_ms = reader.duration() * 1000 / spec.sample_rate;
    let sample_len = reader.len() / spec.channels as u32;
    let sample_chunk_size = (sample_len / (duration_ms / args.window)) as usize;
    let dt = 1.0 / spec.sample_rate as f64;

    if args.debug {
        println!("channels: {}", spec.channels);
        println!("sample_rate: {}", spec.sample_rate);
        println!("bits_per_sample: {}", spec.bits_per_sample);
        println!("duration_ms: {}", duration_ms);
        println!("window: {}", args.window);
        println!("sample_chunk_size: {}", sample_chunk_size);
    }

    assert_eq!(spec.bits_per_sample, 16);
    let samples: Vec<i16> = reader.samples().map(|s| s.unwrap()).collect();
    let splitted = utils::split_wave(samples, spec.channels as usize)
        .iter()
        .map(|samples| process_dtmf(samples, sample_chunk_size, duration_ms, dt, args.debug))
        .collect();

    let samples = utils::join_wave(splitted);
    let mut writer = WavWriter::create(args.output_wav_file, spec).unwrap();

    for &sample in samples.iter() {
        writer.write_sample(sample).unwrap();
    }
}

fn process_dtmf(
    samples: &[i16],
    chunk_size: usize,
    duration_ms: u32,
    dt: f64,
    debug: bool,
) -> Vec<i16> {
    let mut buf = samples
        .iter()
        .map(|&i| Complex::from(i as f64))
        .collect::<Vec<_>>();

    let buf_len = buf.len();
    let last_index = buf_len - (buf_len % chunk_size);

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(chunk_size);
    fft.process(&mut buf[..last_index]);
    let freq = utils::fft_freq(chunk_size, dt);

    let mut detected_dtmf = vec![];
    for (i, chunk) in buf.chunks_mut(chunk_size).enumerate() {
        if chunk.len() != chunk_size {
            break;
        }

        let fft_abs = chunk.iter().map(|c| c.abs()).collect::<Vec<_>>();

        let find_best =
            |min_freq: f64, max_freq: f64, target_freqs: &[f64], allowed_delta: f64| -> f64 {
                let min_i = freq.iter().position(|&f| f > min_freq).unwrap_or(0);
                let max_i = freq
                    .iter()
                    .position(|&f| f > max_freq)
                    .unwrap_or(chunk_size - 1);
                let freq = &freq[min_i..max_i];
                let amp = &fft_abs[min_i..max_i];

                let max_amp =
                    amp.iter().enumerate().fold(
                        (0, 0.0),
                        |max, (i, &v)| {
                            if v > max.1 {
                                (i, v)
                            } else {
                                max
                            }
                        },
                    );
                let found = *freq.get(max_amp.0).unwrap_or(&0.0);

                let mut allowed_delta = allowed_delta;
                let mut best = 0.0;
                for &f in target_freqs.iter() {
                    let delta = (found - f).abs();
                    if delta < allowed_delta {
                        allowed_delta = delta;
                        best = f;
                    }
                }
                best
            };

        let allowed_delta = 20.0;
        let lf = find_best(0.0, 1050.0, &[697.0, 770.0, 852.0, 941.0], allowed_delta);
        let hf = find_best(
            1100.0,
            2000.0,
            &[1209.0, 1336.0, 1477.0, 1633.0],
            allowed_delta,
        );

        let dtmf_key = (lf as u16, hf as u16);
        if let Some(&dtmf) = DTMF.get(&dtmf_key) {
            if debug {
                let duration_delta = (duration_ms as usize / (buf_len / chunk_size)) as u32;
                let time = duration_delta * i as u32;
                println!("found {} at {}ms", dtmf, time);
            }

            detected_dtmf.push(Some(dtmf));
        } else {
            detected_dtmf.push(None);
        }
    }

    let mut buf = samples.to_vec();

    for (i, chunk) in buf.chunks_mut(chunk_size).enumerate() {
        if let Some(_dtmf) = detected_dtmf.get(i).unwrap_or(&None) {
            chunk.fill(0);
        }
    }

    return buf;
}
