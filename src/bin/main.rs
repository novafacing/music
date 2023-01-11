/* This example expose parameter to pass generator of sample.
Good starting point for integration of cpal into your application.
*/

extern crate anyhow;
extern crate clap;
extern crate cpal;

use std::f32::consts::PI;
use std::thread::sleep;
use std::time::Duration;

use anyhow::Error;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, OutputCallbackInfo, Sample, Stream, StreamConfig, SupportedStreamConfig};
use music::note_to_freq;
use rust_music_theory::chord::{Chord, Number as ChordNumber, Quality as ChordQuality};
use rust_music_theory::note::{Notes, PitchClass};

pub enum Wave {
    Sine,
    Square,
    Saw,
    Triangle,
}

impl Wave {
    fn sample(&self, time: f32, freq: f32) -> f32 {
        match self {
            Wave::Sine => (time * freq * PI * 2.0).sin(),
            Wave::Square => {
                if (time * freq * PI * 2.0).sin() > 0.0 {
                    1.0
                } else {
                    -1.0
                }
            }
            Wave::Saw => {
                let mut sample = 0.0;
                for n in 1..100 {
                    sample += (time * freq * PI * 2.0 * n as f32).sin() / n as f32;
                }
                sample
            }
            Wave::Triangle => {
                let mut sample = 0.0;
                for n in 1..100 {
                    sample += (time * freq * PI * 2.0 * n as f32).sin() / (n as f32).powi(2);
                }
                sample
            }
        }
    }
}

pub struct SampleRequestOptions {
    pub sample_rate: f32,
    pub sample_clock: f32,
    pub nchannels: usize,
    pub sample_count: usize,
}

impl SampleRequestOptions {
    fn tone(&self, freq: f32, wave: Wave) -> f32 {
        wave.sample(self.sample_clock, freq)
    }
    fn tick(&mut self) {
        self.sample_clock = (self.sample_clock + 1.0) % self.sample_rate;

        if self.sample_clock == 0.0 {
            self.sample_count += 1;
        }
    }
}

pub fn stream_setup_for<F>(on_sample: F) -> Result<Stream, anyhow::Error>
where
    F: FnMut(&mut SampleRequestOptions) -> f32 + Send + 'static + Copy,
{
    let (_host, device, config) = host_device_setup()?;

    match config.sample_format() {
        cpal::SampleFormat::I16 => stream_make::<i16, _>(&device, &config.into(), on_sample),
        cpal::SampleFormat::U16 => stream_make::<u16, _>(&device, &config.into(), on_sample),
        cpal::SampleFormat::F32 => stream_make::<f32, _>(&device, &config.into(), on_sample),
    }
}

pub fn host_device_setup() -> Result<(Host, Device, SupportedStreamConfig), Error> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .ok_or_else(|| Error::msg("Default output device is not available"))?;
    println!("Output device : {}", device.name()?);

    let config = device.default_output_config()?;
    println!("Default output config : {:?}", config);

    Ok((host, device, config))
}

pub fn stream_make<T, F>(
    device: &Device,
    config: &StreamConfig,
    on_sample: F,
) -> Result<Stream, Error>
where
    T: Sample,
    F: FnMut(&mut SampleRequestOptions) -> f32 + Send + 'static + Copy,
{
    let sample_rate = config.sample_rate.0 as f32;
    let sample_clock = 0f32;
    let nchannels = config.channels as usize;
    let mut request = SampleRequestOptions {
        sample_rate,
        sample_clock,
        nchannels,
        sample_count: 0,
    };
    let err_fn = |err| eprintln!("Error building output sound stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |output: &mut [T], _: &OutputCallbackInfo| on_window(output, &mut request, on_sample),
        err_fn,
    )?;

    Ok(stream)
}

fn on_window<T, F>(output: &mut [T], request: &mut SampleRequestOptions, mut on_sample: F)
where
    T: Sample,
    F: FnMut(&mut SampleRequestOptions) -> f32 + Send + 'static,
{
    for frame in output.chunks_mut(request.nchannels) {
        let value: T = T::from(&on_sample(request));
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

fn sample_next(o: &mut SampleRequestOptions) -> f32 {
    let achord = Chord::new(
        PitchClass::A,
        ChordQuality::Diminished,
        ChordNumber::Seventh,
    );
    let cchord = Chord::new(PitchClass::C, ChordQuality::Major, ChordNumber::Seventh);
    let echord = Chord::new(PitchClass::E, ChordQuality::Minor, ChordNumber::Seventh);
    let gchord = Chord::new(PitchClass::G, ChordQuality::Major, ChordNumber::Seventh);

    let chord = match o.sample_count % 4 {
        0 => achord,
        1 => cchord,
        2 => echord,
        3 => gchord,
        _ => unreachable!(),
    };

    let notes = chord.notes();

    o.tick();

    notes
        .iter()
        .map(|n| o.tone(note_to_freq(n), Wave::Sine) * 0.1)
        .sum()
}

fn main() -> anyhow::Result<()> {
    let stream = stream_setup_for(sample_next)?;
    stream.play()?;
    sleep(Duration::from_millis(5000));
    Ok(())
}
