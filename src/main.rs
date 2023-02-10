use std::time::Instant;
use exoquant::{Color, convert_to_indexed, ditherer, optimizer};
use lodepng::RGBA;
use rayon::prelude::*;

fn main() {
    let quality = 70;
    let now = Instant::now();
    let img = lodepng::decode32_file("./testdata/lenna.png").unwrap();

    let num_colors = (quality as f64 / 100.0 * 256.0).floor() as usize;

    let buffer = img.buffer
        .par_iter()
        .map(|c| Color::new(c.r, c.g, c.b, c.a))
        .collect::<Vec<Color>>();

    let width = img.width;
    let height = img.height;

    let (palette, indexed_data) = convert_to_indexed(
        &buffer,
        width as usize,
        num_colors,
        &optimizer::WeightedKMeans,
        &ditherer::FloydSteinberg::checkered(),
    );

    let palette = palette
        .par_iter()
        .map(|x| RGBA::new(x.r, x.g, x.b, x.a))
        .collect::<Vec<_>>();

    let mut state = lodepng::Encoder::new();
    state.set_palette(&palette).expect("TODO: panic message");
    state.encode_file("./testdata/compressed/lenna_tinify.png", &indexed_data, width as usize, height as usize).unwrap();

    println!("Time taken: {}ms", now.elapsed().as_millis());
}
