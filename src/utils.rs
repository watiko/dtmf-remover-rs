pub fn split_stereo_wave<S: Sized + Copy>(samples: Vec<S>) -> (Vec<S>, Vec<S>) {
    let size = samples.len() / 2;
    let mut l_samples = Vec::with_capacity(size);
    let mut r_samples = Vec::with_capacity(size);
    for i in 0..size {
        l_samples.push(samples[i * 2]);
        r_samples.push(samples[i * 2 + 1]);
    }
    (l_samples, r_samples)
}

pub fn join_stereo_wave<S: Sized + Copy>(l_samples: Vec<S>, r_samples: Vec<S>) -> Vec<S> {
    let mut result = Vec::with_capacity(l_samples.len() * 2);
    for i in 0..l_samples.len() {
        let left = l_samples[i];
        let right = r_samples[i];
        result.push(left);
        result.push(right);
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
