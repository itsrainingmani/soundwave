use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{self, FromSample};
use dasp::Sample;
use dasp_signal::Signal;
use gnuplot::{Caption, Color, Figure};
use ringbuf::HeapRb;
use std::sync::mpsc;

fn main() -> Result<(), anyhow::Error> {
    let host = cpal::default_host();
    let input_device = host
        .default_input_device()
        .expect("Failed to find an input device");
    let output_device = host
        .default_output_device()
        .expect("failed to find a default output device");
    println!("Using input device: \"{}\"", input_device.name()?);
    println!("Using output device: \"{}\"", output_device.name()?);
    let config = output_device.default_output_config()?;
    println!("{:?}", config.sample_rate().0);

    // Create a delay in case the input and output devices aren't synced.
    let latency_frames = (150. / 1_000.0) * config.sample_rate().0 as f32;
    let latency_samples = latency_frames as usize * config.channels() as usize;

    // The buffer to share samples
    let ring = HeapRb::<f32>::new(latency_samples * 2);
    let (mut producer, mut consumer) = ring.split();

    // let handler = std::thread::spawn(|| {
    //     let x = [0u32, 1, 2];
    //     let y = [3u32, 4, 5];
    //     let mut fg = Figure::new();
    //     fg.axes2d().lines(&x, &y, &[Caption("Line"), Color("red")]);
    //     fg.show().unwrap();
    // });

    // let x = [0u32, 1, 2];
    // let y = [3u32, 4, 5];
    // let mut fg = Figure::new();
    // fg.axes2d().lines(&x, &y, &[Caption("Line"), Color("red")]);
    // fg.show().unwrap();

    loop {
        let config = config.clone();
        match config.sample_format() {
            cpal::SampleFormat::F32 => run::<f32>(&output_device, &config.into())?,
            cpal::SampleFormat::I16 => run::<i16>(&output_device, &config.into())?,
            cpal::SampleFormat::U16 => run::<u16>(&output_device, &config.into())?,
            _ => {}
        }
    }

    // handler.join().unwrap();

    Ok(())
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), anyhow::Error>
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    // Create a signal chain to play back 1 second of each oscillator at A4.
    let hz = dasp_signal::rate(config.sample_rate.0 as f64).const_hz(440.0);
    let one_sec = config.sample_rate.0 as usize;
    let mut synth = hz
        .clone()
        .sine()
        .take(one_sec)
        .cycle()
        // .chain(hz.clone().saw().take(one_sec))
        // .chain(hz.clone().square().take(one_sec))
        // .chain(hz.clone().noise_simplex().take(one_sec))
        // .chain(dasp_signal::noise(0).take(one_sec))
        .map(|s| s.to_sample::<f32>() * 0.2);

    // A channel for indicating when playback has completed.
    let (complete_tx, complete_rx) = mpsc::sync_channel(1);

    // Create and run the stream.
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let channels = config.channels as usize;
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &complete_tx, &mut synth)
        },
        err_fn,
        None,
    )?;
    stream.play()?;

    // Wait for playback to complete.
    complete_rx.recv().unwrap();
    stream.pause()?;

    Ok(())
}

fn write_data<T>(
    output: &mut [T],
    channels: usize,
    complete_tx: &mpsc::SyncSender<()>,
    signal: &mut dyn Iterator<Item = f32>,
) where
    T: cpal::Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let sample = match signal.next() {
            None => {
                complete_tx.try_send(()).ok();
                0.0
            }
            Some(sample) => sample,
        };
        let value: T = T::from_sample::<f32>(sample);
        // let value: T = cpal::Sample::from::<f32>(&sample);
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

/*
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::HeapRb;
*/

// fn main() -> anyhow::Result<()> {
//     let host = cpal::default_host();

//     // Find devices.
//     let input_device = host.default_input_device().unwrap();

//     let output_device = host.default_output_device().unwrap();

//     println!("Using input device: \"{}\"", input_device.name()?);
//     println!("Using output device: \"{}\"", output_device.name()?);

//     // We'll try and use the same configuration between streams to keep it simple.
//     let config: cpal::StreamConfig = input_device.default_input_config()?.into();

//     // Create a delay in case the input and output devices aren't synced.
//     let latency_frames = (150. / 1_000.0) * config.sample_rate.0 as f32;
//     let latency_samples = latency_frames as usize * config.channels as usize;

//     // The buffer to share samples
//     let ring = HeapRb::<f32>::new(latency_samples * 2);
//     let (mut producer, mut consumer) = ring.split();

//     // Fill the samples with 0.0 equal to the length of the delay.
//     for _ in 0..latency_samples {
//         // The ring buffer has twice as much space as necessary to add latency here,
//         // so this should never fail
//         producer.push(0.0).unwrap();
//     }

//     let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
//         let mut output_fell_behind = false;
//         for &sample in data {
//             if producer.push(sample).is_err() {
//                 output_fell_behind = true;
//             }
//         }
//         if output_fell_behind {
//             eprintln!("output stream fell behind: try increasing latency");
//         }
//     };

//     let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
//         let mut input_fell_behind = false;
//         for sample in data {
//             *sample = match consumer.pop() {
//                 Some(s) => s,
//                 None => {
//                     input_fell_behind = true;
//                     0.0
//                 }
//             };
//         }
//         if input_fell_behind {
//             eprintln!("input stream fell behind: try increasing latency");
//         }
//     };

//     // Build streams.
//     println!(
//         "Attempting to build both streams with f32 samples and `{:?}`.",
//         config
//     );
//     let input_stream = input_device.build_input_stream(&config, input_data_fn, err_fn, None)?;
//     let output_stream = output_device.build_output_stream(&config, output_data_fn, err_fn, None)?;
//     println!("Successfully built streams.");

//     // Play the streams.
//     println!(
//         "Starting the input and output streams with `{}` milliseconds of latency.",
//         150.
//     );
//     input_stream.play()?;
//     output_stream.play()?;

//     // Run for 3 seconds before closing.
//     println!("Playing for 3 seconds... ");
//     std::thread::sleep(std::time::Duration::from_secs(3));
//     drop(input_stream);
//     drop(output_stream);
//     println!("Done!");
//     Ok(())
// }

// fn err_fn(err: cpal::StreamError) {
//     eprintln!("an error occurred on stream: {}", err);
// }
