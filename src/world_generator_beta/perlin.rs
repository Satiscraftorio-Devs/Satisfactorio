use noise::{NoiseFn, Perlin};

#[inline]
pub fn generate_height(seed: u32, x: f64, z: f64, scale: f64, octaves: u32) -> f32 {
    let perlin = Perlin::new(seed);
    
    let frequency = 16.0 * scale;
    let amplitude = 20.0 / (octaves as f64);
    
    let mut height = 0.0;
    for i in 0..octaves {
        let freq = frequency * (i as f64 + 1.0).powf(2.0);
        let amp = amplitude / (i as f64 + 1.0).powf(2.0);
        
        height += perlin.get([x * freq, z * freq]) * amp;
    }
    height as f32
}