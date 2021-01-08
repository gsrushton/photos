#[derive(Debug)]
pub enum Orientation {
    Identity,
    FlipHorz,
    Rotate180,
    FlipVert,
    Transpose,
    Rotate90Cw,
    Transverse,
    Rotate270Cw,
}

impl Orientation {
    pub fn reorient<'a>(
        &self,
        image: &'a image::DynamicImage,
    ) -> std::borrow::Cow<'a, image::DynamicImage> {
        use std::borrow::Cow;
        match self {
            Self::Identity => Cow::Borrowed(image),
            Self::FlipHorz => Cow::Owned(image.fliph()),
            Self::Rotate180 => Cow::Owned(image.rotate180()),
            Self::FlipVert => Cow::Owned(image.flipv()),
            Self::Transpose => Cow::Owned(image.rotate90().fliph()),
            Self::Rotate90Cw => Cow::Owned(image.rotate90()),
            Self::Transverse => Cow::Owned(image.rotate90().flipv()),
            Self::Rotate270Cw => Cow::Owned(image.rotate270()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct NewMetaDataError(exif::Error);

pub struct MetaData(exif::Exif);

impl MetaData {
    pub fn new<R>(mut r: R) -> Result<Self, NewMetaDataError>
    where
        R: std::io::BufRead + std::io::Seek,
    {
        Ok(Self(
            exif::Reader::new()
                .read_from_container(&mut r)
                .map_err(NewMetaDataError)?,
        ))
    }

    pub fn orientation(&self) -> Option<Orientation> {
        self.0
            .get_field(exif::Tag::Orientation, exif::In::PRIMARY)
            .and_then(|orientation| orientation.value.get_uint(0))
            .and_then(|orientation| match orientation {
                1 => Some(Orientation::Identity),
                2 => Some(Orientation::FlipHorz),
                3 => Some(Orientation::Rotate180),
                4 => Some(Orientation::FlipVert),
                5 => Some(Orientation::Transpose),
                6 => Some(Orientation::Rotate90Cw),
                7 => Some(Orientation::Transverse),
                8 => Some(Orientation::Rotate270Cw),
                _ => None,
            })
    }

    pub fn original_datetime(&self) -> Option<chrono::NaiveDateTime> {
        self.0
            .get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY)
            .and_then(|date_time_original| match &date_time_original.value {
                exif::Value::Ascii(ascii_strings) => ascii_strings.get(0),
                _ => None,
            })
            .and_then(|ascii_string| std::str::from_utf8(ascii_string).ok())
            .and_then(|datetime_string| {
                chrono::NaiveDateTime::parse_from_str(datetime_string, "%Y:%m:%d %H:%M:%S").ok()
            })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NewImageExtError {
    #[error("Failed to read the image")]
    ImageReadError(#[source] std::io::Error),
    #[error("Unsupported image format")]
    UnsupportedImageFormat,
    #[error("Failed to decode image")]
    ImageDecodingError(#[from] image::error::ImageError),
    #[error("Failed to read the image's meta-data")]
    MetaDataReadError(#[from] NewMetaDataError),
}

pub struct ImageExt {
    image: image::DynamicImage,
    format: image::ImageFormat,
    meta_data: Option<MetaData>,
}

impl ImageExt {
    pub fn new<R: std::io::Read>(mut r: R) -> Result<ImageExt, NewImageExtError> {
        let mut bytes = Vec::with_capacity(1024 * 1024);

        r.read_to_end(&mut bytes)
            .map_err(NewImageExtError::ImageReadError)?;

        let image_reader = image::io::Reader::new(std::io::Cursor::new(&bytes))
            .with_guessed_format()
            .unwrap();

        let format = image_reader
            .format()
            .ok_or(NewImageExtError::UnsupportedImageFormat)?;

        let image = image_reader
            .decode()
            .map_err(NewImageExtError::ImageDecodingError)?;

        let meta_data = {
            use image::ImageFormat;
            match format {
                ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::Tiff => {
                    Some(MetaData::new(&mut std::io::Cursor::new(&bytes))?)
                }
                _ => None,
            }
        };

        Ok(Self {
            image,
            format,
            meta_data,
        })
    }

    pub fn format(&self) -> image::ImageFormat {
        self.format
    }

    pub fn orientation(&self) -> Orientation {
        self.meta_data
            .as_ref()
            .and_then(|meta_data| meta_data.orientation())
            .unwrap_or(Orientation::Identity)
    }

    pub fn original_datetime(&self) -> Option<chrono::NaiveDateTime> {
        self.meta_data
            .as_ref()
            .and_then(|meta_data| meta_data.original_datetime())
    }

    pub fn reorient(self) -> image::DynamicImage {
        // TODO fix the unecessary copy
        self.orientation().reorient(&self.image).into_owned()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.image.as_bytes()
    }
}

pub fn thumbnail(image: &image::DynamicImage, size: u32) -> image::DynamicImage {
    use image::GenericImageView;

    let (image_width, image_height) = image.dimensions();
    let ratio = image_width as f32 / image_height as f32;

    let (thumb_width, thumb_height) = if ratio > 2.0 {
        let thumb_width = std::cmp::min(image_width, size * 2);
        (thumb_width, ((thumb_width as f32) / ratio) as u32)
    } else if ratio < 0.5 {
        let thumb_width = std::cmp::max(image_width, size / 2);
        (thumb_width, ((thumb_width as f32) / ratio) as u32)
    } else {
        (((size as f32) * ratio) as u32, size)
    };

    image.resize_exact(
        thumb_width,
        thumb_height,
        image::imageops::FilterType::Lanczos3,
    )
}

pub fn encode_image(
    image: &image::DynamicImage,
) -> Result<actix_web::web::HttpResponse, image::ImageError> {
    let mut cursor = std::io::Cursor::new(Vec::with_capacity(1024 * 1024));

    image.write_to(&mut cursor, image::ImageOutputFormat::Png)?;

    Ok(actix_web::web::HttpResponse::Ok()
        .header(actix_web::http::header::CONTENT_TYPE, "image/png")
        .body(cursor.into_inner()))
}
