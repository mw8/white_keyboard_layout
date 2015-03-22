#![feature(old_io)]
#![feature(old_path)]
#![feature(collections)]

extern crate image;
extern crate freetype;

use std::num::Float;
use image::Rgba;
use image::Pixel;
use image::ImageBuffer;
use image::GenericImage;
use freetype::Bitmap;
use freetype::Library;
use freetype::face::Face;
use freetype::face::RENDER;


// Customizable parameters
static LARGE_KEY_SIZE:            u32   = 40; // pixels
static LARGE_KEY_PADDING:         u32   =  1; // pixels
static LETTERS_PT_SIZE:           isize = 20; // pt
static SYMBOLS_PT_SIZE:           isize = 15; // pt

static DOUBLE_METRIC_KEY_SIZE:    u32   = 20; // pixels
static DOUBLE_METRIC_KEY_PADDING: u32   =  0; // pixels
static DOUBLE_METRIC_PT_SIZE:     isize = 10; // pt

// LAYOUT STRINGS
const QWERTY_STRING: &'static str = "\
 `1234567890-=\
  qwertyuiop[]\\\
   asdfghjkl;'\
    zxcvbnm,./\
 ~!@#$%^&*()_+\
  QWERTYUIOP{}|\
   ASDFGHJKL:\"\
    ZXCVBNM<>?";

const DVORAK_STRING: &'static str = "\
`1234567890[]\
  ',.pyfgcrl/=\\\
   aoeuidhtns-\
    ;qjkxbmwvz\
~!@#$%^&*(){}\
  \"<>PYFGCRL?+|\
   AOEUIDHTNS_\
    :QJKXBMWVZ";

const COLEMAK_STRING: &'static str = "\
`1234567890-=\
  qwfpgjluy;[]\\\
   arstdhneio'\
    zxcvbkm,./\
~!@#$%^&*()_+\
  QWFPGJLUY:{}|\
   ARSTDHNEIO\"\
    ZXCVBKM<>?";

const WORKMAN_STRING: &'static str = "\
`1234567890-=\
  qdrwbjfup;[]\\\
   ashtgyneoi'\
    zxmcvkl,./\
~!@#$%^&*()_+\
  QDRWBJFUP:{}|\
   ASHTGYNEOI\"\
    ZXMCVKL<>?";
    
const PROTO_1_STRING: &'static str = "\
|12345%~67890\
  vyd,'+jmlu_(*\
   atheb=csnoi\
    pkgwqxrf.z\
`^&/}<#@!:{]$\
  VYD?\">JMLU-;\\\
   ATHEB[CSNOI\
    PKGWQXRF)Z";

const WHITE_STRING: &'static str = "\
#12345@$67890\
  vyd,._jmlu()=\
   atheb-csnoi\
    pkgwqxrf'z\
`!<>/|~%\\*[]^\
  VYD;:&JMLU{}?\
   ATHEB+CSNOI\
    PKGWQXRF\"Z";

// Which fingers correspond to which keys
const FINGER_ASSIGNMENT: [usize; 48] = [ 0,
1,  1,  2,  3,  3,  4,  4,  5,  5,  6,  6,  7,  8,
      1,  2,  3,  4,  4,  4,  5,  5,  6,  7,  8,  8,  8,
        1,  2,  3,  4,  4,  5,  5,  5,  6,  7,  8,
          2,  3,  4,  4,  4,  5,  5,  5,  6,  7];

// Which color to use for each finger
const FINGER_COLORS: [(u8, u8, u8, u8); 9] = [
    (0x00, 0x00, 0x00, 0xFF),
    (0x8C, 0x51, 0x0A, 0xFF),
    (0xBF, 0x81, 0x2D, 0xFF),
    (0xDF, 0xC2, 0x7D, 0xFF),
    (0xF6, 0xE8, 0xC3, 0xFF),
    (0xC7, 0xEA, 0xE5, 0xFF),
    (0x80, 0xCD, 0xC1, 0xFF),
    (0x35, 0x97, 0x8F, 0xFF),
    (0x01, 0x66, 0x5E, 0xFF)];

// Penalties for using each key
const SINGLE_METRIC: [f32; 48] = [0.0,
9.0,  7.0,  4.5,  3.5,  3.5,  6.0,  8.0,  9.5,  6.5,  3.5,  3.5,  4.5,  7.0,
         2.5,  0.1, -0.2,  1.0,  2.0,  5.0,  2.5,  1.0, -0.2,  0.1,  2.5,  3.0,  5.0,
           -0.5, -0.9, -1.2, -1.0,  1.0,  4.5,  1.0, -1.0, -1.2, -0.9, -0.5,
               2.0,  2.0,  0.5,  0.0,  3.0,  3.0,  0.0,  0.5,  2.0,  2.0];
               
const WORKMAN_METRIC: [f32; 48] = [0.0,
0.0,  6.0, -1.5,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
         2.0,  0.0,  0.0,  1.0,  2.0,  3.0,  1.0,  0.0,  0.0,  2.0, 0.01,  0.01, 0.01,
           -0.5, -1.0, -1.0, -1.0,  1.0,  1.0, -1.0, -1.0, -1.0, -0.5, 0.01,
               2.0,  2.0,  1.0,  0.0,  3.0,  1.0,  0.0,  1.0,  2.0,  2.0];

// Triples (k1, k2, p) that will assign a penalty p for transitioning between key k1 and key k2
const DOUBLE_METRIC: [(u8, u8, f32); 314] = [
    // left pinky
    ( 1,  2,  2.0),
    ( 1,  3, -0.5),
    ( 1, 14,  3.0),
    ( 1, 15,  0.5),
    ( 1, 27,  4.0),
    ( 1, 28,  2.5),
    ( 1, 38,  4.0),

    ( 2,  3, -1.5),
    ( 2, 14,  1.5),
    ( 2, 15,  0.5),
    ( 2, 27,  3.0),
    ( 2, 28,  2.0),
    ( 2, 38,  4.0),

    (14, 15, -2.0),
    (14, 27,  1.5),
    (14, 28,  0.0),
    (14, 38,  2.0),

    (27, 28, -2.0),
    (27, 38, -1.0),

    // left ring
    ( 3,  4, -2.0),
    ( 3,  5,  0.0),
    ( 3, 14, -1.0),
    ( 3, 15,  1.5),
    ( 3, 16,  0.0),
    ( 3, 27,  1.0),
    ( 3, 28,  3.0),
    ( 3, 29,  1.0),
    ( 3, 38,  5.0),
    ( 3, 39,  3.0),

    (15, 16, -2.0),
    (15, 27, -1.0),
    (15, 28,  1.5),
    (15, 29,  0.0),
    (15, 38,  3.0),
    (15, 39,  2.0),

    (28, 29, -2.0),
    (28, 38,  1.5),
    (28, 39, -1.0),

    (38, 39, -2.0),

    // left middle
    ( 4,  5,  2.0),
    ( 4,  6,  0.0),
    ( 4,  7,  2.0),
    ( 4, 15, -1.0),
    ( 4, 16,  1.5),
    ( 4, 17, -1.0),
    ( 4, 18,  1.0),
    ( 4, 19,  3.0),
    ( 4, 28,  1.0),
    ( 4, 29,  3.0),
    ( 4, 30,  1.0),
    ( 4, 31,  3.0),
    ( 4, 38,  3.0),
    ( 4, 39,  5.0),
    ( 4, 40,  3.0),
    ( 4, 41,  5.0),
    ( 4, 42,  7.0),

    ( 5,  6, -1.5),
    ( 5,  7,  0.5),
    ( 5, 15,  0.0),
    ( 5, 16,  1.5),
    ( 5, 17, -2.0),
    ( 5, 18,  0.0),
    ( 5, 19,  2.0),
    ( 5, 28,  1.0),
    ( 5, 29,  3.0),
    ( 5, 30,  1.0),
    ( 5, 31,  3.0),
    ( 5, 38,  3.0),
    ( 5, 39,  5.0),
    ( 5, 40,  2.0),
    ( 5, 41,  4.0),
    ( 5, 42,  6.0),

    (16, 17, -1.0),
    (16, 18,  1.0),
    (16, 19,  3.0),
    (16, 28, -1.0),
    (16, 29,  1.5),
    (16, 30, -1.5),
    (16, 31,  0.5),
    (16, 38,  1.0),
    (16, 39,  3.0),
    (16, 40,  0.0),
    (16, 41,  2.0),
    (16, 42,  4.0),

    (29, 30, -2.0),
    (29, 31,  0.0),
    (29, 38,  0.0),
    (29, 39,  1.5),
    (29, 40, -1.0),
    (29, 41, -0.5),
    (29, 42,  1.5),

    (39, 40, -2.0),
    (39, 41,  0.0),
    (39, 42,  2.0),

    // left index
    ( 6,  7,  2.0),
    ( 6, 16,  1.0),
    ( 6, 17,  1.5),
    ( 6, 18,  1.5),
    ( 6, 19,  3.0),
    ( 6, 29,  1.0),
    ( 6, 30,  3.0),
    ( 6, 31,  3.0),
    ( 6, 39,  4.0),
    ( 6, 40,  5.0),
    ( 6, 41,  5.0),
    ( 6, 42,  6.0),

    ( 7, 16,  3.0),
    ( 7, 17,  3.0),
    ( 7, 18,  1.5),
    ( 7, 19,  1.5),
    ( 7, 29,  3.0),
    ( 7, 30,  4.0),
    ( 7, 31,  3.0),
    ( 7, 39,  5.0),
    ( 7, 40,  6.0),
    ( 7, 41,  5.0),
    ( 7, 42,  5.0),

    (17, 18,  2.0),
    (17, 19,  4.0),
    (17, 29, -1.0),
    (17, 30,  1.5),
    (17, 31,  3.0),
    (17, 39,  2.0),
    (17, 40,  3.0),
    (17, 41,  4.0),
    (17, 42,  6.0),

    (18, 19,  2.0),
    (18, 29,  0.0),
    (18, 30,  1.5),
    (18, 31,  1.5),
    (18, 39,  3.0),
    (18, 40,  4.0),
    (18, 41,  3.0),
    (18, 42,  4.0),

    (19, 29,  3.0),
    (19, 30,  3.0),
    (19, 31,  1.5),
    (19, 39,  5.0),
    (19, 40,  6.0),
    (19, 41,  4.0),
    (19, 42,  3.0),

    (30, 31,  2.0),
    (30, 39,  0.0),
    (30, 40,  1.5),
    (30, 41,  1.5),
    (30, 42,  3.0),

    (31, 39,  2.0),
    (31, 40,  3.0),
    (31, 41,  1.5),
    (31, 42,  1.5),

    (40, 41,  2.0),
    (40, 42,  4.0),

    (41, 42,  2.0),

    // right index
    ( 8,  9,  2.0),
    ( 8, 10,  0.0),
    ( 8, 11,  2.0),
    ( 8, 20,  1.5),
    ( 8, 21,  3.0),
    ( 8, 22,  2.0),
    ( 8, 32,  3.0),
    ( 8, 33,  3.0),
    ( 8, 34,  4.0),
    ( 8, 35,  3.0),
    ( 8, 43,  5.0),
    ( 8, 44,  5.0),
    ( 8, 45,  6.0),
    ( 8, 46,  5.0),

    ( 9, 10, -1.5),
    ( 9, 11,  0.5),
    ( 9, 20,  1.5),
    ( 9, 21,  1.5),
    ( 9, 22,  0.0),
    ( 9, 32,  4.0),
    ( 9, 33,  3.0),
    ( 9, 34,  3.0),
    ( 9, 35,  1.0),
    ( 9, 43,  6.0),
    ( 9, 44,  5.0),
    ( 9, 45,  5.0),
    ( 9, 46,  4.0),

    (20, 21,  2.0),
    (20, 22,  0.0),
    (20, 32,  1.5),
    (20, 33,  1.5),
    (20, 34,  3.0),
    (20, 35,  2.0),
    (20, 43,  3.0),
    (20, 44,  4.0),
    (20, 45,  5.0),
    (20, 46,  4.0),

    (21, 22, -1.0),
    (21, 32,  3.0),
    (21, 33,  1.5),
    (21, 34,  1.5),
    (21, 35,  0.0),
    (21, 43,  4.0),
    (21, 44,  3.0),
    (21, 45,  4.0),
    (21, 46,  3.0),

    (32, 33,  2.0),
    (32, 34,  4.0),
    (32, 35,  3.0),
    (32, 43,  1.5),
    (32, 44,  3.0),
    (32, 45,  5.0),
    (32, 46,  5.0),

    (33, 34,  3.0),
    (33, 35,  0.0),
    (33, 43,  1.5),
    (33, 44,  1.5),
    (33, 45,  3.0),
    (33, 46,  2.0),

    (34, 35, -2.0),
    (34, 43,  3.0),
    (34, 44,  1.5),
    (34, 45,  1.5),
    (34, 46,  0.0),

    (43, 44,  2.0),
    (43, 45,  4.0),
    (43, 46,  3.0),

    (44, 45,  2.0),
    (44, 46,  0.0),

    (45, 46, -2.0),

    // right middle
    (10, 11,  2.0),
    (10, 12,  0.0),
    (10, 20,  0.0),
    (10, 21, -2.0),
    (10, 22,  1.5),
    (10, 23,  0.0),
    (10, 32,  3.0),
    (10, 33,  1.0),
    (10, 34,  0.0),
    (10, 35,  3.0),
    (10, 36,  2.0),
    (10, 43,  6.0),
    (10, 44,  4.0),
    (10, 45,  2.0),
    (10, 46,  5.0),
    (10, 47,  4.0),

    (11, 12, -2.0),
    (11, 20,  2.0),
    (11, 21,  0.0),
    (11, 22,  1.5),
    (11, 23, -1.0),
    (11, 32,  4.0),
    (11, 33,  2.0),
    (11, 34,  0.0),
    (11, 35,  3.0),
    (11, 36,  1.0),
    (11, 43,  6.0),
    (11, 44,  4.0),
    (11, 45,  2.0),
    (11, 46,  5.0),
    (11, 47,  3.0),

    (22, 23, -2.0),
    (22, 32,  3.0),
    (22, 33,  1.0),
    (22, 34, -1.5),
    (22, 35,  1.5),
    (22, 36,  0.0),
    (22, 43,  4.0),
    (22, 44,  2.0),
    (22, 45,  0.0),
    (22, 46,  3.0),
    (22, 47,  2.0),

    (35, 36, -2.0),
    (35, 43,  0.0),
    (35, 44, -0.5),
    (35, 45, -1.0),
    (35, 46,  1.5),
    (35, 47,  0.0),

    (46, 47, -2.0),

    // right ring
    (12, 13, -2.0),
    (12, 22,  0.0),
    (12, 23,  1.5),
    (12, 24, -1.0),
    (12, 25,  1.0),
    (12, 26,  3.0),
    (12, 35,  1.0),
    (12, 36,  3.0),
    (12, 37,  1.0),
    (12, 46,  3.0),
    (12, 47,  5.0),

    (23, 24, -2.0),
    (23, 25, -1.0),
    (23, 26,  1.0),
    (23, 35, -1.0),
    (23, 36,  1.5),
    (23, 37,  0.0),
    (23, 46,  1.0),
    (23, 47,  3.0),

    (36, 37, -2.0),
    (36, 46, -1.0),
    (36, 47,  1.5),

    // right pinky
    (13, 23,  0.0),
    (13, 24,  1.5),
    (13, 25,  1.5),
    (13, 26,  3.0),
    (13, 36,  1.0),
    (13, 37,  3.0),
    (13, 47,  3.0),

    (24, 25,  2.0),
    (24, 26,  4.0),
    (24, 36, -1.0),
    (24, 37,  1.5),
    (24, 47,  1.0),

    (25, 26,  2.0),
    (25, 36,  1.0),
    (25, 37,  1.5),
    (25, 47,  2.0),

    (26, 36,  2.0),
    (26, 37,  3.0),
    (26, 47,  3.0),

    (37, 47, -1.0)
];


// Alpha blend a new pixel into an image buffer
fn blend_pixel(i: u32, j: u32, src_pixel: Rgba<u8>, ib: &mut ImageBuffer<Rgba<u8>,Vec<u8>>)
{    
    let (dst_r, dst_g, dst_b, dst_a) = {
        let dst_pixel = ib.get_pixel(i, j);
        dst_pixel.channels4()
    };
    let (src_r, src_g, src_b, src_a) = src_pixel.channels4();
    
    let src_alpha = (src_a as f32) / 255.0f32;
    
    let r = 255.0f32.min(src_alpha * src_r as f32 + (1.032 - src_alpha) * dst_r as f32) as u8;
    let g = 255.0f32.min(src_alpha * src_g as f32 + (1.032 - src_alpha) * dst_g as f32) as u8;
    let b = 255.0f32.min(src_alpha * src_b as f32 + (1.032 - src_alpha) * dst_b as f32) as u8;
    let a = 255.0f32.min(src_alpha * src_a as f32 + (1.032 - src_alpha) * dst_a as f32) as u8;

    ib.put_pixel(i, j, Rgba::from_channels(r, g, b, a));
}

// Alpha blend an entire bitmap into an image buffer
fn blend_bitmap(x: u32, y: u32, bitmap: &Bitmap, ib: &mut ImageBuffer<Rgba<u8>,Vec<u8>>)
{
    for j in 0u32..(bitmap.rows() as u32) {
        for i in 0u32..(bitmap.width() as u32) {
            let alpha = bitmap.buffer()[(bitmap.width() as u32 * j + i) as usize];
            if  alpha != 0 {
                blend_pixel(x+i, y+j, Rgba::from_channels(0, 0, 0, alpha), ib);
            }
        }
    }
}


// Object to load fonts for drawing letters on keys
struct KeyFaces
{
    key_size:     u32,
    key_padding:  u32,
    letter_face:  Face,
    symbol_face:  Face,
}

impl KeyFaces
{
    fn new(key_size: u32, key_padding: u32, letter_pt_size: isize, symbol_pt_size: isize)
        -> KeyFaces {
        let library = Library::init().unwrap();
        
        let lf = library.new_face(&Path::new("font.ttf"), 0).unwrap();
        lf.set_char_size(0, letter_pt_size * 64, 0, 0).unwrap();
        
        let sf = library.new_face(&Path::new("font.ttf"), 0).unwrap();
        sf.set_char_size(0, symbol_pt_size * 64, 0, 0).unwrap();
                
        KeyFaces{key_size: key_size, key_padding: key_padding, letter_face: lf, symbol_face: sf}
    }
    
    fn draw_key_cap(&self, x: u32, y: u32, c1: char, c2: char,
                    ib: &mut ImageBuffer<Rgba<u8>,Vec<u8>>) {
        if 'a' <= c1 && c1 <= 'z' {
            assert!('A' <= c2 && c2 <= 'Z', "Layout must match lower and upper case letters.");
            self.letter_face.load_char(c2 as usize, RENDER).unwrap();
            let glyph    = &self.letter_face.glyph();
            let bitmap   = &glyph.bitmap();
            let x_offset = (self.key_size - bitmap.width() as u32)/2;
            let y_offset = self.key_size/2 + (self.letter_face.ascender()/128) as u32
                         - (glyph.metrics().horiBearingY/64) as u32;
            blend_bitmap(x + x_offset, y + y_offset, bitmap, ib);
        } else {
            self.symbol_face.load_char(c1 as usize, RENDER).unwrap();
            let glyph1    = &self.symbol_face.glyph();
            let bitmap1   = &glyph1.bitmap();
            let x_offset1 = (self.key_size - bitmap1.width() as u32)/2;
            let y_offset1 = (self.key_size*16)/24 + (self.symbol_face.ascender()/128) as u32
                          - (glyph1.metrics().horiBearingY/64) as u32;
            blend_bitmap(x + x_offset1, y + y_offset1, bitmap1, ib);

            self.symbol_face.load_char(c2 as usize, RENDER).unwrap();
            let glyph2    = &self.symbol_face.glyph();
            let bitmap2   = &glyph2.bitmap();
            let x_offset2 = (self.key_size - bitmap2.width() as u32)/2;
            let y_offset2 = (self.key_size*6)/24 + (self.symbol_face.ascender()/128) as u32
                          - (glyph2.metrics().horiBearingY/64) as u32;
            blend_bitmap(x + x_offset2, y + y_offset2, bitmap2, ib);
        }
    }
    
    fn draw_key_score(&self, x: u32, y: u32, score: f32, ib: &mut ImageBuffer<Rgba<u8>,Vec<u8>>) {
        let s  = format!("{:3.1}", score);
        let ss = if &s[..] == "0.0" {
            ""
        } else {
            s[..].trim_right_matches('0').trim_right_matches('.')
        };
        let mut str_half_width = 0i64;
        for c in ss.chars() {
            self.letter_face.load_char(c as usize, RENDER).unwrap();
            str_half_width += self.letter_face.glyph().metrics().horiAdvance;
            if c == '.' || c == '-' {
                str_half_width -= self.letter_face.glyph().metrics().horiAdvance * 3 / 4;
            }
        }
        str_half_width /= 128i64;
        let mut xx = (x + self.key_size/2) as i64 - str_half_width;
        for c in ss[..].chars() {
            self.letter_face.load_char(c as usize, RENDER).unwrap();
            let glyph    = &self.letter_face.glyph();
            let bitmap   = &glyph.bitmap();
            let hbx      =  glyph.metrics().horiBearingX / 64;
            let y_offset =  self.key_size/2 + (self.letter_face.ascender()/128) as u32
                         - (glyph.metrics().horiBearingY/64) as u32;
            if c == '.' || c == '-' {
                xx -= self.letter_face.glyph().metrics().horiAdvance * 3 / 8 / 64;
            }
            blend_bitmap((xx + hbx) as u32, y + y_offset, bitmap, ib);
            if c == '.' || c == '-' {
                xx -= self.letter_face.glyph().metrics().horiAdvance * 3 / 8 / 64;
            }
            xx += glyph.metrics().horiAdvance / 64;
        }
    }
    
    fn draw_key_background(&self, x: u32, y: u32, w: u32, p: Rgba<u8>,
                           ib: &mut ImageBuffer<Rgba<u8>,Vec<u8>>)
    {
        let kp22 = self.key_padding * 2 + 2;
        assert!(w > kp22);
        assert!(self.key_size > kp22);
        let ww = w - kp22;
        let hh = self.key_size - kp22;
        for j in 0..hh {
            for i in 0..ww {
                ib.put_pixel(x + i + self.key_padding + 1, y + j + self.key_padding + 1, p);
            }
        }
    }
    
    fn draw_key_border(&self, x: u32, y: u32, w: u32, p: Rgba<u8>,
                       ib: &mut ImageBuffer<Rgba<u8>,Vec<u8>>) {
        let kp22 = self.key_padding * 2 + 2;
        assert!(w > kp22);
        assert!(self.key_size > kp22);
        let ww = w - kp22;
        let hh = self.key_size - kp22;
        for i in 0..ww {
            ib.put_pixel(x + i + self.key_padding+1, y + self.key_padding, p);
            ib.put_pixel(x + i + self.key_padding+1, y + self.key_size-1-self.key_padding,p);
        }
        for j in 0..hh {
            ib.put_pixel(x +     self.key_padding, y + j + self.key_padding+1, p);
            ib.put_pixel(x + w-1-self.key_padding, y + j + self.key_padding+1, p);
        }    
    }
}

// Check all the assumptions that make a byte array into a layout array
fn assert_valid_layout_string(s: &str)
{
    let mut layout = [0u8; 190];
    let l = layout.as_mut_slice();
    let mut ki = 1u8;
    for c in s.chars() {
        let ci = (c as u8) - 32;
        assert!(0 < ci && ci < 95, "Layout string has invalid character: {} -> {}", ci, c);
        l[ci as usize] = ki;
        l[(ki + 95) as usize] = ci;
        ki += 1;
    }
    assert!(ki == 95, "Layout string must have 94 characters.");
    assert!(l[0] == 0 && l[95] == 0, "Layout must assign 0 to the space key.");
    let mut occurrences = [0u8; 95];
    for i in 0..95 {
        let li = l[i] as usize;
        assert!(li < 95, format!("Layout contains the invalid key number {}.", li));
        occurrences[li] += 1;
    }
    for i in 0..95 {
        assert!(occurrences[i] == 1, format!("Layout isn't bijective at {}.", i));
    }
    for i in 0..95 {
        assert!(l[(l[i]+95) as usize] == i as u8,
            "Second half of layout is not the inverse of the first half.");
    }
    for i in 65u8..91 {
        assert!(l[i as usize] <= 47,
            format!("Lower-case letter {} must correspond to a lower-case key.", (i+32) as char));
    }
    for i in 33u8..59 {
        assert!(l[i as usize] > 47,
            format!("Upper-case letter {} must correspond to an upper-case key.", (i+32) as char));
    }
}

fn diagram_keyboard<D: FnMut(u32, u32, u32, usize)>(mut draw_key: D)
{    
    // Key sizes
    let ks0 = LARGE_KEY_SIZE;
    let ks1 = (ks0 * 3) / 2; // backspace and tab key sizes
    let ks2 = (ks0 * 7) / 4; // caps lock and enter key sizes
    let ks3 = (ks0 * 9) / 4; // shift key sizes
    
    // top row
    for i in 0..13 {
        draw_key(i * ks0, 0, ks0, i as usize);
    }
    draw_key(13 * ks0, 0, ks1, 47); // backspace key
        
    // second row down
    draw_key(0, ks0, ks1, 48); // tab key
    for i in 0..13 {
        draw_key(ks1 + i * ks0, ks0, ks0, i as usize + 13);
    }

    // third row down
    draw_key(0, 2 * ks0, ks2, 49); // caps lock key
    for i in 0..11 {
        draw_key(ks2 + i * ks0, 2 * ks0, ks0, i as usize + 26);
    }
    draw_key(ks2 + 11 * ks0, 2 * ks0, ks2, 50); // enter key
        
    // fourth row down
    draw_key(0, 3 * ks0, ks3, 51); // left shift
    for i in 0..10 {
        draw_key(ks3 + i * ks0, 3 * ks0, ks0, i as usize + 37);
    }
    draw_key(ks3 + 10 * ks0, 3 * ks0, ks3, 52); // right shift
}

// Draw a keyboard layout diagram from a txt file
fn diagram_layout(layout: &str, file_name: &str)
{    
    assert_valid_layout_string(layout);
    let kf = KeyFaces::new(LARGE_KEY_SIZE, LARGE_KEY_PADDING, LETTERS_PT_SIZE, SYMBOLS_PT_SIZE);
    let ks = LARGE_KEY_SIZE;
    let bc = Rgba::from_channels(0, 0, 0, 255); // border color: black

    let mut ib = ImageBuffer::new(13 * ks + (3 * ks) / 2, 4 * ks);
    diagram_keyboard(|x: u32, y: u32, w: u32, li: usize| {
        if li < 47 {
            kf.draw_key_cap(x, y, layout[..].char_at(li), layout[..].char_at(li+47), &mut ib);
        }
        kf.draw_key_border(x, y, w, bc, &mut ib);
    });
        
    let output_file_name = "diagram-".to_string() + file_name + ".png";
    let mut output_file  = std::old_io::fs::File::create(&Path::new(output_file_name)).unwrap();
    let _ = image::ImageRgba8(ib).save(&mut output_file, image::PNG);
}


// Draw a keyboard layout diagram from a txt file
//fn diagram_layout(layout: &str, file_name: &str)
//{    
//    // verify input
//    assert_valid_layout_string(layout);
//
//    // Load the font
//    let kf = KeyFaces::new(LARGE_KEY_SIZE, LARGE_KEY_PADDING, LETTERS_PT_SIZE, SYMBOLS_PT_SIZE);
//    
//    // Key sizes
//    let ks0 = LARGE_KEY_SIZE;
//    let ks1 = (ks0 * 3) / 2; // backspace and tab key sizes
//    let ks2 = (ks0 * 7) / 4; // caps lock and enter key sizes
//    let ks3 = (ks0 * 9) / 4; // shift key sizes
//    
//    // Draw the layout using the font
//    let w = 13 * ks0 + (3 * ks0) / 2;
//    let h =  4 * ks0;
//    let mut ib = ImageBuffer::new(w, h);
//    {
//        let mut li = 0; // layout char index
//        let     ls = &layout[..];
//        let     bc = Rgba::from_channels(0, 0, 0, 255); // border color: black
//        
//        // top row
//        for i in 0..13 {
//            let x = i * ks0;
//            let y = 0;
//            kf.draw_key_cap(x, y, ls.char_at(li), ls.char_at(li+47), &mut ib);
//            kf.draw_key_border(x, y, ks0, bc, &mut ib);
//            li += 1; 
//        }
//        kf.draw_key_border(13 * ks0, 0, ks1, bc, &mut ib); // backspace key
//        
//        // second row down
//        kf.draw_key_border(0, ks0, ks1, bc, &mut ib); // tab key
//        for i in 0..13 {
//            let x = ks1 + i * ks0;
//            let y = ks0;
//            kf.draw_key_cap(x, y, ls.char_at(li), ls.char_at(li+47), &mut ib);
//            kf.draw_key_border(x, y, ks0, bc, &mut ib);
//            li += 1;
//        }
//
//        // third row down
//        kf.draw_key_border(0, 2 * ks0, ks2, bc, &mut ib); // caps lock key
//        for i in 0..11 {
//            let x = ks2 + i * ks0;
//            let y = 2 * ks0;
//            kf.draw_key_cap(x, y, ls.char_at(li), ls.char_at(li+47), &mut ib);
//            kf.draw_key_border(x, y, ks0, bc, &mut ib);
//            li += 1;
//        }
//        kf.draw_key_border(ks2 + 11 * ks0, 2 * ks0, ks2, bc, &mut ib); // enter key
//        
//        // fourth row down
//        kf.draw_key_border(0, 3 * ks0, ks3, bc, &mut ib); // left shift
//        for i in 0..10 {
//            let x = ks3 + i * ks0;
//            let y = 3 * ks0;
//            kf.draw_key_cap(x, y, ls.char_at(li), ls.char_at(li+47), &mut ib);
//            kf.draw_key_border(x, y, ks0, bc, &mut ib);
//            li += 1;
//        }
//        kf.draw_key_border(ks3 + 10 * ks0, 3 * ks0, ks3, bc, &mut ib); // right shift
//    }    
//    
//    // Output the resulting image as a png
//    let output_file_name = "diagram-".to_string() + file_name + ".png";
//    let mut output_file      = std::old_io::fs::File::create(&Path::new(output_file_name)).unwrap();
//    let _ = image::ImageRgba8(ib).save(&mut output_file, image::PNG);
//}

// Draw a diagram of keyboard layout penalties
fn diagram_finger_assignments(print_key_numbers: bool)
{
    // Load the key faces
    let kf = KeyFaces::new(LARGE_KEY_SIZE, LARGE_KEY_PADDING, LETTERS_PT_SIZE, SYMBOLS_PT_SIZE);
    
    // Key sizes
    let ks0 = LARGE_KEY_SIZE;
    let ks1 = (ks0 * 3) / 2; // backspace and tab key sizes
    let ks2 = (ks0 * 7) / 4; // caps lock and enter key sizes
    let ks3 = (ks0 * 9) / 4; // shift key sizes
    
    let w  = 13 * ks0 + (3 * ks0) / 2;
    let h  =  4 * ks0;
    let mut ib = ImageBuffer::new(w, h);
    {
        let bc = Rgba::from_channels(0, 0, 0, 255); // border color: black
        
        // top row
        for i in 0..13 {
            let x = i * ks0;
            let y = 0;
            let f = FINGER_ASSIGNMENT[i as usize + 1];
            let (r, g, b, a) = FINGER_COLORS[f];
            kf.draw_key_border(x, y, ks0, bc, &mut ib);
            kf.draw_key_background(x, y, ks0, Rgba::from_channels(r, g, b, a), &mut ib);
            if print_key_numbers {
                kf.draw_key_score(x, y, (i+1)as f32, &mut ib);
            }
        }
        kf.draw_key_border(13 * ks0, 0, ks1, bc, &mut ib); // backspace key
        
        // second row down
        kf.draw_key_border(0, ks0, ks1, bc, &mut ib); // tab key
        for i in 0..13 {
            let x = ks1 + i * ks0;
            let y = ks0;
            let f = FINGER_ASSIGNMENT[i as usize + 14];
            let (r, g, b, a) = FINGER_COLORS[f];
            kf.draw_key_border(x, y, ks0, bc, &mut ib);
            kf.draw_key_background(x, y, ks0, Rgba::from_channels(r, g, b, a), &mut ib);
            if print_key_numbers {
                kf.draw_key_score(x, y, (i+14) as f32, &mut ib);
            }
        }

        // third row down
        kf.draw_key_border(0, 2 * ks0, ks2, bc, &mut ib); // caps lock key
        for i in 0..11 {
            let x = ks2 + i * ks0;
            let y = 2 * ks0;
            let f = FINGER_ASSIGNMENT[i as usize + 27];
            let (r, g, b, a) = FINGER_COLORS[f];
            kf.draw_key_border(x, y, ks0, bc, &mut ib);
            kf.draw_key_background(x, y, ks0, Rgba::from_channels(r, g, b, a), &mut ib);
            if print_key_numbers {
                kf.draw_key_score(x, y, (i+27) as f32, &mut ib);
            }
        }
        kf.draw_key_border(ks2 + 11 * ks0, 2 * ks0, ks2, bc, &mut ib); // enter key
        
        // fourth row down
        kf.draw_key_border(0, 3 * ks0, ks3, bc, &mut ib); // left shift
        for i in 0..10 {
            let x = ks3 + i * ks0;
            let y = 3 * ks0;
            let f = FINGER_ASSIGNMENT[i as usize+ 38];
            let (r, g, b, a) = FINGER_COLORS[f];
            kf.draw_key_border(x, y, ks0, bc, &mut ib);
            kf.draw_key_background(x, y, ks0, Rgba::from_channels(r, g, b, a), &mut ib);
            if print_key_numbers {
                kf.draw_key_score(x, y, (i+38) as f32, &mut ib);
            }
        }
        kf.draw_key_border(ks3 + 10 * ks0, 3 * ks0, ks3, bc, &mut ib); // right shift
    }    
    
    // Output the resulting image as a png
    let output_filename = if print_key_numbers {
        "diagram-finger_assignments_numbered.png"
    } else {
        "diagram-finger_assignments.png"
    };
    let mut output_file = std::old_io::fs::File::create(&Path::new(output_filename)).unwrap();
    let _ = image::ImageRgba8(ib).save(&mut output_file, image::PNG);
}


// Color a pixel based on a scalar value between 0 and 1, with clamping      
fn intensity(x: f32) -> Rgba<u8>
{
    let r0 = 0.0;
    let g0 = 1.0;
    let b0 = 0.0;
    
    let r1 = 1.0;
    let g1 = 1.0;
    let b1 = 0.0;
    
    let r2 = 1.0;
    let g2 = 0.0;
    let b2 = 0.0;
    
    let t  = if x >= 1.0 { 1.0 } else if x <= -1.0 { -1.0 } else { x };
    let (r, g, b, a) = if t < 0.0 {
        (-t*r0 + (1.0+t)*r1, -t*g0 + (1.0+t)*g1, -t*b0 + (1.0+t)*b1, 0.15 - t*0.85)
    } else {
        ( (1.0-t)*r1 + t*r2,  (1.0-t)*g1 + t*g2,  (1.0-t)*b1 + t*b2, 0.15 + t*0.85)
    };
    
    Rgba::from_channels((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, (a * 255.0) as u8)
}

// Draw a diagram of keyboard layout penalties for single keys
fn diagram_single_metric(metric: &[f32; 48], file_name: &str)
{
    // Load the font
    let kf = KeyFaces::new(LARGE_KEY_SIZE, LARGE_KEY_PADDING, LETTERS_PT_SIZE, SYMBOLS_PT_SIZE);
    
    // Find the minimum value of the single metric
    let min          = metric.iter().fold(std::f32::INFINITY,     |min, &x| min.min(x));
    let max          = metric.iter().fold(std::f32::NEG_INFINITY, |max, &x| max.max(x));
    let weighted_ave = 0.2 * max + 0.8 * min;
    let scale        = 1.0f32 / (max - weighted_ave - 4.0);
    let offset       = -weighted_ave;
    
    // Key sizes
    let ks0 = LARGE_KEY_SIZE;
    let ks1 = (ks0 * 3) / 2; // backspace and tab key sizes
    let ks2 = (ks0 * 7) / 4; // caps lock and enter key sizes
    let ks3 = (ks0 * 9) / 4; // shift key sizes
    
    // Draw the diagram using the font
    let w  = 13 * ks0 + (3 * ks0) / 2;
    let h  =  4 * ks0;
    let mut ib = ImageBuffer::new(w, h);
    {
        let bc = Rgba::from_channels(0, 0, 0, 255); // border color: black
                
        // top row
        for i in 0..13 {
            let x = i * ks0;
            let y = 0;
            let m = metric[i as usize + 1];
            kf.draw_key_border(x, y, ks0, bc, &mut ib);
            if metric[1] > 0.0 {
                kf.draw_key_background(x, y, ks0, intensity((m + offset) * scale), &mut ib);
                kf.draw_key_score(x, y, m + 2f32, &mut ib);
            }
        }
        kf.draw_key_border(13 * ks0, 0, ks1, bc, &mut ib); // backspace key
        
        // second row down
        kf.draw_key_border(0, ks0, ks1, bc, &mut ib); // tab key
        for i in 0..13 {
            let x = ks1 + i * ks0;
            let y = ks0;
            let m = metric[i as usize + 14];
            kf.draw_key_border(x, y, ks0, bc, &mut ib);
            if m != 0.01 {
                kf.draw_key_background(x, y, ks0, intensity((m + offset) * scale), &mut ib);
                kf.draw_key_score(x, y, m + 2f32, &mut ib);
            }
        }

        // third row down
        kf.draw_key_border(0, 2 * ks0, ks2, bc, &mut ib); // caps lock key
        for i in 0..11 {
            let x = ks2 + i * ks0;
            let y = 2 * ks0;
            let m = metric[i as usize + 27];
            kf.draw_key_border(x, y, ks0, bc, &mut ib);
            if m != 0.01 {
                kf.draw_key_background(x, y, ks0, intensity((m + offset) * scale), &mut ib);
                kf.draw_key_score(x, y, m + 2f32, &mut ib);
            }
        }
        kf.draw_key_border(ks2 + 11 * ks0, 2 * ks0, ks2, bc, &mut ib); // enter key
        
        // fourth row down
        kf.draw_key_border(0, 3 * ks0, ks3, bc, &mut ib); // left shift
        for i in 0..10 {
            let x = ks3 + i * ks0;
            let y = 3 * ks0;
            let m = metric[i as usize + 38];
            kf.draw_key_border(x, y, ks0, bc, &mut ib);
            kf.draw_key_background(x, y, ks0, intensity((m + offset) * scale), &mut ib);
            kf.draw_key_score(x, y, m + 2f32, &mut ib);
        }
        kf.draw_key_border(ks3 + 10 * ks0, 3 * ks0, ks3, bc, &mut ib); // right shift
    }    
    
    // Output the resulting image as a png
    let output_file_name = "diagram-".to_string() + file_name + ".png";
    let mut output_file = std::old_io::fs::File::create(&Path::new(output_file_name)).unwrap();
    let _ = image::ImageRgba8(ib).save(&mut output_file, image::PNG);
}

// Draw a diagram of keyboard layout scores for pairs of keys
fn diagram_double_metric()
{
    let padding:        u32 = 4;
    let columns:        u32 = 2;
    let finger_color        = Rgba::from_channels(0x64, 0x95, 0xED, 0xFF); // cornflower blue
    let score_y_offset: u32 = 5;
    let border_color_0      = Rgba::from_channels(  0,   0,   0, 255); // black
    let border_color_1      = Rgba::from_channels(196, 196, 196, 255); // grey
    
    let rows = 47u32 / columns + if 47u32 % columns > 0 { 1 } else { 0 };
    
    // Load the font
    let kf = KeyFaces::new(DOUBLE_METRIC_KEY_SIZE, DOUBLE_METRIC_KEY_PADDING,
                           DOUBLE_METRIC_PT_SIZE,  DOUBLE_METRIC_PT_SIZE);
    
    //Find the minimum value of the double key metric
    let min = DOUBLE_METRIC.iter().fold(std::f32::INFINITY,     |min, &x| min.min(x.2));
    let max = DOUBLE_METRIC.iter().fold(std::f32::NEG_INFINITY, |max, &x| max.max(x.2));
    let scale  = 1.0 / (max - min - 4.0);
    let offset = 0.0f32;
    
    // Key sizes
    let ks0 = DOUBLE_METRIC_KEY_SIZE;
    let ks1 = (ks0 * 3) / 2; // backspace and tab key sizes
    let ks2 = (ks0 * 7) / 4; // caps lock and enter key sizes
    let ks3 = (ks0 * 9) / 4; // shift key sizes
    
    // Draw the diagram using the font
    let lw = 13 * ks0 + (3 * ks0) / 2;
    let lh =  4 * ks0;
    let w  = lw * columns + padding * (columns - 1);
    let h  = lh * rows    + padding * (rows    - 1);
    let mut ib = ImageBuffer::new(w, h);
    
    for key in 1..48 {
        
        let c  = (key - 1) as u32 % columns;
        let r  = (key - 1) as u32 / columns;
        let x0 = lw * c + padding * c;
        let y0 = lh * r + padding * r;
        
        // make scoring array for this key
        let mut score = [0f32; 48];
        for i in 0..314 {
            let (k1, k2, s) = DOUBLE_METRIC[i];
            if k1 == key {
                score[k2 as usize] += s;
            }
            if k2 == key {
                score[k1 as usize] += s;
            }
        }
                        
        // top row
        for i in 1..14 {
            let x = x0 + (i - 1)*ks0;
            let y = y0;
            let s = score[i as usize];            
            let b = if FINGER_ASSIGNMENT[i as usize] % 2 == 0 {
                border_color_0
            } else {
                border_color_1
            };
            
            kf.draw_key_border(x, y, ks0, b, &mut ib);
            kf.draw_key_background(x, y, ks0, intensity((s + offset) * scale), &mut ib);
            kf.draw_key_score(x, y - score_y_offset, s, &mut ib);
            if i == key as u32 {
                kf.draw_key_background(x, y, ks0, finger_color, &mut ib);
            }
        }
        kf.draw_key_border(x0 + 13*ks0, y0, ks1, border_color_0, &mut ib); // backspace key
        
        // second row down
        kf.draw_key_border(x0, y0 + ks0, ks1, border_color_1, &mut ib); // tab key
        for i in 14..27 {
            let x = x0 + ks1 + (i - 14)*ks0;
            let y = y0 + ks0;
            let s = score[i as usize];
            let b = if FINGER_ASSIGNMENT[i as usize] % 2 == 0 {
                border_color_0
            } else {
                border_color_1
            };
            
            kf.draw_key_border(x, y, ks0, b, &mut ib);
            kf.draw_key_background(x, y, ks0, intensity((s + offset) * scale), &mut ib);
            kf.draw_key_score(x, y - score_y_offset, s, &mut ib);
            if i == key as u32 {
                kf.draw_key_background(x, y, ks0, finger_color, &mut ib);
            }
        }

        // third row down
        kf.draw_key_border(x0, y0 + 2*ks0, ks2, border_color_1, &mut ib); // caps lock key
        for i in 27..38 {
            let x = x0 + ks2 + (i - 27)*ks0;
            let y = y0 + 2*ks0;
            let s = score[i as usize];
            let b = if FINGER_ASSIGNMENT[i as usize] % 2 == 0 {
                border_color_0
            } else {
                border_color_1
            };
            
            kf.draw_key_border(x, y, ks0, b, &mut ib);
            kf.draw_key_background(x, y, ks0, intensity((s + offset) * scale), &mut ib);
            kf.draw_key_score(x, y - score_y_offset, s, &mut ib);
            if i == key as u32 {
                kf.draw_key_background(x, y, ks0, finger_color, &mut ib);
            }
        }
        // enter key
        kf.draw_key_border(x0 + ks2 + 11*ks0, y0 + 2*ks0, ks2, border_color_0, &mut ib);
        
        // fourth row down
        kf.draw_key_border(x0, y0 + 3*ks0, ks3, border_color_1, &mut ib); // left shift
        for i in 38..48 {
            let x = x0 + ks3 + (i - 38)*ks0;
            let y = y0 + 3*ks0;
            let s = score[i as usize];                        
            let b = if FINGER_ASSIGNMENT[i as usize] % 2 == 0 {
                border_color_0
            } else {
                border_color_1
            };
            
            kf.draw_key_border(x, y, ks0, b, &mut ib);
            kf.draw_key_background(x, y, ks0, intensity((s + offset) * scale), &mut ib);
            kf.draw_key_score(x, y - score_y_offset, s, &mut ib);
            if i == key as u32 {
                kf.draw_key_background(x, y, ks0, finger_color, &mut ib);
            }
        }
        // right shift
        kf.draw_key_border(x0 + ks3 + 10*ks0, y0 + 3*ks0, ks3, border_color_0, &mut ib);
    }    
    
    // Output the resulting image as a png
    let output_path = Path::new("diagram-double_metric.png");
    let mut output_file = std::old_io::fs::File::create(&output_path).unwrap();
    let _ = image::ImageRgba8(ib).save(&mut output_file, image::PNG);
}


fn main()
{
    diagram_layout( QWERTY_STRING,  "qwerty_layout");
    diagram_layout( DVORAK_STRING,  "dvorak_layout");
    diagram_layout(COLEMAK_STRING, "colemak_layout");
    diagram_layout(WORKMAN_STRING, "workman_layout");
    diagram_layout(PROTO_1_STRING, "proto_1_layout");
    diagram_layout(  WHITE_STRING,   "white_layout");
    
    diagram_finger_assignments(false);
    diagram_finger_assignments(true);
    
    diagram_single_metric( &SINGLE_METRIC,  "single_metric");
    diagram_single_metric(&WORKMAN_METRIC, "workman_metric");
    
    diagram_double_metric();
}

