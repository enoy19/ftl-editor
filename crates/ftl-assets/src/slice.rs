//! 9-slice insets of FTL's frame and button images, measured from the pixels.
//!
//! FTL's frames (`img/window_outline.png`, the `box_*` and `button_*`
//! images) have a straight outline between cut or rounded corners. The
//! insets keep the corners, with their glow, and leave only a uniform
//! middle to stretch: a stretched strip that is not uniform smears a corner
//! across the whole edge. Measuring at runtime keeps the
//! numbers in step with the installed `ftl.dat`; `docs/original-look.md`
//! lists what 1.6.13 answers.

use crate::Error;

/// Pixels from each edge that a 9-slice renderer must not stretch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Insets {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

/// The insets of the outermost opaque outline of `png`.
///
/// First the straight edges: each row's first opaque pixel (alpha 255)
/// gives the left outline. The rows where it reaches its minimum are the
/// straight left edge; the rows above and below it are the corners, so they
/// are the top and bottom insets. The columns give left and right the same
/// way. Then the uniform middle: the glow of a cut corner reaches past the
/// start of the straight edge (8 pixels in `img/window_outline.png`), so
/// left and right grow until every column of the top and bottom strips
/// equals the middle column, and top and bottom grow the same way over the
/// left and right strips. An image without an opaque pixel has no outline
/// and answers zero insets.
pub fn measure_insets(inner_path: &str, png: &[u8]) -> Result<Insets, Error> {
    let img = image::load_from_memory_with_format(png, image::ImageFormat::Png)
        .map_err(|e| Error::Image {
            inner_path: inner_path.to_owned(),
            detail: e.to_string(),
        })?
        .to_rgba8();
    let (w, h) = img.dimensions();
    let opaque = |x: u32, y: u32| img.get_pixel(x, y).0[3] == 255;
    let lefts: Vec<Option<u32>> = (0..h).map(|y| (0..w).find(|&x| opaque(x, y))).collect();
    let tops: Vec<Option<u32>> = (0..w).map(|x| (0..h).find(|&y| opaque(x, y))).collect();
    let (Some((top, bottom)), Some((left, right))) = (straight_run(&lefts), straight_run(&tops))
    else {
        return Ok(Insets::default());
    };
    let (top, bottom) = (top, h - 1 - bottom);
    let img = &img;
    let column = |x: u32| {
        (0..top)
            .chain(h - bottom..h)
            .map(move |y| img.get_pixel(x, y).0)
    };
    let (from, to) = uniform_run(w, |a, b| column(a).eq(column(b)));
    let (left, right) = (left.max(from), (w - 1 - right).max(w - 1 - to));
    let row = |y: u32| {
        (0..left)
            .chain(w - right..w)
            .map(move |x| img.get_pixel(x, y).0)
    };
    let (from, to) = uniform_run(h, |a, b| row(a).eq(row(b)));
    let (top, bottom) = keep_a_middle(top.max(from), bottom.max(h - 1 - to), h);
    let (left, right) = keep_a_middle(left, right, w);
    Ok(Insets {
        left,
        top,
        right,
        bottom,
    })
}

/// Insets that leave no middle (a slanted tab has no uniform row) are cut
/// back to a middle of one pixel, its centre.
fn keep_a_middle(a: u32, b: u32, len: u32) -> (u32, u32) {
    match a + b < len {
        true => (a, b),
        false => (a.min(len / 2), len - 1 - a.min(len / 2)),
    }
}

/// The widest run of indices around the middle of `0..len` that `same`
/// finds equal to the middle one.
fn uniform_run(len: u32, same: impl Fn(u32, u32) -> bool) -> (u32, u32) {
    let mid = len / 2;
    let from = (0..mid)
        .rev()
        .take_while(|&i| same(i, mid))
        .last()
        .unwrap_or(mid);
    let to = (mid + 1..len)
        .take_while(|&i| same(i, mid))
        .last()
        .unwrap_or(mid);
    (from, to)
}

/// The first and last index where `edge` reaches its minimum.
fn straight_run(edge: &[Option<u32>]) -> Option<(u32, u32)> {
    let min = edge.iter().flatten().min()?;
    let first = edge.iter().position(|e| e == &Some(*min))?;
    let last = edge.iter().rposition(|e| e == &Some(*min))?;
    Some((first as u32, last as u32))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// A `w` x `h` PNG: an opaque octagon outline whose corners are cut by
    /// `cut` pixels, `glow` transparent-ish pixels around it.
    fn octagon(w: u32, h: u32, cut: u32, glow: u32) -> Vec<u8> {
        let img = image::RgbaImage::from_fn(w, h, |x, y| {
            let inside = x >= glow && y >= glow && x < w - glow && y < h - glow;
            let (dx, dy) = (
                x.min(w - 1 - x).saturating_sub(glow),
                y.min(h - 1 - y).saturating_sub(glow),
            );
            match inside && dx + dy >= cut {
                true => image::Rgba([235, 245, 229, 255]),
                false => image::Rgba([121, 242, 253, 40]),
            }
        });
        let mut out = Cursor::new(Vec::new());
        img.write_to(&mut out, image::ImageFormat::Png).unwrap();
        out.into_inner()
    }

    #[test]
    fn the_insets_end_where_the_straight_edge_starts() {
        let i = measure_insets("t.png", &octagon(66, 72, 12, 7)).unwrap();
        assert_eq!(
            i,
            Insets {
                left: 19,
                top: 19,
                right: 19,
                bottom: 19
            }
        );
    }

    #[test]
    fn a_corner_glow_past_the_straight_edge_widens_the_insets() {
        let png = octagon(66, 72, 12, 7);
        let mut img = image::load_from_memory(&png).unwrap().to_rgba8();
        img.put_pixel(26, 3, image::Rgba([121, 242, 253, 90])); // top strip, right of the straight start
        img.put_pixel(3, 50, image::Rgba([121, 242, 253, 90])); // left strip, above the bottom corner
        let mut out = Cursor::new(Vec::new());
        img.write_to(&mut out, image::ImageFormat::Png).unwrap();
        let i = measure_insets("t.png", &out.into_inner()).unwrap();
        assert_eq!(
            i,
            Insets {
                left: 27,
                top: 19,
                right: 19,
                bottom: 72 - 50
            }
        );
    }

    #[test]
    fn insets_that_leave_no_middle_keep_one_middle_pixel() {
        assert_eq!(keep_a_middle(26, 14, 29), (14, 14));
        assert_eq!(keep_a_middle(10, 10, 21), (10, 10));
        assert_eq!(keep_a_middle(10, 11, 21), (10, 10));
    }

    #[test]
    fn a_rectangle_without_cut_corners_has_insets_of_its_glow() {
        let i = measure_insets("t.png", &octagon(20, 10, 0, 2)).unwrap();
        assert_eq!(
            i,
            Insets {
                left: 2,
                top: 2,
                right: 2,
                bottom: 2
            }
        );
    }

    #[test]
    fn an_image_without_an_opaque_pixel_has_no_insets() {
        let img = image::RgbaImage::from_pixel(4, 4, image::Rgba([0, 0, 0, 10]));
        let mut out = Cursor::new(Vec::new());
        img.write_to(&mut out, image::ImageFormat::Png).unwrap();
        assert_eq!(
            measure_insets("t.png", &out.into_inner()).unwrap(),
            Insets::default()
        );
    }

    #[test]
    fn bytes_that_are_no_png_name_the_entry() {
        let err = measure_insets("img/x.png", b"nope").unwrap_err();
        assert!(err.to_string().starts_with("img/x.png: "), "{err}");
    }
}
