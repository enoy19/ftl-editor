//! Cutting frames out of sprite sheets (`ImageUtilities.getCroppedImage`
//! of the Java editor), with the `image` crate.

use std::io::Cursor;

use crate::{Error, Rgba};

/// A rectangle `x, y, w, h` in pixels of a sheet.
pub type Rect = (u32, u32, u32, u32);

/// The standing frame of a crew race: the first 35x35 cell of
/// `img/people/<race>_base.png` (offsets 0, 0 in `DefaultSpriteImageProvider`).
pub const CREW_FRAME: Rect = (0, 0, 35, 35);

/// Moves the opaque pixels of a PNG to the middle of the same canvas: the
/// box around every pixel with alpha above 0 is centered (an odd remainder
/// leaves the extra pixel right or below). A fully transparent image stays.
///
/// FTL draws a crew cell centered on the crew position, but the art inside
/// the 35x35 cells is not centered: the visible Lanius of the standing frame
/// spans x 4..27 (its middle 2 px left of the cell's), Rock y 10..25, Slug
/// y 7..23 (measured in `ftl.dat` of 1.6.13 on 2026-11-19). A portrait
/// scaled to a card makes those pixels visible offsets.
pub(crate) fn center_opaque(inner_path: &str, png: &[u8]) -> Result<Vec<u8>, Error> {
    let err = |detail: String| Error::Image {
        inner_path: inner_path.to_owned(),
        detail,
    };
    let frame = image::load_from_memory_with_format(png, image::ImageFormat::Png)
        .map_err(|e| err(e.to_string()))?
        .to_rgba8();
    let (w, h) = frame.dimensions();
    let opaque = || frame.enumerate_pixels().filter(|(_, _, p)| p.0[3] > 0);
    let (Some(left), Some(right)) = (
        opaque().map(|(x, ..)| x).min(),
        opaque().map(|(x, ..)| x).max(),
    ) else {
        return Ok(png.to_vec());
    };
    let top = opaque().map(|(_, y, _)| y).min().unwrap_or(0);
    let bottom = opaque().map(|(_, y, _)| y).max().unwrap_or(0);
    let (bw, bh) = (right - left + 1, bottom - top + 1);
    let (dx, dy) = ((w - bw).div_ceil(2), (h - bh).div_ceil(2));
    let mut out = image::RgbaImage::new(w, h);
    image::imageops::replace(
        &mut out,
        &image::imageops::crop_imm(&frame, left, top, bw, bh).to_image(),
        i64::from(dx),
        i64::from(dy),
    );
    let mut bytes = Cursor::new(Vec::new());
    out.write_to(&mut bytes, image::ImageFormat::Png)
        .map_err(|e| err(e.to_string()))?;
    Ok(bytes.into_inner())
}

/// Decodes `png`, cuts `rect` and encodes the cut as PNG. A rectangle outside
/// the sheet is an error (the Java editor logs it and draws a stand-in).
pub(crate) fn crop_png(inner_path: &str, png: &[u8], (x, y, w, h): Rect) -> Result<Vec<u8>, Error> {
    let err = |detail: String| Error::Image {
        inner_path: inner_path.to_owned(),
        detail,
    };
    let sheet = image::load_from_memory_with_format(png, image::ImageFormat::Png)
        .map_err(|e| err(e.to_string()))?;
    if x + w > sheet.width() || y + h > sheet.height() {
        return Err(err(format!(
            "the frame {w}x{h} at {x},{y} lies outside the {}x{} sheet",
            sheet.width(),
            sheet.height()
        )));
    }
    let mut out = Cursor::new(Vec::new());
    sheet
        .crop_imm(x, y, w, h)
        .write_to(&mut out, image::ImageFormat::Png)
        .map_err(|e| err(e.to_string()))?;
    Ok(out.into_inner())
}

/// Cuts `rect` of the base sheet and lays each layer sheet's cut over it,
/// multiplied with its color: the color times the pixel, its alpha times
/// the pixel's alpha, blended over what lies below (a GL color on a
/// texture, as FTL draws a tint layer).
pub(crate) fn tinted_frame(
    base_path: &str,
    base: &[u8],
    layers: &[(String, &[u8], Rgba)],
    (x, y, w, h): Rect,
) -> Result<Vec<u8>, Error> {
    let decode = |path: &str, png: &[u8]| {
        image::load_from_memory_with_format(png, image::ImageFormat::Png)
            .map(|i| i.to_rgba8())
            .map_err(|e| Error::Image {
                inner_path: path.to_owned(),
                detail: e.to_string(),
            })
    };
    let sheet = decode(base_path, base)?;
    if x + w > sheet.width() || y + h > sheet.height() {
        return Err(Error::Image {
            inner_path: base_path.to_owned(),
            detail: format!("the frame {w}x{h} at {x},{y} lies outside the sheet"),
        });
    }
    let mut out = image::imageops::crop_imm(&sheet, x, y, w, h).to_image();
    for (path, png, color) in layers {
        let layer = decode(path, png)?;
        for (px, py, dst) in out.enumerate_pixels_mut() {
            let (lx, ly) = (x + px, y + py);
            if lx >= layer.width() || ly >= layer.height() {
                continue;
            }
            let src = layer.get_pixel(lx, ly).0;
            let tint = [color.r, color.g, color.b];
            let sa = f32::from(src[3]) * f32::from(color.a) / (255.0 * 255.0);
            if sa <= 0.0 {
                continue;
            }
            let da = f32::from(dst.0[3]) / 255.0;
            let oa = sa + da * (1.0 - sa);
            for c in 0..3 {
                let s = f32::from(src[c]) * f32::from(tint[c]) / 255.0;
                let d = f32::from(dst.0[c]);
                dst.0[c] = ((s * sa + d * da * (1.0 - sa)) / oa).round() as u8;
            }
            dst.0[3] = (oa * 255.0).round() as u8;
        }
    }
    let mut bytes = Cursor::new(Vec::new());
    out.write_to(&mut bytes, image::ImageFormat::Png)
        .map_err(|e| Error::Image {
            inner_path: base_path.to_owned(),
            detail: e.to_string(),
        })?;
    Ok(bytes.into_inner())
}
