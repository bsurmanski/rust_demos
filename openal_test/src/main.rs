use alto::*;
use alto::AltoError;
use alto::Mono;
use std::error::Error;
use std::f32;
use std::sync::Arc;

struct SinWave {
    sample_step: f32,
}

impl SinWave {
    pub fn new(frequency: i32, sampling_rate: i32) -> Self {
        SinWave {
            sample_step: frequency as f32 / sampling_rate as f32 * 2.0 * std::f32::consts::PI,
        }
    }

    pub fn sample(self, i: i32) -> f32 {
        return (i as f32 * self.sample_step).sin();
    }
}

fn main() -> Result<(), Box<Error>> {
    let alto = Alto::load_default()?;

    for s in alto.enumerate_outputs() {
        println!("Found device: {}", s.to_str()?);
    }

    let device = alto.open(None)?; // Opens the default audio device
    let context = device.new_context(None)?; // Creates a default context

    let mut source = context.new_streaming_source()?;

    // Now you can load your samples and store them in a buffer with
    // `context.new_buffer(samples, frequency)`;
    let mut samples: Vec<Mono<f32>> = vec![Mono{center: 0.0}; 44100];
    for i in 0..44100 {
        let mut x = i as f32 * 440.0 / 44100.0 * 2.0 * std::f32::consts::PI;
        x = x.sin();
        if x > 0.0 {
            x = 1.0;
        } else {
            x = -1.0;
        }
        samples[i].center = x * 0.50;
    }

    let buf = context.new_buffer(samples.as_slice(), 44100)?;
    source.queue_buffer(buf)?;
    source.play();

    std::thread::sleep(std::time::Duration::new(2, 0));
    
    Ok(())
}
