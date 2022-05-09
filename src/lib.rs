use crate::DynamicImage::ImageRgba8;
use base64::DecodeError;
use base64::{decode, encode};
use image::DynamicImage;
use image::GenericImageView;
use image::ImageBuffer;
use image::Rgba;
use lcs_diff::DiffResult;
use lcs_diff::DiffResult::{Added, Common, Removed};
use std::cmp;

pub static BLACK: (u8, u8, u8) = (0, 0, 0);
pub static RED: (u8, u8, u8) = (255, 119, 119);
pub static GREEN: (u8, u8, u8) = (99, 195, 99);
static RATE: f32 = 0.25;

fn blend(base: Rgba<u8>, rgb: (u8, u8, u8), rate: f32) -> Rgba<u8> {
    return Rgba([
        (base[0] as f32 * (1.0 - rate) + rgb.0 as f32 * (rate)) as u8,
        (base[1] as f32 * (1.0 - rate) + rgb.1 as f32 * (rate)) as u8,
        (base[2] as f32 * (1.0 - rate) + rgb.2 as f32 * (rate)) as u8,
        base[3],
    ]);
}

fn put_diff_pixels(
    y: usize,
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    row_width: u32,
    data: &String,
    rgb: (u8, u8, u8),
    rate: f32,
) -> Result<(), base64::DecodeError> {
    let row = decode(data)?;
    for x in 0..img.dimensions().0 {
        let index = x as usize * 4;
        let pixel: Rgba<u8> = if row_width > x {
            Rgba([row[index], row[index + 1], row[index + 2], row[index + 3]])
        } else {
            Rgba([0, 0, 0, 0])
        };
        img.put_pixel(x as u32, y as u32, blend(pixel, rgb, rate));
    }
    Ok(())
}

pub fn diff(
    before_png: &DynamicImage,
    after_png: &DynamicImage,
) -> Result<DynamicImage, DecodeError> {
    let after_w = after_png.dimensions().0;
    let before_w = before_png.dimensions().0;
    let before_encoded_png: Vec<String> = before_png
        .as_bytes()
        .to_vec()
        .chunks(before_w as usize * 4)
        .map(|chunk| encode(chunk))
        .collect();
    let after_encoded_png: Vec<String> = after_png
        .as_bytes()
        .to_vec()
        .chunks(after_w as usize * 4)
        .map(|chunk| encode(chunk))
        .collect();

    let diff_result: Vec<DiffResult<String>> =
        lcs_diff::diff(&before_encoded_png, &after_encoded_png);

    let height = diff_result.len() as u32;
    let width = cmp::max(before_w, after_w) as u32;
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    for (y, d) in diff_result.iter().enumerate() {
        match d {
            &Added(ref a) => put_diff_pixels(y, &mut img, after_w as u32, &a.data, GREEN, RATE)?,
            &Removed(ref r) => put_diff_pixels(y, &mut img, before_w as u32, &r.data, RED, RATE)?,
            &Common(ref c) => put_diff_pixels(y, &mut img, width, &c.data, BLACK, 0.0)?,
        }
    }
    Ok(ImageRgba8(img))
}
