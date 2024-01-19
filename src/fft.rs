extern crate rustfft;
use rustfft::{num_complex::Complex, FftPlanner};

pub const FFT_CHUNK_SIZE: usize = 2048;

pub fn process_stream_data(stream_data: &[f32]) -> Vec<Complex<f32>> {
    // let fft_size = 1024;

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

    let halfway = FFT_CHUNK_SIZE / 2;
    // let three_quarters = FFT_CHUNK_SIZE * 3 / 4;

    signal[halfway..].to_vec()

    // lol wtf does this mean
    //
    // // rustfft doesn't normalize when it computes the fft, so we need to normalize ourselves by
    // // dividing by `sqrt(signal.len())` each time we take an fft or inverse fft.
    // // Since the fft is linear and we are doing fft -> inverse fft, we can just divide by
    // // `signal.len()` once.
    // let normalization_const = T::one() / T::from_usize(signal.len()).unwrap();
    // signal
    //     .iter_mut()
    //     .zip(truncated_signal_complex.iter())
    //     .for_each(|(a, b)| {
    //         *a = *a * normalization_const * b.conj();
    //     });
}
