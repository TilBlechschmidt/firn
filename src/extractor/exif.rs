use std::io::{BufRead, Read, Seek};

use exif::{DateTime, Field, In, Tag, Value::*};

use super::{Extractor, MimeInfer};
use crate::{
    db::{Attribute, Entity, Value},
    handle::Handle,
};

pub struct ExifExtractor;

#[derive(Default)]
pub struct ExifData {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub timestamp: Option<DateTime>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub alt: Option<f64>,
    pub camera: Option<(String, String)>,
}

impl ExifExtractor {
    pub fn extract<T: Read + BufRead + Seek>(
        reader: &mut T,
    ) -> Result<ExifData, Box<dyn std::error::Error>> {
        let exifreader = exif::Reader::new();
        let exif = exifreader.read_from_container(reader)?;
        let mut data = ExifData::default();

        if let Some(width) = exif
            .get_field(Tag::PixelXDimension, In::PRIMARY)
            .map(long_to_u32)
            .flatten()
        {
            data.width = Some(width);
        }

        if let Some(height) = exif
            .get_field(Tag::PixelYDimension, In::PRIMARY)
            .map(long_to_u32)
            .flatten()
        {
            data.height = Some(height)
        }

        if let Some(timestamp) = exif
            .get_field(Tag::DateTimeOriginal, In::PRIMARY)
            .map(ascii_to_str)
            .flatten()
            .map(|string| DateTime::from_ascii(string.as_bytes()).ok())
            .flatten()
        {
            data.timestamp = Some(timestamp);
        }

        // TODO Take GPS*Ref into account as the coordinates may be S/W!
        if let (Some(lat), Some(lng)) = (
            exif.get_field(Tag::GPSLatitude, In::PRIMARY)
                .map(coord_to_decimal_degree)
                .flatten(),
            exif.get_field(Tag::GPSLongitude, In::PRIMARY)
                .map(coord_to_decimal_degree)
                .flatten(),
        ) {
            data.lat = Some(lat);
            data.lng = Some(lng);
        }

        if let Some(alt) = exif
            .get_field(Tag::GPSAltitude, In::PRIMARY)
            .map(rational_to_f64)
            .flatten()
        {
            data.alt = Some(alt);
        }

        if let (Some(make), Some(model)) = (
            exif.get_field(Tag::Make, In::PRIMARY)
                .map(ascii_to_str)
                .flatten(),
            exif.get_field(Tag::Model, In::PRIMARY)
                .map(ascii_to_str)
                .flatten(),
        ) {
            data.camera = Some((make.to_owned(), model.to_owned()));
        }

        Ok(data)
    }

    fn extract_exif(
        &self,
        handle: &mut Handle,
        entity: &Entity,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut blob = handle.blob(entity)?;
        let data = Self::extract(&mut blob)?;

        // for f in exif.fields() {
        //     println!(
        //         "{} {} {}",
        //         f.tag,
        //         f.ifd_num,
        //         f.display_value().with_unit(&exif)
        //     );
        // }

        if let Some(width) = data.width {
            handle.insert(entity.clone(), "image/width", width);
        }

        if let Some(height) = data.height {
            handle.insert(entity.clone(), "image/height", height);
        }

        if let Some(timestamp) = data.timestamp {
            // TODO Use Tag::SubSecTimeOriginal for millis
            // TODO Take Tag::OffsetOriginal into account for the time zone, yuck
            // TODO Convert to RFC3339 or whatever it was
            let formatted = format!(
                "{:0>4}-{:0>2}-{:0>2}T{:0>2}:{:0>2}:{:0>2}.00Z",
                timestamp.year,
                timestamp.month,
                timestamp.day,
                timestamp.hour,
                timestamp.minute,
                timestamp.second
            );
            handle.insert(entity.clone(), "time/creation", formatted);
        }

        // TODO Take GPS*Ref into account as the coordinates may be S/W!
        if let (Some(lat), Some(lng)) = (data.lat, data.lng) {
            handle.insert(entity.clone(), "location/latitude", lat);
            handle.insert(entity.clone(), "location/longitude", lng);
        }

        if let Some(alt) = data.alt {
            handle.insert(entity.clone(), "location/altitude", alt);
        }

        if let Some((make, model)) = data.camera {
            let camera = Entity::from(model);

            if handle
                .get(&camera, &"device/manufacturer".into())
                .next()
                .is_none()
            {
                handle.insert(
                    camera.clone(),
                    "device/manufacturer",
                    uppercase_first_letter(&make),
                );
            }

            handle.insert(entity.clone(), "image/camera", Value::Reference(camera));
        }

        Ok(())
    }
}

fn ascii_to_str(field: &Field) -> Option<&str> {
    if let Ascii(t) = &field.value {
        if t.len() == 1 {
            return std::str::from_utf8(&t[0]).ok();
        }
    }

    None
}

fn long_to_u32(field: &Field) -> Option<u32> {
    if let Long(l) = &field.value {
        if l.len() == 1 {
            return Some(l[0]);
        }
    }

    None
}

fn rational_to_f64(field: &Field) -> Option<f64> {
    if let Rational(r) = &field.value {
        if r.len() == 1 {
            return Some(r[0].to_f64());
        }
    }

    None
}

fn coord_to_decimal_degree(field: &Field) -> Option<f64> {
    if let Rational(r) = &field.value {
        if r.len() == 3 {
            return Some(r[0].to_f64() + r[1].to_f64() / 60.0 + r[2].to_f64() / 3600.0);
        }
    }

    None
}

fn uppercase_first_letter(s: &str) -> String {
    let lowercased = s.to_lowercase();
    let mut c = lowercased.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

impl Extractor for ExifExtractor {
    fn entry_added(
        &mut self,
        handle: &mut Handle,
        entity: &Entity,
        attribute: &Attribute,
        value: &Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if attribute == &MimeInfer::attribute() {
            match value {
                Value::Data(mime) if mime.starts_with("image") => {
                    self.extract_exif(handle, entity)?;
                }
                _ => {}
            }
        }

        Ok(())
    }
}
