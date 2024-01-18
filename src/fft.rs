extern crate rustfft;

use rustfft::{num_complex::Complex, FftPlanner};

pub fn process_stream_data(stream_data: &[f32]) -> Vec<Complex<f32>> {
    // let fft_size = 1024;

    let fft_size = 2048;

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(fft_size);

    let last_chunk = stream_data.chunks_exact(fft_size).last().unwrap();

    let mut signal: Vec<Complex<f32>> = last_chunk
        .iter()
        .map(|&val| Complex {
            re: val,
            im: 0.0f32,
        })
        .collect();

    fft.process(&mut signal);

    let halfway = fft_size / 2;
    let three_quarters = fft_size * 3 / 4;

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

pub fn process() {
    println!("hey there");
}
