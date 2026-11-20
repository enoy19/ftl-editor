//! FTL's bitmap fonts (`fonts/*.font` in `ftl.dat`, FTL 1.6.x) as glyph
//! metrics and an atlas PNG.
//!
//! The layout was read from the bytes of the seven Latin fonts of 1.6.13
//! (`docs/original-look.md` has the hexdumps); no public description was
//! found. All numbers are big-endian.
//!
//! ```text
//! 0   "FONT"
//! 4   u8 1 (version)   u8 line height   u8 baseline   u8 0
//! 8   u32 24           offset of the first glyph record
//! 12  u16 count        u16 16 (size of a glyph record)
//! 16  u32 tex          offset of the TEX block
//! 20  u32              length of the TEX block (32 + width * height)
//! 24  count x glyph record (16 bytes):
//!     u32 code point, u16 x, u16 y, u8 w, u8 h,
//!     i8 top (rows above the baseline), u8 0,
//!     i8 left (space before), u8 0, i8 right (space after), u8 pad
//!     (the HL fonts fill the last byte with 0x9a or 0x66)
//! ..  zero padding up to tex
//! tex "TEX\n", u16 0x0240, u16 0, u16 width, u16 height, u16 1, u16 0,
//!     u32 32 (header size), u32 width * height, then one byte per pixel,
//!     row by row: 0 or 255, the coverage of the glyph
//! ```
//!
//! The advance of a glyph is `left + w + right`. The baseline equals the
//! tallest `top` in HL1, HL2, JustinFont8 and num_font; in JustinFont10,
//! 11Bold and 12Bold a few accented capitals reach 1 to 3 rows above it.
//! The line height is baseline plus descent plus 3 in JustinFont8 and plus 0
//! in the others; it is read, not derived.

use std::io::Cursor;

/// One glyph of a [`BitmapFont`]: where it sits in the atlas and how it is
/// placed on a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyph {
    pub code_point: u32,
    pub x: u16,
    pub y: u16,
    pub width: u8,
    pub height: u8,
    /// Rows of the glyph above the baseline (negative: it starts below).
    pub top: i8,
    /// Columns before the glyph.
    pub left: i8,
    /// Columns after the glyph.
    pub right: i8,
}

impl Glyph {
    /// How far the pen moves after this glyph.
    pub fn advance(&self) -> i32 {
        i32::from(self.left) + i32::from(self.width) + i32::from(self.right)
    }
}

/// A decoded `.font`: glyphs and the atlas they point into.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitmapFont {
    pub glyphs: Vec<Glyph>,
    pub atlas_width: u16,
    pub atlas_height: u16,
    /// One coverage byte per pixel, row by row (0 or 255 in every 1.6.13 font).
    pub coverage: Vec<u8>,
    /// Header byte 5: the distance from one line to the next.
    pub line_height: u8,
    /// Header byte 6: rows from the top of a line to the baseline.
    pub baseline: u8,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("{0}")]
pub struct FontError(String);

const GLYPH_RECORD: usize = 16;
const TEX_HEADER: usize = 32;

impl BitmapFont {
    /// Decodes the bytes of a `.font` file.
    pub fn parse(bytes: &[u8]) -> Result<BitmapFont, FontError> {
        let err = |m: String| Err(FontError(m));
        let u16_at = |o: usize| {
            bytes
                .get(o..o + 2)
                .map(|b| u16::from_be_bytes([b[0], b[1]]))
        };
        let u32_at = |o: usize| {
            bytes
                .get(o..o + 4)
                .map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
        };
        if bytes.get(..4) != Some(b"FONT") {
            return err("no FONT signature".into());
        }
        let (Some(first), Some(count), Some(size), Some(tex), Some(tex_len)) =
            (u32_at(8), u16_at(12), u16_at(14), u32_at(16), u32_at(20))
        else {
            return err(format!(
                "the header needs 24 bytes, the file has {}",
                bytes.len()
            ));
        };
        if bytes[4] != 1 {
            return err(format!("version {} (known: 1)", bytes[4]));
        }
        if usize::from(size) != GLYPH_RECORD {
            return err(format!(
                "glyph records of {size} bytes (known: {GLYPH_RECORD})"
            ));
        }
        let (first, tex) = (first as usize, tex as usize);
        let records_end = first + usize::from(count) * GLYPH_RECORD;
        if records_end > tex || tex > bytes.len() {
            return err(format!(
                "{count} glyph records from {first} end at {records_end}, the TEX block starts at {tex}, the file has {} bytes",
                bytes.len()
            ));
        }
        let glyphs = bytes[first..records_end]
            .as_chunks::<GLYPH_RECORD>()
            .0
            .iter()
            .map(|r| Glyph {
                code_point: u32::from_be_bytes([r[0], r[1], r[2], r[3]]),
                x: u16::from_be_bytes([r[4], r[5]]),
                y: u16::from_be_bytes([r[6], r[7]]),
                width: r[8],
                height: r[9],
                top: r[10] as i8,
                left: r[12] as i8,
                right: r[14] as i8,
            })
            .collect::<Vec<_>>();

        if bytes.get(tex..tex + 4) != Some(b"TEX\n") {
            return err(format!("no TEX block at {tex}"));
        }
        let (Some(width), Some(height), Some(header), Some(len)) = (
            u16_at(tex + 8),
            u16_at(tex + 10),
            u32_at(tex + 16),
            u32_at(tex + 20),
        ) else {
            return err(format!("the TEX header at {tex} is cut off"));
        };
        let pixels = usize::from(width) * usize::from(height);
        if header as usize != TEX_HEADER
            || len as usize != pixels
            || tex_len as usize != TEX_HEADER + pixels
        {
            return err(format!(
                "TEX {width}x{height}: header {header} bytes, {len} pixel bytes, block {tex_len} bytes (known: {TEX_HEADER}, one byte per pixel, their sum)"
            ));
        }
        let start = tex + TEX_HEADER;
        let Some(coverage) = bytes.get(start..start + pixels) else {
            return err(format!(
                "the TEX block needs {pixels} pixel bytes from {start}, the file has {}",
                bytes.len()
            ));
        };
        if let Some(g) = glyphs.iter().find(|g| {
            usize::from(g.x) + usize::from(g.width) > usize::from(width)
                || usize::from(g.y) + usize::from(g.height) > usize::from(height)
        }) {
            return err(format!(
                "glyph U+{:04X} lies outside the {width}x{height} atlas",
                g.code_point
            ));
        }
        Ok(BitmapFont {
            glyphs,
            atlas_width: width,
            atlas_height: height,
            coverage: coverage.to_vec(),
            line_height: bytes[5],
            baseline: bytes[6],
        })
    }

    pub fn glyph(&self, c: char) -> Option<&Glyph> {
        self.glyphs.iter().find(|g| g.code_point == u32::from(c))
    }

    /// Rows above the baseline of the tallest glyph.
    pub fn ascent(&self) -> i32 {
        self.glyphs
            .iter()
            .map(|g| i32::from(g.top))
            .max()
            .unwrap_or(0)
    }

    /// Rows below the baseline of the deepest glyph.
    pub fn descent(&self) -> i32 {
        self.glyphs
            .iter()
            .map(|g| i32::from(g.height) - i32::from(g.top))
            .max()
            .unwrap_or(0)
            .max(0)
    }

    /// The atlas as a white PNG whose alpha is the coverage, so a renderer
    /// tints it with any color.
    pub fn atlas_png(&self) -> Vec<u8> {
        let pixels: Vec<u8> = self.coverage.iter().flat_map(|&a| [255, a]).collect();
        let img = image::GrayAlphaImage::from_raw(
            u32::from(self.atlas_width),
            u32::from(self.atlas_height),
            pixels,
        )
        .expect("coverage holds width * height bytes");
        let mut out = Cursor::new(Vec::new());
        img.write_to(&mut out, image::ImageFormat::Png)
            .expect("encoding a PNG into memory does not fail");
        out.into_inner()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A font of two glyphs on a 4x2 atlas, built byte by byte.
    pub(crate) fn tiny_font() -> Vec<u8> {
        let mut b = b"FONT".to_vec();
        b.extend([1, 19, 12, 0]);
        b.extend(24u32.to_be_bytes());
        b.extend(2u16.to_be_bytes());
        b.extend(16u16.to_be_bytes());
        let tex = 24 + 2 * 16 + 8;
        b.extend((tex as u32).to_be_bytes());
        b.extend(40u32.to_be_bytes());
        // 'A' at 0,0, 2x2, top 2, left 0, right 1
        b.extend(u32::from('A').to_be_bytes());
        b.extend([0, 0, 0, 0, 2, 2, 2, 0, 0, 0, 1, 0]);
        // ',' at 2,0, 1x2, top 1 (one row below the baseline), left 1, right -1, pad 0x9a
        b.extend(u32::from(',').to_be_bytes());
        b.extend([0, 2, 0, 0, 1, 2, 1, 0, 1, 0, 0xff, 0x9a]);
        b.extend([0; 8]);
        b.extend(b"TEX\n");
        b.extend([0x02, 0x40, 0, 0, 0, 4, 0, 2, 0, 1, 0, 0]);
        b.extend(32u32.to_be_bytes());
        b.extend(8u32.to_be_bytes());
        b.extend([0; 8]);
        b.extend([255, 0, 255, 0, 0, 255, 255, 0]);
        b
    }

    #[test]
    fn decodes_glyphs_metrics_and_coverage_of_a_built_font() {
        let f = BitmapFont::parse(&tiny_font()).unwrap();
        assert_eq!((f.atlas_width, f.atlas_height, f.glyphs.len()), (4, 2, 2));
        let a = *f.glyph('A').unwrap();
        assert_eq!(
            (a.x, a.y, a.width, a.height, a.top, a.left, a.right),
            (0, 0, 2, 2, 2, 0, 1)
        );
        assert_eq!(a.advance(), 3);
        let comma = *f.glyph(',').unwrap();
        assert_eq!(
            (comma.x, comma.top, comma.left, comma.right, comma.advance()),
            (2, 1, 1, -1, 1)
        );
        assert_eq!((f.ascent(), f.descent()), (2, 1));
        assert_eq!(f.coverage, [255, 0, 255, 0, 0, 255, 255, 0]);
        assert_eq!((f.line_height, f.baseline), (19, 12));
    }

    #[test]
    fn the_atlas_png_is_white_with_the_coverage_as_alpha() {
        let f = BitmapFont::parse(&tiny_font()).unwrap();
        let img = image::load_from_memory(&f.atlas_png())
            .unwrap()
            .to_luma_alpha8();
        assert_eq!(img.dimensions(), (4, 2));
        let alpha: Vec<u8> = img.pixels().map(|p| p.0[1]).collect();
        assert_eq!(alpha, f.coverage);
        assert!(img.pixels().all(|p| p.0[0] == 255));
    }

    #[test]
    fn refuses_a_damaged_font_with_the_reason() {
        let font = tiny_font();
        let mut no_sig = font.clone();
        no_sig[0] = b'X';
        let mut wrong_record = font.clone();
        wrong_record[15] = 20;
        let mut outside = font.clone();
        outside[24 + 16 + 8] = 3; // ',' 3 wide at x 2 on a 4 wide atlas
        let cut = &font[..font.len() - 1];
        for (bytes, text) in [
            (&no_sig[..], "no FONT signature"),
            (&wrong_record[..], "glyph records of 20 bytes (known: 16)"),
            (&outside[..], "glyph U+002C lies outside the 4x2 atlas"),
            (
                cut,
                "the TEX block needs 8 pixel bytes from 96, the file has 103",
            ),
            (&font[..10], "the header needs 24 bytes, the file has 10"),
        ] {
            assert_eq!(BitmapFont::parse(bytes).unwrap_err().to_string(), text);
        }
    }
}
