pub mod toml;

use std::time::Duration;

pub fn read_string(path: &str) -> String {
    use std::io::Read;
    use std::fs::File;

    let mut f = File::open(path).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

pub fn get_duration_millis(duration: &Duration) -> u64 {
    let nanos = duration.subsec_nanos() as u64;
    (1000 * 1000 * 1000 * duration.as_secs() + nanos) / (1000 * 1000)
}

pub fn clamp(input: f32, min: f32, max: f32) -> f32 {
    if input > max {
        max
    } else if input < min {
        min
    } else {
        input
    }
}
