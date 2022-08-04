use base64::{decode, encode, DecodeError};
use image::io::Reader;
use image::DynamicImage;
use image::DynamicImage::ImageRgba8;
use image::GenericImageView;
use image::ImageBuffer;
use image::Rgba;
use std::io::Cursor;
use std::{cmp, vec};

pub static BLACK: (u8, u8, u8) = (0, 0, 0);
pub static RED: (u8, u8, u8) = (255, 119, 119);
pub static GREEN: (u8, u8, u8) = (99, 195, 99);
static RATE: f32 = 0.25;

#[derive(Debug, PartialEq)]
enum DiffResult<'a, T: PartialEq> {
    Removed(DiffElement<'a, T>),
    Common(DiffElement<'a, T>),
    Added(DiffElement<'a, T>),
}

#[derive(Debug, PartialEq)]
struct DiffElement<'a, T: PartialEq> {
    pub data: &'a T,
}

// Table is like:
// \ o l d
// n
// e
// w
pub fn create_table<T: PartialEq>(old: &[T], new: &[T]) -> Vec<Vec<u32>> {
    let new_len = new.len();
    let old_len = old.len();
    let mut table = vec![vec![0; old_len + 1]; new_len + 1];
    for i in 0..new_len {
        let i = new_len - i - 1;
        for j in 0..old_len {
            let j = old_len - j - 1;
            // Performance bottle neck - long string comparison
            table[i][j] = if new[i] == old[j] {
                table[i + 1][j + 1] + 1
            } else {
                cmp::max(table[i + 1][j], table[i][j + 1])
            }
        }
    }
    table
}

fn lcs_diff<'a, T: PartialEq>(old: &'a [T], new: &'a [T]) -> Vec<DiffResult<'a, T>> {
    let new_len = new.len();
    let old_len = old.len();

    if new_len == 0 {
        let mut result = Vec::with_capacity(old_len);
        let mut o = 0;
        while o < old_len {
            result.push(DiffResult::Removed(DiffElement { data: &old[o] }));
            o += 1;
        }
        return result;
    } else if old_len == 0 {
        let mut result = Vec::with_capacity(new_len);
        let mut n = 0;
        while n < new_len {
            result.push(DiffResult::Added(DiffElement { data: &new[n] }));
            n += 1;
        }
        return result;
    } else {
        let mut o = 0;
        let mut n = 0;
        let common_prefix = old.iter().zip(new).take_while(|p| p.0 == p.1);
        let prefix_size = common_prefix.count();
        let common_suffix = old
            .iter()
            .rev()
            .zip(new.iter().rev())
            .take(cmp::min(old_len, new_len) - prefix_size)
            .take_while(|p| p.0 == p.1);
        let suffix_size = common_suffix.count();
        let table = create_table(
            &old[prefix_size..(old_len - suffix_size)],
            &new[prefix_size..(new_len - suffix_size)],
        );
        let new_len = new_len - prefix_size - suffix_size;
        let old_len = old_len - prefix_size - suffix_size;
        let mut result = Vec::with_capacity(prefix_size + cmp::max(old_len, new_len) + suffix_size);

        // Restore common prefix
        let mut prefix_index = 0;
        while prefix_index < prefix_size {
            result.push(DiffResult::Common(DiffElement {
                data: &old[prefix_index],
            }));
            prefix_index += 1;
        }

        loop {
            if n >= new_len || o >= old_len {
                break;
            }
            let new_index = n + prefix_size;
            let old_index = o + prefix_size;
            if new[new_index] == old[old_index] {
                result.push(DiffResult::Common(DiffElement {
                    data: &new[new_index],
                }));
                n += 1;
                o += 1;
            } else if table[n + 1][o] >= table[n][o + 1] {
                result.push(DiffResult::Added(DiffElement {
                    data: &new[new_index],
                }));
                n += 1;
            } else {
                result.push(DiffResult::Removed(DiffElement {
                    data: &old[old_index],
                }));
                o += 1;
            }
        }
        while n < new_len {
            let new_index = n + prefix_size;
            result.push(DiffResult::Added(DiffElement {
                data: &new[new_index],
            }));
            n += 1;
        }
        while o < old_len {
            let old_index = o + prefix_size;
            result.push(DiffResult::Removed(DiffElement {
                data: &old[old_index],
            }));
            o += 1;
        }

        // Restore common suffix
        let mut suffix_index = 0;
        while suffix_index < suffix_size {
            let old_index = suffix_index + old_len + prefix_size;
            result.push(DiffResult::Common(DiffElement {
                data: &old[old_index],
            }));
            suffix_index += 1;
        }
        result
    }
}

fn blend(base: Rgba<u8>, rgb: (u8, u8, u8), rate: f32) -> Rgba<u8> {
    Rgba([
        (base[0] as f32 * (1.0 - rate) + rgb.0 as f32 * (rate)) as u8,
        (base[1] as f32 * (1.0 - rate) + rgb.1 as f32 * (rate)) as u8,
        (base[2] as f32 * (1.0 - rate) + rgb.2 as f32 * (rate)) as u8,
        base[3],
    ])
}

fn put_diff_pixels(
    y: usize,
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    row_width: u32,
    data: &str,
    rgb: (u8, u8, u8),
    rate: f32,
) -> Result<(), base64::DecodeError> {
    let row = decode(data)?;
    for x in 0..img.dimensions().0 {
        let index = x as usize * 4;
        let pixel = if row_width > x {
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
        .map(encode)
        .collect();
    let after_encoded_png: Vec<String> = after_png
        .as_bytes()
        .to_vec()
        .chunks(after_w as usize * 4)
        .map(encode)
        .collect();

    let diff_result = lcs_diff(&before_encoded_png, &after_encoded_png);

    let height = diff_result.len() as u32;
    let width = cmp::max(before_w, after_w) as u32;
    let mut img = ImageBuffer::new(width, height);
    for (y, d) in diff_result.iter().enumerate() {
        match d {
            DiffResult::Added(ref a) => {
                put_diff_pixels(y, &mut img, after_w as u32, a.data, GREEN, RATE)?
            }
            DiffResult::Removed(ref r) => {
                put_diff_pixels(y, &mut img, before_w as u32, r.data, RED, RATE)?
            }
            DiffResult::Common(ref c) => put_diff_pixels(y, &mut img, width, c.data, BLACK, 0.0)?,
        }
    }
    Ok(ImageRgba8(img))
}

pub fn diff_slice(before_slice: &[u8], after_slice: &[u8]) -> Result<Vec<u8>, DecodeError> {
    let before_png = Reader::new(Cursor::new(before_slice))
        .with_guessed_format()
        .expect("Cursor io never fails")
        .decode()
        .expect("Unable to decode before_png");
    let after_png = Reader::new(Cursor::new(after_slice))
        .with_guessed_format()
        .expect("Cursor io never fails")
        .decode()
        .expect("Unable to decode after_png");
    diff(&before_png, &after_png).map(|img| img.as_bytes().to_vec())
}

#[allow(dead_code)]
fn gen_lcs<'a, T: PartialEq>(table: &Vec<Vec<u32>>, old: &[T], new: &'a [T]) -> Vec<&'a T> {
    let o_len = old.len();
    let n_len = new.len();
    let mut o = 0;
    let mut n = 0;
    let mut res = vec![];
    while o < o_len && n < n_len {
        if old[o] == new[n] {
            res.push(&new[n]);
            o = o + 1;
            n = n + 1; // Common
        } else if table[n + 1][o] >= table[n][o + 1] {
            n += 1; // Add from new
        } else {
            o += 1; // Remove from old
        }
    }
    res
}

#[test]
fn should_create_table_with_encode_pixel_array() {
    let old = [
        255, 255, 255, 5, 167, 167, 133, 7, 133, 71, 132, 4, 255, 255, 255, 10,
    ];
    let old_chunks: Vec<String> = old.to_vec().chunks(4).map(encode).collect();
    let new = [
        255, 255, 255, 5, 133, 71, 132, 4, 167, 167, 133, 7, 255, 255, 255, 10,
    ];
    let new_chunks: Vec<String> = new.to_vec().chunks(4).map(encode).collect();
    let lcs_table = create_table(&old_chunks, &new_chunks);
    assert_eq!(
        vec!["////BQ==", "p6eFBw==", "////Cg=="],
        gen_lcs(&lcs_table, &old_chunks, &new_chunks)
    );
    assert_eq!(vec![255, 255, 255, 5], decode("////BQ==").unwrap());
    assert_eq!(vec![167, 167, 133, 7], decode("p6eFBw==").unwrap());
    assert_eq!(vec![255, 255, 255, 10], decode("////Cg==").unwrap());
    assert_eq!(3, lcs_table[0][0]);
}

#[test]
fn should_create_table_with_strings2() {
    let old = [
        "HH", "ee", "ll", "ll", "oo", "  ", "ww", "oo", "rr", "ll", "dd",
    ];
    let new = [
        "HH", "aa", "cc", "kk", "yy", "ii", "nn", "  ", "oo", "oo", "zz",
    ];
    let lcs_table = create_table(&old, &new);
    let expected = vec![
        /* * * * * H  e  l  l  o  _  w  o  r  l  d  */
        /*H*/ vec![3, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*a*/ vec![2, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*c*/ vec![2, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*k*/ vec![2, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*y*/ vec![2, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*i*/ vec![2, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*n*/ vec![2, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*_*/ vec![2, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*o*/ vec![2, 2, 2, 2, 2, 1, 1, 1, 0, 0, 0, 0],
        /*o*/ vec![1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0],
        /*z*/ vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        /* */ vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    assert_eq!(
        ["HH", "oo", "oo"].iter().collect::<Vec<_>>(),
        gen_lcs(&lcs_table, &old, &new)
    );
    assert_eq!(expected, lcs_table);
}

#[test]
fn should_create_table_with_strings() {
    let old = ["H", "e", "l", "l", "o", " ", "w", "o", "r", "l", "d"];
    let new = ["H", "a", "c", "k", "y", "i", "n", " ", "o", "o", "z"];
    let lcs_table = create_table(&old, &new);
    let expected = vec![
        /* * * * * H  e  l  l  o  _  w  o  r  l  d  */
        /*H*/ vec![3, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*a*/ vec![2, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*c*/ vec![2, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*k*/ vec![2, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*y*/ vec![2, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*i*/ vec![2, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*n*/ vec![2, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*_*/ vec![2, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*o*/ vec![2, 2, 2, 2, 2, 1, 1, 1, 0, 0, 0, 0],
        /*o*/ vec![1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0],
        /*z*/ vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        /* */ vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    assert_eq!(["H", "o", "o"].iter().collect::<Vec<_>>(), gen_lcs(&lcs_table, &old, &new));
    assert_eq!(expected, lcs_table);
}

#[test]
fn should_create_table_with_chars() {
    let old = ['H', 'e', 'l', 'l', 'o', ' ', 'w', 'o', 'r', 'l', 'd'];
    let new = ['H', 'a', 'c', 'k', 'y', 'i', 'n', ' ', 'o', 'o', 'z'];
    let lcs_table = create_table(&old, &new);
    let expected = vec![
        /* * * * * H  e  l  l  o  _  w  o  r  l  d  */
        /*H*/ vec![3, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*a*/ vec![2, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*c*/ vec![2, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*k*/ vec![2, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*y*/ vec![2, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*i*/ vec![2, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*n*/ vec![2, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*_*/ vec![2, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0],
        /*o*/ vec![2, 2, 2, 2, 2, 1, 1, 1, 0, 0, 0, 0],
        /*o*/ vec![1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0],
        /*z*/ vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        /* */ vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    assert_eq!(
        ['H', 'o', 'o'].iter().collect::<Vec<_>>(),
        gen_lcs(&lcs_table, &old, &new)
    );
    assert_eq!(expected, lcs_table);
}

#[test]
fn should_create_table_with_numbers() {
    let old = [1, 2, 3, 4];
    let new = [2, 4];
    let lcs_table = create_table(&old, &new);
    let expected = vec![
        vec![2, 2, 1, 1, 0],
        vec![1, 1, 1, 1, 0],
        vec![0, 0, 0, 0, 0],
    ];
    let res = gen_lcs(&lcs_table, &old, &new);
    assert_eq!([2, 4].iter().collect::<Vec<_>>(), res);
    assert_eq!(expected, lcs_table);
}
