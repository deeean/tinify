use exoquant::{Color, convert_to_indexed, ditherer, optimizer};
use image::GenericImageView;
use imghdr::Type;

#[derive(Debug)]
pub enum Error {
    QualityOutOfRange,
    UnexpectedImageType,
    UnsupportedImageType(Type),
    ImageError(image::ImageError),
    LodepngError(lodepng::Error),
    MozjpegError,
}

pub fn tinify<T: AsRef<[u8]>>(buf: T, quality: f32) -> Result<Vec<u8>, Error> {
    if (quality < 0.0) || (quality > 100.0) {
        return Err(Error::QualityOutOfRange);
    }

    let format = match imghdr::from_bytes(buf.as_ref()) {
        None => return Err(Error::UnexpectedImageType),
        Some(format) => match format {
            Type::Png | Type::Jpeg => match format {
                Type::Png => image::ImageFormat::Png,
                Type::Jpeg => image::ImageFormat::Jpeg,
                _ => unreachable!(),
            },
            _ => return Err(Error::UnsupportedImageType(format)),
        },
    };

    let img = match image::load_from_memory_with_format(buf.as_ref(), format) {
        Ok(img) => img,
        Err(e) => return Err(Error::ImageError(e)),
    };

    let width = img.width() as usize;
    let height = img.height() as usize;

    match format {
        image::ImageFormat::Png => {
            let num_colors = (quality / 100.0 * 256.0).floor() as usize;

            let buffer = img
                .pixels()
                .map(|(_, _, c)| Color::new(c.0[0], c.0[1], c.0[2], c.0[3]))
                .collect::<Vec<Color>>();

            let (palette, indexed_data) = convert_to_indexed(
                &buffer,
                width as usize,
                num_colors,
                &optimizer::WeightedKMeans,
                &ditherer::FloydSteinberg::checkered(),
            );

            let buffer = indexed_data
                .iter()
                .map(|x| {
                    let color = palette[*x as usize];
                    [color.r, color.g, color.b, color.a]
                })
                .flatten()
                .collect::<Vec<_>>();

            match lodepng::encode_memory(&buffer, width, height, lodepng::ColorType::RGBA, 8) {
                Ok(x) => Ok(x),
                Err(e) => Err(Error::LodepngError(e)),
            }
        }
        image::ImageFormat::Jpeg => {
            let mut comp = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);

            comp.set_size(width, height);
            comp.set_mem_dest();
            comp.set_quality(quality);
            comp.start_compress();
            comp.write_scanlines(&img.as_bytes());
            comp.finish_compress();

            match comp.data_to_vec() {
                Ok(x) => Ok(x),
                Err(_) => Err(Error::MozjpegError),
            }
        }
        _ => unreachable!(),
    }
}