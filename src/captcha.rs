//! Locates the slider gap in the MIIT captcha background image.

use base64::Engine;
use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD};

use crate::{Error, Result};

const COLOR_TOLERANCE: i32 = 12;
const RELAXED_COLOR_TOLERANCE: i32 = 24;
const MIN_COMPONENT_AREA: i32 = 900;
const MIN_SIDE_LENGTH: i32 = 24;
const TOP_HINT_ALLOWANCE: i32 = 8;
const GAP_APPROX_SIZE: i32 = 72;
const TARGET_COLOR: [u8; 3] = [199, 186, 183];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub area: i32,
}

struct Pixels {
    width: usize,
    height: usize,
    /// Row-major alpha-premultiplied RGB.
    rgb: Vec<[u8; 3]>,
}

impl Pixels {
    fn decode(data: &[u8]) -> Result<Self> {
        let img = image::load_from_memory(data)?.into_rgba8();
        let (width, height) = img.dimensions();
        let rgb = img.pixels().map(|p| premultiply(p.0)).collect();
        Ok(Self {
            width: width as usize,
            height: height as usize,
            rgb,
        })
    }
}

/// Matches Go's `color.NRGBA.RGBA() >> 8` so tolerances behave identically.
fn premultiply([r, g, b, a]: [u8; 4]) -> [u8; 3] {
    let a = u32::from(a);
    let channel = |c: u8| ((u32::from(c) * 0x101 * a / 0xff) >> 8) as u8;
    [channel(r), channel(g), channel(b)]
}

pub(crate) fn detect_square_base64_with_hint(encoded: &str, top_hint: i32) -> Result<Rect> {
    let pixels = Pixels::decode(&decode_base64_image(encoded)?)?;

    for tolerance in [COLOR_TOLERANCE, RELAXED_COLOR_TOLERANCE] {
        if let Some(found) = find_captcha_square(&pixels, tolerance, top_hint) {
            return Ok(found);
        }
    }

    if top_hint >= 0 {
        return Ok(estimate_gap_from_hint(&pixels, top_hint));
    }

    Err(Error::SquareNotFound)
}

fn decode_base64_image(encoded: &str) -> Result<Vec<u8>> {
    let cleaned = encoded.trim();
    let cleaned = cleaned.split_once(',').map_or(cleaned, |(_, data)| data);

    STANDARD
        .decode(cleaned)
        .or_else(|_| STANDARD_NO_PAD.decode(cleaned))
        .map_err(|_| Error::Base64)
}

fn find_captcha_square(pixels: &Pixels, tolerance: i32, top_hint: i32) -> Option<Rect> {
    let mut visited = vec![false; pixels.rgb.len()];
    let mut queue = Vec::new();
    let mut best: Option<Rect> = None;

    for index in 0..pixels.rgb.len() {
        if visited[index] {
            continue;
        }
        visited[index] = true;

        if !is_near_target(pixels.rgb[index], tolerance) {
            continue;
        }

        let component = flood_fill(pixels, index, tolerance, &mut visited, &mut queue);
        if !looks_like_square(&component) || !matches_top_hint(&component, top_hint) {
            continue;
        }

        if best.is_none_or(|b| component.area > b.area) {
            best = Some(component);
        }
    }

    best
}

fn flood_fill(
    pixels: &Pixels,
    start: usize,
    tolerance: i32,
    visited: &mut [bool],
    queue: &mut Vec<usize>,
) -> Rect {
    let width = pixels.width;
    let (start_x, start_y) = ((start % width) as i32, (start / width) as i32);
    let mut component = Rect {
        left: start_x,
        top: start_y,
        right: start_x,
        bottom: start_y,
        area: 0,
    };

    queue.clear();
    queue.push(start);
    let mut head = 0;
    while let Some(&index) = queue.get(head) {
        head += 1;
        let (x, y) = (index % width, index / width);

        component.area += 1;
        component.left = component.left.min(x as i32);
        component.right = component.right.max(x as i32);
        component.top = component.top.min(y as i32);
        component.bottom = component.bottom.max(y as i32);

        let neighbors = [
            (x > 0).then(|| index - 1),
            (x + 1 < width).then(|| index + 1),
            (y > 0).then(|| index - width),
            (y + 1 < pixels.height).then(|| index + width),
        ];
        for next in neighbors.into_iter().flatten() {
            if visited[next] {
                continue;
            }
            visited[next] = true;

            if is_near_target(pixels.rgb[next], tolerance) {
                queue.push(next);
            }
        }
    }

    component
}

fn estimate_gap_from_hint(pixels: &Pixels, top_hint: i32) -> Rect {
    let width = pixels.width as i32;
    let height = pixels.height as i32;
    let top = top_hint.clamp(0, (height - GAP_APPROX_SIZE).max(0));
    let bottom = height.min(top + GAP_APPROX_SIZE);

    // Per-column score sums over the hinted row band; each window is a slice sum.
    let mut column_scores = vec![0i32; pixels.width];
    for row in pixels
        .rgb
        .chunks_exact(pixels.width.max(1))
        .take(bottom as usize)
        .skip(top as usize)
    {
        for (sum, &pixel) in column_scores.iter_mut().zip(row) {
            *sum += gap_pixel_score(pixel);
        }
    }

    let mut best_left = 0;
    let mut best_score = -1;
    for left in 0..=(width - GAP_APPROX_SIZE).max(0) {
        let end = width.min(left + GAP_APPROX_SIZE);
        let score: i32 = column_scores[left as usize..end as usize].iter().sum();
        if score > best_score {
            best_score = score;
            best_left = left;
        }
    }

    Rect {
        left: best_left,
        top,
        right: (width - 1).min(best_left + GAP_APPROX_SIZE - 1),
        bottom: (height - 1).min(top + GAP_APPROX_SIZE - 1),
        area: GAP_APPROX_SIZE * GAP_APPROX_SIZE,
    }
}

fn gap_pixel_score(pixel: [u8; 3]) -> i32 {
    let [r, g, b] = pixel.map(i32::from);
    let spread = r.max(g).max(b) - r.min(g).min(b);
    let avg = (r + g + b) / 3;

    if !(100..=235).contains(&avg) {
        return 0;
    }

    let mut score = (45 - spread * 2).max(0) + (30 - (avg - 189).abs()).max(0);
    if is_near_target(pixel, RELAXED_COLOR_TOLERANCE) {
        score += 25;
    }
    score
}

fn looks_like_square(rect: &Rect) -> bool {
    let width = rect.right - rect.left + 1;
    let height = rect.bottom - rect.top + 1;
    if rect.area < MIN_COMPONENT_AREA || width < MIN_SIDE_LENGTH || height < MIN_SIDE_LENGTH {
        return false;
    }

    if width > height {
        width - height <= width / 3
    } else {
        height - width <= height / 3
    }
}

fn matches_top_hint(rect: &Rect, top_hint: i32) -> bool {
    top_hint < 0 || (rect.top - top_hint).abs() <= TOP_HINT_ALLOWANCE
}

fn is_near_target(pixel: [u8; 3], tolerance: i32) -> bool {
    pixel
        .iter()
        .zip(TARGET_COLOR)
        .all(|(&c, t)| (i32::from(c) - i32::from(t)).abs() <= tolerance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageFormat, Rgb, RgbImage};
    use std::io::Cursor;

    fn png_base64(img: &RgbImage) -> String {
        let mut buf = Cursor::new(Vec::new());
        img.write_to(&mut buf, ImageFormat::Png).unwrap();
        format!(
            "data:image/png;base64,{}",
            STANDARD.encode(buf.into_inner())
        )
    }

    #[test]
    fn detects_target_colored_square_matching_top_hint() {
        let mut img = RgbImage::from_pixel(300, 150, Rgb([20, 60, 120]));
        // Decoy: right color and shape but far from the top hint.
        for y in 5..45 {
            for x in 10..50 {
                img.put_pixel(x, y, Rgb([199, 186, 183]));
            }
        }
        for y in 70..120 {
            for x in 180..230 {
                img.put_pixel(x, y, Rgb([205, 190, 180]));
            }
        }

        let found = detect_square_base64_with_hint(&png_base64(&img), 72).unwrap();
        assert_eq!(
            found,
            Rect {
                left: 180,
                top: 70,
                right: 229,
                bottom: 119,
                area: 2500
            }
        );
    }

    #[test]
    fn falls_back_to_grey_window_near_hint_when_no_square() {
        let mut img = RgbImage::from_pixel(300, 150, Rgb([20, 60, 120]));
        // Greyish patch too far from the target colour to flood-fill.
        for y in 40..112 {
            for x in 100..172 {
                img.put_pixel(x, y, Rgb([170, 170, 170]));
            }
        }

        let found = detect_square_base64_with_hint(&png_base64(&img), 40).unwrap();
        assert_eq!((found.left, found.top), (100, 40));
        assert!(matches!(
            detect_square_base64_with_hint(&png_base64(&img), -1),
            Err(Error::SquareNotFound)
        ));
    }
}
