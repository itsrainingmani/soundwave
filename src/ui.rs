use core::borrow::{Borrow, BorrowMut};
use plotters::{chart::ChartState, coord::types::RangedCoordf32, prelude::*};
use plotters_backend::{text_anchor::HPos, text_anchor::Pos, text_anchor::VPos, BackendColor};
use plotters_bitmap::bitmap_pixel::BGRXPixel;
use rustfft::num_complex::{Complex32, ComplexFloat};
use std::vec;

use crate::{buffer::BufferWrapper, fft::FFT_CHUNK_SIZE};
// const SAMPLE_CHUNK_SIZE: usize = 512; // We don't know sample chunk size
// window constants
pub const W: usize = 1000;
pub const H: usize = 800;

type Result<T> = anyhow::Result<T>;
fn _get_drawing_area(
    chart_buffer: &mut [u8],
) -> Result<DrawingArea<BitMapBackend<BGRXPixel>, plotters::coord::Shift>> {
    Ok(
        BitMapBackend::<BGRXPixel>::with_buffer_and_format(chart_buffer, (W as u32, H as u32))?
            .into_drawing_area(),
    )
}

pub fn initialize_chart_state(
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

pub fn get_fft_frame(
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
