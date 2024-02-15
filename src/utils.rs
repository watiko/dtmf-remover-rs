pub fn split_wave<S: Sized + Copy>(samples: Vec<S>, channels: usize) -> Vec<Vec<S>> {
    let size = samples.len() / channels;
    let mut splitted = vec![Vec::with_capacity(size); channels];
    for i in 0..size {
        for (j, s) in splitted.iter_mut().enumerate() {
            s.push(samples[i * channels + j])
        }
    }
    splitted
}

pub fn join_wave<S: Sized + Copy>(splitted: Vec<Vec<S>>) -> Vec<S> {
    let channels = splitted.len();
    let size = splitted.first().expect("msg").len();
    let mut result = Vec::with_capacity(size * channels);
    for i in 0..size {
        for s in splitted.iter() {
            result.push(s[i]);
        }
    }
    result
}

pub fn fft_freq(n: usize, d: f64) -> Vec<f64> {
    let mut buf = vec![0f64; n];
    let val = 1.0 / (n as f64 * d);
    let middle = (n - 1) / 2 + 1;
    for (i, e) in buf.iter_mut().enumerate() {
        if i < middle {
            *e = i as f64 * val;
        } else {
            let i = (i - middle) as f64;
            *e = val * (i - (n as f64 / 2.0));
        }
    }
    buf
}
