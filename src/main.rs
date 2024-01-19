mod buffer;
mod fft;

use buffer::BufferWrapper;
use core::borrow::{Borrow, BorrowMut};
use cpal::{self, FromSample, SampleRate};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    StreamConfig,
};
use dasp::Sample;
use dasp_signal::Signal;
use fft::FFT_CHUNK_SIZE;
use minifb::{Key, Window, WindowOptions};
use plotters::{chart::ChartState, coord::types::RangedCoordf32, prelude::*};
use plotters_backend::{text_anchor::HPos, text_anchor::Pos, text_anchor::VPos, BackendColor};
use plotters_bitmap::bitmap_pixel::BGRXPixel;
use rustfft::num_complex::{Complex32, ComplexFloat};
use std::{collections::VecDeque, sync::mpsc, vec};

// const SAMPLE_CHUNK_SIZE: usize = 512; // We don't know sample chunk size

// window constants
const W: usize = 1000;
const H: usize = 800;

type Result<T> = anyhow::Result<T>;

fn _get_drawing_area(
    chart_buffer: &mut [u8],
) -> Result<DrawingArea<BitMapBackend<BGRXPixel>, plotters::coord::Shift>> {
    Ok(
        BitMapBackend::<BGRXPixel>::with_buffer_and_format(chart_buffer, (W as u32, H as u32))?
            .into_drawing_area(),
    )
}

fn initialize_chart_state(
    chart_buffer: &mut [u8],
) -> Result<ChartState<Cartesian2d<RangedCoordf32, RangedCoordf32>>> {
    let root = _get_drawing_area(chart_buffer)?;
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("SoundWave", ("Arial", 20).into_font())
        // .margin(5u32)
        .x_label_area_size(40u32)
        .y_label_area_size(40u32)
        .build_cartesian_2d(0f32..2f32, -1f32..1f32)?;

    chart
        .configure_mesh()
        // .x_label_style(&RED)
        // .y_label_style(&RED)
        .draw()?;

    Ok(chart.into_chart_state())
}

fn get_fft_frame(
    initial_chart: &ChartState<Cartesian2d<RangedCoordf32, RangedCoordf32>>,
    fft_data: &[Complex32],
    frame_count: u32,
    sample_rate: usize,
) -> Result<Vec<u32>> {
    let mut chart_buffer = BufferWrapper(vec![0u32; W * H]);

    let drawing_area = _get_drawing_area(chart_buffer.borrow_mut()).unwrap();
    let mut chart = initial_chart.clone().restore(&drawing_area);

    chart.plotting_area().fill(&WHITE)?;

    chart
        .configure_mesh()
        .bold_line_style(&GREEN.mix(0.2))
        .light_line_style(&TRANSPARENT)
        .draw()?;

    // println!("{:?}", fft_data.first());

    chart.draw_series(
        fft_data
            .iter()
            .zip(fft_data.iter().skip(1))
            .enumerate()
            .map(|(n, (point_0, point_1))| {
                // How much of the frequency domain we want to display
                // Higher number shows fewer frequency bins
                let chart_width = 4.0;

                let x_val: f32 = chart_width * n as f32 / fft_data.len() as f32;
                let x_step: f32 = chart_width / fft_data.len() as f32;

                PathElement::new(
                    vec![(x_val, point_0.re()), (x_val + x_step, point_1.re())],
                    &BLUE,
                )
            }),
    )?;

    let style = TextStyle {
        font: ("sans-serif", 15.0).into_font(),
        color: BackendColor {
            alpha: 1.0,
            rgb: (0, 0, 1),
        },
        pos: Pos::new(HPos::Left, VPos::Top),
    };

    // calculate biggest frequency bin
    let max_bin_index = fft_data
        .iter()
        .enumerate()
        .max_by_key(|&(_, freq)| freq.re() as u32)
        .unwrap()
        .0;

    let freq_bin = sample_rate / FFT_CHUNK_SIZE; // TODO: add fft size as constant somehwere

    println!("FreqBin: {:?} | Max Bin Idx: {:?}", freq_bin, max_bin_index);

    let frequency = freq_bin * max_bin_index;

    drawing_area.draw_text(format!("{frame_count}").borrow(), &style, (50, 50))?;

    drawing_area.draw_text(
        format!("Max freq: {frequency} Hz").borrow(),
        &style,
        (50, 60),
    )?;

    drawing_area.draw_text(
        format!("Sample Rate: {sample_rate} Hz").borrow(),
        &style,
        (50, 70),
    )?;

    drop(chart);
    drop(drawing_area);

    Ok(chart_buffer.0)
}

fn _get_chart_frame(
    initial_chart: &ChartState<Cartesian2d<RangedCoordf32, RangedCoordf32>>,
    stream_data: &[f32],
    sample_window: f32,
    sample_rate: f32,
    frame_count: u32,
) -> Result<Vec<u32>> {
    let mut chart_buffer = BufferWrapper(vec![0u32; W * H]);

    let drawing_area = _get_drawing_area(chart_buffer.borrow_mut()).unwrap();
    let mut chart = initial_chart.clone().restore(&drawing_area);

    chart.plotting_area().fill(&WHITE)?;

    chart
        .configure_mesh()
        .bold_line_style(&GREEN.mix(0.2))
        .light_line_style(&TRANSPARENT)
        .draw()?;

    chart.draw_series(
        stream_data
            .iter()
            .zip(stream_data.iter().skip(1))
            .enumerate()
            .map(|(n, (&y0, &y1))| {
                let x_val: f32 = n as f32 / stream_data.len() as f32;
                let x_step: f32 = 1.0 / stream_data.len() as f32;

                PathElement::new(
                    vec![(x_val, y0), (x_val + x_step, y1)],
                    &BLUE.mix(x_val.into()),
                )
            }),
    )?;

    let style = TextStyle {
        font: ("sans-serif", 15.0).into_font(),
        color: BackendColor {
            alpha: 1.0,
            rgb: (0, 0, 1),
        },
        pos: Pos::new(HPos::Left, VPos::Top),
    };

    drawing_area.draw_text(frame_count.to_string().borrow(), &style, (50, 50))?;

    drop(chart);
    drop(drawing_area);

    Ok(chart_buffer.0)
}

fn main() -> Result<()> {
    let host = cpal::default_host();
    let input_device = host
        .default_input_device()
        .expect("Failed to find an input device");
    let output_device = host
        .default_output_device()
        .expect("failed to find a default output device");

    println!("Using input device: \"{}\"", input_device.name()?);
    println!("Using output device: \"{}\"", output_device.name()?);

    let output_config = output_device.default_output_config()?;
    println!("Output Sample Rate: {:?}", output_config.sample_rate().0);

    let default_inp_config = input_device.default_input_config()?;
    let input_config = StreamConfig {
        channels: default_inp_config.channels(),
        sample_rate: SampleRate(44100),
        buffer_size: cpal::BufferSize::Default,
    };
    println!("Input Sample Rate: {:?}", 44100);

    // make buffer
    let sample_buffer_size = input_config.sample_rate.0 as usize * 2; // two seconds of samples
    let mut sample_buffer = VecDeque::<f32>::with_capacity(sample_buffer_size);
    // println!("{:?}", sample_buffer_size);

    let (tx, rx) = mpsc::channel();

    let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        tx.send(data.to_vec()).unwrap();
    };
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    // Create a delay in case the input and output devices aren't synced.
    // let latency_frames = (150. / 1_000.0) * output_config.sample_rate().0 as f32;
    // let latency_samples = latency_frames as usize * output_config.channels() as usize;
    // let sample_format = config.sample_format();
    // let output_config = output_config.clone();
    let input_config = input_config.clone();

    let input_stream =
        input_device.build_input_stream(&input_config, input_data_fn, err_fn, None)?;
    println!("Successfully built streams.");

    // Play the streams.
    println!("Starting the input stream.",);
    input_stream.play()?;

    println!("Playing... ");

    // initialize window
    let mut window = Window::new("Window", W, H, WindowOptions::default())?;

    let buttery_smooth = Some(std::time::Duration::from_secs(1) / 60); //60 fps;
    window.limit_update_rate(buttery_smooth);

    let mut frame_count = 0;

    // initialize chart state
    let initial_chart =
        initialize_chart_state(BufferWrapper(vec![0u32; W * H]).borrow_mut()).unwrap();

    let output_handle = std::thread::spawn(move || {
        run_output::<f32>(&output_device, &output_config.into()).unwrap();
    });
    // let sample_window = 1. / input_config.sample_rate.0 as f32;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let frame_start = std::time::Instant::now();

        // while within frame budget, and channel not empty, pull more data
        while let (true, Ok(sample)) = (
            std::time::Instant::now() - frame_start <= buttery_smooth.unwrap(),
            rx.try_recv(),
        ) {
            if sample_buffer.len() >= sample_buffer_size {
                sample_buffer.drain(..sample.len());
            }
            sample_buffer.extend(sample.iter());
        }

        let stream_data = sample_buffer.make_contiguous();

        // let frame = get_chart_frame(
        //     &initial_chart,
        //     &stream_data,
        //     sample_window,
        //     input_config.sample_rate.0 as f32,
        //     frame_count,
        // )?;
        // window.update_with_buffer(frame.as_slice(), W, H).unwrap();

        if stream_data.len() >= 4096 {
            let fft_data = fft::process_stream_data(stream_data);

            let fft_frame = get_fft_frame(
                &initial_chart,
                &fft_data,
                frame_count,
                input_config.sample_rate.0 as usize,
            )?;

            window
                .update_with_buffer(fft_frame.as_slice(), W, H)
                .unwrap();
        }

        frame_count += 1;
    }

    output_handle.join().unwrap();

    drop(input_stream);
    println!("Done!");

    Ok(())
}

// fn run_input(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), anyhow::Error> {
//     let (tx, rx) = mpsc::sync_channel(1);
//     // Create and run the stream.
//     let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
//     let channels = config.channels as usize;
//     let stream = device.build_input_stream(
//         config,
//         move |data: &[f32], _: &cpal::InputCallbackInfo| {
//             if data.len() == 0 {
//                 tx.try_send(()).ok();
//             } else {
//                 for &sample in data {
//                     println!("{:?}", sample);
//                 }
//             }
//         },
//         err_fn,
//         None,
//     )?;
//     stream.play()?;
//     rx.recv().unwrap();

//     Ok(())
// }

fn run_output<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<()>
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    // Create a signal chain to play back 1 second of each oscillator at A4.
    let hz = dasp_signal::rate(config.sample_rate.0 as f64).const_hz(18000.0);
    let one_sec = config.sample_rate.0 as usize;
    let mut synth = hz
        .clone()
        .sine()
        .take(one_sec)
        .cycle()
        .map(|s| s.to_sample::<f32>() * 0.2);

    let (tx, rx) = mpsc::sync_channel(1);

    // Create and run the stream.
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let channels = config.channels as usize;
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &tx, &mut synth)
        },
        err_fn,
        None,
    )?;
    stream.play()?;
    rx.recv().unwrap();

    Ok(())
}

fn write_data<T>(
    output: &mut [T],
    channels: usize,
    tx: &mpsc::SyncSender<()>,
    signal: &mut dyn Iterator<Item = f32>,
) where
    T: cpal::Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let sample = match signal.next() {
            None => {
                tx.try_send(()).ok();
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
