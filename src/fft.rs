extern crate rustfft;
use rustfft::{num_complex::Complex, FftPlanner};

pub const FFT_CHUNK_SIZE: usize = 2048;
// pub const FFT_CHUNK_SIZE: usize = 1024;

// Function to generate a Hamming window
fn hamming_window(size: usize) -> Vec<f32> {
    let alpha = 0.54;
    let beta = 1.0 - alpha;
    (0..size)
        .map(|n| {
            alpha - beta * ((2.0 * std::f32::consts::PI * n as f32) / (size as f32 - 1.0)).cos()
        })
        .collect()
}

pub fn process_stream_data(stream_data: &[f32]) -> Vec<Complex<f32>> {
    // let fft_size = 1024;
    let fft_size = stream_data.len();
    let hamming = hamming_window(fft_size);

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(FFT_CHUNK_SIZE);

    let last_chunk = stream_data.chunks_exact(FFT_CHUNK_SIZE).last().unwrap();

    let mut signal: Vec<Complex<f32>> = last_chunk
        .iter()
        .map(|&val| Complex {
            re: val,
            im: 0.0f32,
        })
        .collect();

    fft.process(&mut signal);

    let normalization_factor = (fft_size as f32).sqrt();
    for x in &mut signal {
        *x /= normalization_factor;
    }

    let halfway = FFT_CHUNK_SIZE / 2;

    signal[halfway..].to_vec()
}
