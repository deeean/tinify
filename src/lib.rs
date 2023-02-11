use exoquant::{Color, convert_to_indexed, ditherer, optimizer};
use image::GenericImageView;
use imghdr::Type;
use lodepng::RGBA;

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
            Type::Png | Type::Jpeg => format,
            _ => return Err(Error::UnsupportedImageType(format)),
        },
    };

    let img = match format {
        Type::Png => image::load_from_memory_with_format(buf.as_ref(), image::ImageFormat::Png),
        Type::Jpeg => image::load_from_memory_with_format(buf.as_ref(), image::ImageFormat::Jpeg),
        _ => unreachable!(),
    };

    let img = match img {
        Ok(img) => img,
        Err(e) => return Err(Error::ImageError(e)),
    };

    let width = img.width() as usize;
    let height = img.height() as usize;

    match format {
        Type::Png => {
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
        },
        Type::Jpeg => {
            let mut comp = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);

            println!("{}", quality);

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
        _ => return Err(Error::UnsupportedImageType(format)),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use super::*;

    #[test]
    fn test_tinify() {
        let entries = std::fs::read_dir("./testdata")
            .unwrap()
            .map(|x| x.unwrap().path())
            .filter(|x| {
                let ext = match x.extension() {
                    Some(x) => match x.to_str() {
                        Some(x) => x,
                        None => return false,
                    },
                    None => return false,
                };

                ext == "png" || ext == "jpg"
            })
            .collect::<Vec<_>>();

        entries
            .iter()
            .map(|x| {
                println!("Compressing {}...", x.to_str().unwrap());

                let buf = std::fs::read(x).unwrap();
                let file_name = x.file_stem().unwrap().to_str().unwrap();
                let ext = x.extension().unwrap().to_str().unwrap();

                match tinify(&buf, 70.0) {
                    Ok(buf) => {
                        let mut file = std::fs::File::create(format!("./dist/{}_tinify.{}", file_name, ext)).unwrap();
                        file.write_all(&buf).unwrap();
                    }
                    Err(e) => panic!("{:?}", e),
                }
            })
            .collect::<Vec<_>>();
    }
}