// EXPLANATION OF THE KEYBOARD LAYOUT REPRESENTATION

// A "layout" will be defined as a bijective mapping from 95 specific ASCII characters to some
// combinations of the physical keyboard keys being pressed. It will be represented in memory as an
// array of 190 bytes. The first 95 bytes in the array encode the mapping from ASCII characters to
// keys, and the second 95 bytes encode the mapping from keys to ASCII characters.

// Since we do not care about the first 32 characters of the ASCII specification, we will subtract
// 32 from each ASCII character code to get its representation. The representation for the keys on
// the keyboard is created first by assigning 0 to the space bar. Then we assign the numbers 1-47
// to the 47 remaining keys starting with the left-most key of the second row from the top (which
// is the ` (backtick) key for QWERTY) and moving right across that row, followed by moving down a
// row, doing that row from left to right, etc, while skipping the delete, tab, caps lock, enter,
// and shift keys. Finally we assign the numbers 48-94 to the same keys in the same order as we did
// for 1-47, but these key presses now also include holding down a shift button. The reason for
// this ordering is so a layout may be specified using a string that looks like the layout itself.

// As an example, given a layout array l, in order to find out which key corresponds to the letter
// 'a', we take its ASCII code 97, subtract 32 to get 65, and then l[65] will give us back the
// number of the key, which is 27 in QWERTY. Likewise, if we want to find out which ASCII code
// corresponds to pressing key number 27 in QWERTY, then l[95+27] will give us back the number 65,
// which when added to 32 gives us the ASCII code (97) for the character 'a'.


// TABLE OF THE 95 ASCII CHARACTER CODES

// 32 space  48 0      64 @      80 P      96 `     112 p
// 33 !      49 1      65 A      81 Q      97 a     113 q
// 34 "      50 2      66 B      82 R      98 b     114 r
// 35 #      51 3      67 C      83 S      99 c     115 s
// 36 $      52 4      68 D      84 T     100 d     116 t
// 37 %      53 5      69 E      85 U     101 e     117 u
// 38 &      54 6      70 F      86 V     102 f     118 v
// 39 '      55 7      71 G      87 W     103 g     119 w
// 40 (      56 8      72 H      88 X     104 h     120 x
// 41 )      57 9      73 I      89 Y     105 i     121 y
// 42 *      58 :      74 J      90 Z     106 j     122 z
// 43 +      59 ;      75 K      91 [     107 k     123 {
// 44 ,      60 <      76 L      92 \     108 l     124 |
// 45 -      61 =      77 M      93 ]     109 m     125 }
// 46 .      62 >      78 N      94 ^     110 n     126 ~
// 47 /      63 ?      79 O      95 _     111 o

extern crate rand;
use std::ops::Add;
use std::fs::File;
use std::path::Path;
use std::io::{Read, Write};
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Vacant, Occupied};


// CUSTOMIZABLE OPTIMIZATION PARAMETERS
const PRINT_OBJECTIVE_FUNCTION:  bool  =  false;

const FROZEN_SYMBOLS:   &'static str   = "0123456789";

const SINGLE_METRIC_COEFFICIENT: f32   =  1.00;
const DOUBLE_METRIC_COEFFICIENT: f32   =  1.00;
const TRIPLE_METRIC_COEFFICIENT: f32   =  1.00;
const SHIFT_HOLDING_PENALTY:     f32   =  1.50;
const REVERSED_TRIPLE_PENALTY:   f32   =  0.25;
const HAND_ALTERNATION_PENALTY:  f32   =  0.20;

const CORPUS_MIN_WORD_FREQUENCY: f32   =  20.0;
const CORPUS_1_COEFFICIENT:      f64   =  1e-3;
const CORPUS_2_COEFFICIENT:      f64   =  0.50;

const CYCLE_TEMPERATURE_START:   f64   =  1e+5;
const CYCLE_TEMPERATURE_FINAL:   f64   =  5e+3;
const CYCLE_TEMPERATURE_FACTOR:  f64   =  0.50;
const TEMPERATURE_FINAL:         f64   =  1.00;
const TEMPERATURE_FACTOR:        f64   =  0.99999;
const NUM_TABU_SWAPS:            usize =  10;


// LAYOUT STRINGS
const _QWERTY_STRING: &'static str = "
 `1234567890-=
  qwertyuiop[]\\
   asdfghjkl;'
    zxcvbnm,./
 ~!@#$%^&*()_+
  QWERTYUIOP{}|
   ASDFGHJKL:\"
    ZXCVBNM<>?";

const _DVORAK_STRING: &'static str = "
`1234567890[]
  ',.pyfgcrl/=\\
   aoeuidhtns-
    ;qjkxbmwvz
~!@#$%^&*(){}
  \"<>PYFGCRL?+|
   AOEUIDHTNS_
    :QJKXBMWVZ";

const _COLEMAK_STRING: &'static str = "
`1234567890-=
  qwfpgjluy;[]\\
   arstdhneio'
    zxcvbkm,./
~!@#$%^&*()_+
  QWFPGJLUY:{}|
   ARSTDHNEIO\"
    ZXCVBKM<>?";

const _WORKMAN_STRING: &'static str = "
`1234567890-=
  qdrwbjfup;[]\\
   ashtgyneoi'
    zxmcvkl,./
~!@#$%^&*()_+
  QDRWBJFUP:{}|
   ASHTGYNEOI\"
    ZXMCVKL<>?";

const WHITE_STRING: &'static str = "
#12345@$67890
  vyd,'_jmlu()=
   atheb-csnoi
    pkgwqxrf.z
`!<>/|~%\\*[]^
  VYD;\"&JMLU{}?
   ATHEB+CSNOI
    PKGWQXRF:Z";

// Which fingers correspond to which keys
const FINGER_ASSIGNMENT: [u8; 48] = [ 0,
1,  1,  2,  3,  3,  4,  4,  5,  5,  6,  6,  7,  8,
      1,  2,  3,  4,  4,  4,  5,  5,  6,  7,  8,  8,  8,
        1,  2,  3,  4,  4,  5,  5,  5,  6,  7,  8,
          2,  3,  4,  4,  4,  5,  5,  5,  6,  7];

// Which keys the fingers start on
const HOME_EIGHT:[u8; 8] = [27, 28, 29, 30, 34, 35, 36, 37];

// Penalties for using each key
const SINGLE_METRIC: [f32; 48] = [ 0.0,
9.0,  7.0,  4.5,  3.5,  3.5,  6.0,  8.0,  9.5,  6.5,  3.5,  3.5,  4.5,  7.0,
         2.5,  0.1, -0.2,  1.0,  2.0,  5.0,  2.5,  1.0, -0.2,  0.1,  2.5,  3.0,  5.0,
           -0.5, -0.9, -1.2, -1.0,  1.0,  4.5,  1.0, -1.0, -1.2, -0.9, -0.5,
               2.0,  2.0,  0.5,  0.0,  3.0,  3.0,  0.0,  0.5,  2.0,  2.0];

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

const TRIPLE_METRIC: [(u8, u8, u8, f32); 52] = [
    // left pinky
    ( 1,  3,  4, -0.5),
    ( 2,  3,  4, -1.0),
    ( 2,  3,  5, -0.5),
    (14,  3,  4, -1.5),
    (14,  3,  5, -1.0),
    (14, 15, 16, -2.0),
    (27, 15, 16, -1.5),
    (27, 28, 29, -2.0),
    (27, 28, 30, -0.5),
    (27, 29, 30, -0.5),

    // left ring
    ( 3,  4,  6, -1.0),
    ( 3,  5,  6, -1.0),
    ( 3,  4, 17, -1.5),
    ( 3,  5, 17, -1.0),
    (15, 16, 17, -2.0),
    (15, 16, 18, -1.0),
    (15, 16, 30, -1.5),
    (28, 29, 30, -2.5),
    (28, 29, 31, -1.0),
    (28, 29, 40, -1.5),
    (28, 29, 41, -1.5),
    (28, 16, 30, -1.0),
    (28, 16, 17, -0.5),
    (38, 39, 40, -2.0),
    (38, 39, 41, -0.5),

    // right ring
    (12, 11,  9, -1.0),
    (12, 10,  9, -1.0),
    (12, 11, 21, -1.5),
    (12, 10, 21, -1.0),
    (23, 22, 21, -2.0),
    (23, 22, 20, -1.0),
    (23, 22, 34, -1.5),
    (36, 35, 34, -2.5),
    (36, 35, 33, -1.0),
    (36, 35, 45, -1.5),
    (36, 35, 44, -1.5),
    (36, 22, 34, -1.0),
    (36, 22, 21, -0.5),
    (47, 46, 45, -2.0),
    (47, 46, 44, -0.5),

    // right pinky
    (13, 12, 11, -1.0),
    (13, 12, 10, -0.5),
    (24, 12, 11, -1.5),
    (24, 12, 10, -1.0),
    (24, 23, 22, -2.0),
    (25, 12, 11, -1.5),
    (25, 12, 10, -0.5),
    (25, 23, 22, -1.0),
    (37, 36, 35, -2.0),
    (37, 23, 22, -1.5),
    (37, 35, 34, -0.5),
    (37, 36, 34, -0.5),
];


// LAYOUT FUNCTIONS

// Check all the assumptions that make a byte array into a layout array
fn assert_valid_layout(l: &[u8; 190])
{
    assert!(l[0] == 0 && l[95] == 0, "Layout must assign 0 to the space key.");
    let mut occurrences = [0u8; 95];
    for k in 0..95 {
        let li = l[k+95] as usize;
        let c = (li as u8 + 32) as char;
        assert!(li < 95, "Layout contains the invalid character '{}'.", c);
        occurrences[li] += 1;
        if li == 0 && occurrences[li] != 1 {
            break;
        } else {
            assert!(occurrences[li] == 1,
                "Layout assigns multiple keys to character '{}'", c);
        }
    }
    for i in 0..95 {
        assert!(occurrences[i as usize] >= 1,
            "Layout doesn't assign any keys to character '{}'.", (i as u8 + 32) as char);
    }
    for i in 0..95 {
        assert!(l[(l[i]+95) as usize] == i as u8,
            "Second half of layout is not the inverse of the first half.");
    }
    for i in 65u8..91 {
        assert!(l[i as usize] <= 47,
            "Lower-case letter {} must correspond to a lower-case key.", (i+32) as char);
    }
    for i in 33u8..59 {
        assert!(l[i as usize] > 47,
            "Upper-case letter {} must correspond to an upper-case key.", (i+32) as char);
    }
}

// Check whether a given string has only the allowed subset of ASCII characters.
fn check_valid_ascii_subset(s: &str) -> Result<(), (char, usize)>
{
    let mut line_num = 1usize;
    for line in s.lines() {
        for c in line.chars() {
            if !((' ' <= c && c <= '~') || c == '\n' || c == '\r' || c == '\t') {
                return Err((c, line_num));
            }
        }
        line_num += 1;
    }
    Ok(())
}

// Create a layout array from a string in the correct format.
fn layout_from_string(s_with_whitespace: &str) -> [u8; 190]
{
    let s_string = s_with_whitespace.chars().filter(|x| !x.is_whitespace()).collect::<String>();
    assert!(s_string.len() == 94, "Layout string is {} characters, not 94.", s_string.len());
    let s = &s_string; // s without whitespace
    if let Err((c,_)) = check_valid_ascii_subset(s) {
        panic!("Invalid character in layout: {} -> {}", c as u32, c);
    }
    let mut layout = [0u8; 190];
    let mut ki = 1u8;
    for c in s.chars() {
        let ci = (c as u8) - 32;
        assert!(ci > 0u8 && ci < 95u8, "Layout string has invalid character: {}", c);
        layout[ci as usize] = ki;
        layout[(ki + 95) as usize] = ci;
        ki += 1;
    }
    assert_valid_layout(&layout);
    layout
}

#[test]
fn qwerty_valid()
{
    let l = layout_from_string(_QWERTY_STRING);
    assert_valid_layout(&l);
}

#[test]
fn dvorak_valid()
{
    let l = layout_from_string(_DVORAK_STRING);
    assert_valid_layout(&l);
}

#[test]
fn colemak_valid()
{
    let l = layout_from_string(_COLEMAK_STRING);
    assert_valid_layout(&l);
}

#[test]
fn workman_valid()
{
    let l = layout_from_string(_WORKMAN_STRING);
    assert_valid_layout(&l);
}

#[test]
fn initial_layout_valid()
{
    let l = layout_from_string(INITIAL_LAYOUT_STRING);
    assert_valid_layout(&l);
}

// Convert a layout to a string and write it to a text file.
fn write_layout_file(layout: &[u8], filename: &str)
{
    let mut output = [0u8; 94];
    for i in 1u8..95 {
        output[(layout[i as usize] - 1) as usize] = i + 32;
    }
    let path = Path::new(filename);
    let mut file = std::fs::File::create(&path).unwrap();
    let io_result = file.write_all(&output);
    assert!(io_result.is_ok());
}

// Read a string from a file and convert it to a layout. If the file doesn't exist, use default.
fn read_layout_file(filename: &str) -> [u8; 190]
{
    let path = Path::new(filename);
    if std::fs::metadata(path).is_ok() {
        let mut file = File::open(&path).unwrap();
        let mut text = String::new();
        file.read_to_string(&mut text).unwrap();
        let layout = layout_from_string(&text[..]);
        layout
    } else {
        let layout = layout_from_string(WHITE_STRING);
        layout
    }
}


// TERMINAL OUTPUT FUNCTIONS

// Print out a key score in the terminal in colors to distinguish finger assignments.
fn print_key_score(k: usize, k0: u8, score: f32)
{
    let i = if k < 48 { k } else { k - 47 };
    let finger_color_string = match FINGER_ASSIGNMENT[i] {
        1 => "\x1B[41m", // red
        2 => "\x1B[42m", // green
        3 => "\x1B[45m", // magenta
        4 => "\x1B[46m", // cyan
        5 => "\x1B[44m", // blue
        6 => "\x1B[45m", // magenta
        7 => "\x1B[42m", // green
        8 => "\x1B[41m", // red
        _ => ""
    };
    print!("{}", finger_color_string);
    if k == k0 as usize {
        print!("\x1B[30m[\x1B[0m{}({:2})\x1B[30m]", finger_color_string, k);
    } else if score != 0f32 {
        print!("\x1B[30m[\x1B[0m{}{:4.1}\x1B[30m]", finger_color_string, score);
    } else {
        print!("\x1B[30m[    ]");
    }
    print!("\x1B[0m");
}

// Print a double key metric score diagram which shows the penalty from moving from that key (in
// parentheses) to other keys [in brackets]. Blank space means the penalty for that key is 0.
fn print_double_metric(key: u8)
{
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
    for i in 1..14 {
        print_key_score(i, key, score[i]);
    }
    print!("\n        ");
    for i in 14..27 {
        print_key_score(i, key, score[i]);
    }
    print!("\n          ");
    for i in 27..38 {
        print_key_score(i, key, score[i]);
    }
    print!("\n             ");
    for i in 38..48 {
        print_key_score(i, key, score[i]);
    }
    print!("\n\n");
}

// Print out a diagram for the single key scores static array.
fn print_single_metric()
{
    for i in 1..14 {
        print_key_score(i, 0, SINGLE_METRIC[i]);
    }
    print!("\n        ");
    for i in 14..27 {
        print_key_score(i, 0, SINGLE_METRIC[i]);
    }
    print!("\n          ");
    for i in 27..38 {
        print_key_score(i, 0, SINGLE_METRIC[i]);
    }
    print!("\n             ");
    for i in 38..48 {
        print_key_score(i, 0, SINGLE_METRIC[i]);
    }
    print!("\n\n");
}

// Print a layout array in a visually useful way.
fn print_layout(l: &[u8; 190])
{
    assert_valid_layout(l);
    let lower_case = &l[95..190];
    let upper_case = &l[95+47..190];

    for i in lower_case[1..14].iter() {
        print!("{} ", (i + 32) as char);
    }
    print!("     ");
    for i in upper_case[1..14].iter() {
        print!("{} ", (i + 32) as char);
    }
    print!("\n   ");
    for i in lower_case[14..27].iter() {
        print!("{} ", (i + 32) as char);
    }
    print!("     ");
    for i in upper_case[14..27].iter() {
        print!("{} ", (i + 32) as char);
    }
    print!("\n    ");
    for i in lower_case[27..38].iter() {
        print!("{} ", (i + 32) as char);
    }
    print!("         ");
    for i in upper_case[27..38].iter() {
        print!("{} ", (i + 32) as char);
    }
    print!("\n     ");
    for i in lower_case[38..48].iter() {
        print!("{} ", (i + 32) as char);
    }
    print!("           ");
    for i in upper_case[38..48].iter() {
        print!("{} ", (i + 32) as char);
    }
    print!("\n");
}


// EVALUATION TEXT FILE LOADING AND OUTPUT FUNCTIONS

// Load a word frequency list text file into a single string (with words separated by spaces) and a
// corresponding array of frequencies for those words.
fn load_word_frequency_list_to_string(path: &Path, multiplier: f64) -> (String, Vec<f32>)
{
    let mut file = std::fs::File::open(path).unwrap();
    let mut list = String::new();
    file.read_to_string(&mut list).unwrap();
    let filename = path.file_name().unwrap();
    let filename_str = filename.to_str().unwrap();
    if let Err((c, ln)) = check_valid_ascii_subset(&list[..]) {
        panic!("On line {} of {}: invalid character, {} -> {}", ln, filename_str, c as u32, c);
    }
    let mut line_num = 1usize;
    let mut words = String::new();
    let mut freqs = Vec::new();
    for line in list[..].lines() {
        let mut word_freq_pair = line.split('\t');
        let word = match word_freq_pair.next() {
            Some(w) => w.trim(),
            None    => panic!("In word frequency list {} on line {}: empty line.",
                              filename_str, line_num)
        };
        if words.len() > 0 {
            words.push_str(" ");
        }
        words.push_str(word);
        let freq_input = match word_freq_pair.next() {
            Some(i) => i,
            None    => panic!("In word frequency list {} on line {}: no frequency listed.",
                              filename_str, line_num)
        };
        let freq = match freq_input.parse::<f64>() {
            Ok(f) => f,
            _     => panic!("In word frequency list {} on line {}: {} is not a double.",
                            filename_str, line_num, freq_input)
        };
        assert!(freq.is_finite(), "In {} on line {}: infinite frequency!", filename_str, line_num);
        freqs.push((freq * multiplier) as f32);
        line_num += 1;
    }
    (words, freqs)
}

// Create a word frequency list by loading it from a text file and normalizing the character bytes.
//fn load_word_frequency_list(path: &Path, multiplier: f64) -> (Vec<u8>, Vec<f32>)
//{
//    let (words, freqs) = load_word_frequency_list_to_string(path, multiplier);
//    let mut words_bytes = words.as_bytes().to_vec();
//    for byte in words_bytes.iter_mut() {
//        *byte -= 32;
//    }
//    return (words_bytes, freqs);
//}

// Load a word frequency list text file into a word frequency hashmap
fn load_list_to_word_frequency_hashmap(path: &Path, multiplier: f64, hm: &mut HashMap<String, f32>)
{
    let (words, freqs) = load_word_frequency_list_to_string(path, multiplier);
    for (word, freq) in words[..].split(' ').zip(freqs.iter()) {
        match hm.entry(word.to_string()) {
            Vacant(entry) => { entry.insert(*freq); },
            Occupied(mut entry) => { *entry.get_mut() += *freq; },
        }
    }
}

// Load an evaluation text file as a string, verify it, and add it to a word frequency hashmap
fn load_text_to_word_frequency_hashmap(path: &Path, hm: &mut HashMap<String, f32>)
{
    let mut file = std::fs::File::open(path).unwrap();
    let mut text = String::new();
    file.read_to_string(&mut text).unwrap();
    let filename = path.file_name().unwrap();
    if let Err((c,ln)) = check_valid_ascii_subset(&text[..]) {
        panic!("On line {} of {}: invalid character {} -> {}",
               ln, filename.to_str().unwrap(), c as u32, c);
    }
    let new_text = text.replace("\n", " ").replace("\r", " ").replace("\t", " ");
    for line in new_text[..].lines() {
        for word in line.split(' ') {
            let mut c0 = 0;
            let mut c1 = 0;
            for c in word.chars() {
                if !('A' <= c && c <= 'Z') && !('a' <= c && c <= 'z') && c != '\'' {
                    if c1 > c0 {
                        match hm.entry(word[c0..c1].to_string()) {
                            Vacant(entry) => { entry.insert(1.0); },
                            Occupied(mut entry) => { *entry.get_mut() += 1.0; },
                        }
                    }
                    match hm.entry(c.to_string()) {
                        Vacant(entry) => { entry.insert(1.0); },
                        Occupied(mut entry) => { *entry.get_mut() += 1.0; },
                    }
                    c0 = c1 + 1;
                }
                c1 += 1;
            }
            if c1 > c0 {
                match hm.entry(word[c0..c1].to_string()) {
                    Vacant(entry) => { entry.insert(1.0); },
                    Occupied(mut entry) => { *entry.get_mut() += 1.0; },
                }
            }
        }
    }
}

// Output a word frequency list to a text file
//fn output_word_frequency_list(path: &Path, words: &[u8], freqs: &[f32])
//{
//    let mut text = String::new();
//    for (word, freq) in words.split(|x| { *x == 0u8 }).zip(freqs.iter()) {
//        for i in word.iter() {
//            text.push((i + 32u8) as char);
//        }
//        text.push('\t');
//        text.push_str(&std::f32::to_string(*freq)[..]);
//        text.push('\n');
//    }
//    let mut file = std::fs::File::create(path).unwrap();
//    let io_result = file.write_all(text.as_bytes());
//    assert!(io_result.is_ok());
//}

// Load a directory of evaluation texts and word frequency list files into an array of byte arrays
fn load_texts_directory(dir_filename: &str) -> (Vec<u8>, Vec<f32>)
{
    let mut hm = HashMap::new();
    let dir = Path::new(dir_filename);
    let dir_metadata = std::fs::metadata(dir).unwrap();
    assert!(dir_metadata.is_dir(), "File is not a directory: {:?}", dir);
    let dir_contents = std::fs::read_dir(&dir).unwrap();
    for entry in dir_contents {
        let entry = entry.unwrap().path();
        let efn = entry.file_name().unwrap().to_str().unwrap();
        let l = efn.len();
        if l >= 4 && &efn[l-4..l] == ".txt" {
            if l >= 8 && &efn[l-8..l-4] == ".wfl" {
                let multiplier = if efn == "corpus_1.wfl.txt" {
                    CORPUS_1_COEFFICIENT
                } else if efn == "corpus_2.wfl.txt" {
                    CORPUS_2_COEFFICIENT
                } else {
                    1.0f64
                };
                load_list_to_word_frequency_hashmap(&entry, multiplier, &mut hm);
            } else {
                load_text_to_word_frequency_hashmap(&entry, &mut hm);
            }
        }
    }
    let mut hm_vec = hm.iter().collect::<Vec<(&String, &f32)>>();
    hm_vec.sort_by(|a: &(&String, &f32), b: &(&String, &f32)| -> std::cmp::Ordering {
        let (_, a2): (&String, &f32) = *a;
        let (_, b2): (&String, &f32) = *b;
        b2.partial_cmp(a2).unwrap()
    });
    let words_size = hm_vec.len() + hm_vec.iter().map(|&x| x.0.len()).fold(0usize, |a, b| a + b);
    let mut words: Vec<u8>  = Vec::with_capacity(words_size);
    let mut freqs: Vec<f32> = Vec::with_capacity(hm_vec.len());
    for wf_tuple in hm_vec.iter() {
        let (word, freq) = *wf_tuple;
        if *freq > CORPUS_MIN_WORD_FREQUENCY {
            words.extend(word.as_bytes().iter().map(|&x| x));
            words.push(32);
            freqs.push(*freq);
        }
    }
    words.pop();
    for c in words.iter_mut() {
        *c -= 32;
    }
    //output_word_frequency_list(&wfl, &words[..], &freqs[..]);
    return (words, freqs);
}


// LAYOUT OPTIMIZATION FUNCTIONS

// When it comes to discrete optimization, there are many different techniques. For now, I am
// opting to use a combination of simulated annealing and tabu search. These rely on the idea of
// swapping individual symbols or keys in the layout. The word "symbol" in this context means one
// of the two characters assigned to a key: one you get when pressing the key by itself and the
// other you get by holding down shift while pressing the key. A complication arises from the fact
// that we always want to keep the lowercase and uppercase versions of letters together on the same
// key, and also we might want to keep certain letters on the home row, and perhaps keep some
// symbols, like the numbers, from moving around, because they make more sense to keep in order in
// the final layout. We also want to keep letters out of the top row, and some other places.

// Therefore, we create the idea of a layout swap, of which there are three types: symbol swaps,
// home-eight key swaps, and letter key swaps. These swap types do not interact with one another.
// To perform a swap, we randomly select a type, and then within that type we randomly select two
// entities to swap.

#[derive(Copy, Clone)]
enum LayoutSwap {
    None,
    Symbol(u8),
    Home8K(u8),
    Letter(u8),
}

struct LayoutSwapper
{
    tabu_swaps:      [LayoutSwap; 2*NUM_TABU_SWAPS],
    symbol_swaps:     Vec<u8>,
    home8k_swaps:     Vec<u8>,
    letter_swaps:     Vec<u8>,
    iteration:        usize,
    random_bits:      usize,
    random_bits_left: usize,
}

impl LayoutSwapper
{
    fn new(layout: &[u8; 190]) -> LayoutSwapper {
        assert_valid_layout(layout);
        let frozen = |s: u8| { FROZEN_SYMBOLS.chars().any(|x| x == (s+32) as char) };

        let symbol_swaps = (1u8..95).filter(|s| {
            !(33 <= *s && *s < 59) &&
            !(65 <= *s && *s < 91) &&
            !frozen(*s)
        }).collect::<Vec<u8>>();
        assert!(symbol_swaps.len() != 1, "Must not have exactly 1 free symbol.");

        let home8k_swaps = HOME_EIGHT.iter().map(|k| *k).filter(|k| {
            let s       = layout[(95+*k)    as usize];
            let s_shift = layout[(95+*k+47) as usize];
            !frozen(s) && !frozen(s_shift)
        }).collect::<Vec<u8>>();
        assert!(home8k_swaps.len() != 1, "Must not have exactly 1 free home eight key.");

        let letter_swaps = (14u8..47).filter(|k| {
            let s       = layout[(95+*k)    as usize];
            let s_shift = layout[(95+*k+47) as usize];
            let c       = (s + 32) as char;
            c.is_alphabetic() && !frozen(s) && !frozen(s_shift) && !HOME_EIGHT.contains(k) &&
            *k != 32 && *k != 26 && *k != 19 && *k != 25
        }).collect::<Vec<u8>>();
        assert!(letter_swaps.len() != 1, "Must not have exactly 1 free letter key.");

        assert!(symbol_swaps.len() + home8k_swaps.len() + letter_swaps.len() >= 2 * NUM_TABU_SWAPS,
                "The number of tabu swaps is higher than the number of possible swaps.");
        LayoutSwapper{
            tabu_swaps:      [LayoutSwap::None; 2*NUM_TABU_SWAPS],
            symbol_swaps:     symbol_swaps,
            home8k_swaps:     home8k_swaps,
            letter_swaps:     letter_swaps,
            iteration:        0,
            random_bits:      0,
            random_bits_left: 0,
        }
    }

    fn random_small_index(&mut self, array_length: usize) -> usize {
        // Possibly create new random bits
        if self.random_bits_left < 10 {
            self.random_bits_left = std::mem::size_of::<usize>() * 8;
            self.random_bits = rand::random::<usize>();
        }
        // Sample 10 bits at a time
        let result = (self.random_bits & ((1<<10)-1)) % array_length;
        self.random_bits_left -= 10;
        self.random_bits >>= 10;
        result
    }

    fn swap(&mut self, layout: &mut [u8; 190]) {
        if NUM_TABU_SWAPS > 0 {
            let t1 = self.tabu_swaps[2*self.iteration+0];
            match t1 {
                LayoutSwap::Symbol(s) => { self.symbol_swaps.push(s); },
                LayoutSwap::Home8K(k) => { self.home8k_swaps.push(k); },
                LayoutSwap::Letter(k) => { self.letter_swaps.push(k); },
                LayoutSwap::None      => { }
            }
            let t2 = self.tabu_swaps[2*self.iteration+1];
            match t2 {
                LayoutSwap::Symbol(s) => { self.symbol_swaps.push(s); },
                LayoutSwap::Home8K(k) => { self.home8k_swaps.push(k); },
                LayoutSwap::Letter(k) => { self.letter_swaps.push(k); },
                LayoutSwap::None      => { }
            }
        }

        let symbol_len = if self.symbol_swaps.len() > 1 { self.symbol_swaps.len() } else { 0 };
        let home8k_len = if self.home8k_swaps.len() > 1 { self.home8k_swaps.len() } else { 0 };
        let letter_len = if self.letter_swaps.len() > 1 { self.letter_swaps.len() } else { 0 };
        let num_swaps = symbol_len + home8k_len + letter_len;
        let mut i1 = self.random_small_index(num_swaps);
        if i1 < symbol_len {
            // symbol swap
            let mut i2 = self.random_small_index(symbol_len - 1);
            if i1 <= i2 {
                i2 += 1;
            }
            let s1 = self.symbol_swaps[i1];
            let s2 = self.symbol_swaps[i2];
            let k1 = layout[s1 as usize] as usize;
            let k2 = layout[s2 as usize] as usize;
            layout.swap(s1 as usize, s2 as usize);
            layout.swap(95+k1, 95+k2);
            // add swaps to tabu list
            if NUM_TABU_SWAPS > 0 {
                self.tabu_swaps[2*self.iteration+0] = LayoutSwap::Symbol(s1);
                self.tabu_swaps[2*self.iteration+1] = LayoutSwap::Symbol(s2);
                if i1 < i2 {
                    self.symbol_swaps.remove(i2);
                    self.symbol_swaps.remove(i1);
                } else {
                    self.symbol_swaps.remove(i1);
                    self.symbol_swaps.remove(i2);
                }
            }
        } else if i1 < symbol_len + home8k_len {
            // home-eight-key swap
            i1 -= symbol_len;
            let mut i2 = self.random_small_index(home8k_len - 1);
            if i1 <= i2 {
                i2 += 1;
            }
            let k1 = self.home8k_swaps[i1];
            let k2 = self.home8k_swaps[i2];
            let k1_index        = 95+(k1 as usize);
            let k2_index        = 95+(k2 as usize);
            let k1_shift_index  = 95+(k1 as usize)+47;
            let k2_shift_index  = 95+(k2 as usize)+47;
            let k1_symbol       = layout[k1_index]       as usize;
            let k2_symbol       = layout[k2_index]       as usize;
            let k1_shift_symbol = layout[k1_shift_index] as usize;
            let k2_shift_symbol = layout[k2_shift_index] as usize;
            layout.swap(k1_index,        k2_index);
            layout.swap(k1_symbol,       k2_symbol);
            layout.swap(k1_shift_index,  k2_shift_index);
            layout.swap(k1_shift_symbol, k2_shift_symbol);
            // add swaps to tabu list
            if NUM_TABU_SWAPS > 0 {
                self.tabu_swaps[2*self.iteration+0] = LayoutSwap::Home8K(k1);
                self.tabu_swaps[2*self.iteration+1] = LayoutSwap::Home8K(k2);
                if i1 < i2 {
                    self.home8k_swaps.remove(i2);
                    self.home8k_swaps.remove(i1);
                } else {
                    self.home8k_swaps.remove(i1);
                    self.home8k_swaps.remove(i2);
                }
            }
        } else {
            // letter key swap
            i1 -= symbol_len + home8k_len;
            let mut i2 = self.random_small_index(letter_len - 1);
            if i1 <= i2 {
                i2 += 1;
            }
            let k1 = self.letter_swaps[i1];
            let k2 = self.letter_swaps[i2];
            let k1_index        = 95+(k1 as usize);
            let k2_index        = 95+(k2 as usize);
            let k1_shift_index  = 95+(k1 as usize)+47;
            let k2_shift_index  = 95+(k2 as usize)+47;
            let k1_symbol       = layout[k1_index]       as usize;
            let k2_symbol       = layout[k2_index]       as usize;
            let k1_shift_symbol = layout[k1_shift_index] as usize;
            let k2_shift_symbol = layout[k2_shift_index] as usize;
            layout.swap(k1_index,        k2_index);
            layout.swap(k1_symbol,       k2_symbol);
            layout.swap(k1_shift_index,  k2_shift_index);
            layout.swap(k1_shift_symbol, k2_shift_symbol);
            // add swaps to tabu list
            if NUM_TABU_SWAPS > 0 {
                self.tabu_swaps[2*self.iteration+0] = LayoutSwap::Letter(k1);
                self.tabu_swaps[2*self.iteration+1] = LayoutSwap::Letter(k2);
                if i1 < i2 {
                    self.letter_swaps.remove(i2);
                    self.letter_swaps.remove(i1);
                } else {
                    self.letter_swaps.remove(i1);
                    self.letter_swaps.remove(i2);
                }
            }
        }

        if NUM_TABU_SWAPS > 0 {
            self.iteration = (self.iteration + 1) % NUM_TABU_SWAPS;
        }
    }
}


// Custom extended precision data structure to keep track of many summed floats
#[derive(Copy, Clone)]
struct LayoutScore
{
    i: i32,
    f: f32
}

impl LayoutScore
{
    fn zero() -> LayoutScore {
        LayoutScore{ i: 0i32, f: 0f32 }
    }

    fn to_f64(&self) -> f64 {
        (self.i as f64)*1000.0 + (self.f as f64)
    }

    fn add_f32(&mut self, addend: f32) {
        self.f += addend;
        if self.f >  1000.0 {
            let d_i = (self.f / 1000.0) as i32;
            self.i +=  d_i;
            self.f -= (d_i * 1000) as f32;
            return;
        }
        if self.f < -1000.0 {
            let d_i = (self.f / 1000.0) as i32;
            self.i +=  d_i;
            self.f -= (d_i * 1000) as f32;
        }
    }
}

impl Add<LayoutScore> for LayoutScore {
    type Output = LayoutScore;

    fn add(self, other: LayoutScore) -> LayoutScore {
        let mut new_i = self.i + other.i;
        let mut new_f = self.f + other.f;
        while new_f > 1000.0 {
            new_i += 1;
            new_f -= 1000.0;
        }
        while new_f < -1000.0 {
            new_i -= 1;
            new_f += 1000.0;
        }
        LayoutScore {i: new_i, f: new_f}
    }
}

impl PartialEq for LayoutScore
{
    fn eq(&self, other: &LayoutScore) -> bool {
        ((self.i == other.i   && self.f == other.f) ||
         (self.i == other.i-1 && self.f == other.f + 1000.0) ||
         (self.i == other.i+1 && self.f == other.f - 1000.0))
    }
}

impl PartialOrd for LayoutScore
{
    fn partial_cmp(&self, other: &LayoutScore) -> Option<std::cmp::Ordering> {
        if self.i == other.i {
            self.f.partial_cmp(&other.f)
        } else if self.i < other.i-1 {
            Some(std::cmp::Ordering::Less)
        } else if self.i > other.i+1 {
            Some(std::cmp::Ordering::Greater)
        } else if self.i == other.i+1 {
            self.f.partial_cmp(&(other.f - 1000.0))
        } else {
            self.f.partial_cmp(&(other.f + 1000.0))
        }
    }

    fn lt(&self, other: &LayoutScore) -> bool {
        ((self.i <  other.i-1) ||
         (self.i == other.i-1 && self.f-1000.0 < other.f) ||
         (self.i == other.i   && self.f        < other.f) ||
         (self.i == other.i+1 && self.f+1000.0 < other.f))
    }
}

// Calculate the probability of accepting a random layout swap based on score differences.
fn probability(s0: LayoutScore, s1: LayoutScore, t: f64) -> f64
{
    if s1 < s0 {
        1.0
    } else {
        ((((s0.i - s1.i) as f64) * 1000.0 + ((s0.f - s1.f) as f64)) / t).exp()
    }
}

// Hard-coded bloom-like filter (false positives, no false negatives) to speed up hashmap look-up
fn triple_filter(k0: u8, k1: u8, k2: u8) -> bool
{
    let d = 2*(k1 as i32) - (k0 as i32) - (k2 as i32);
    (-1 <= d && d <= 1) || (-13 <= d && d <= -10) || d == -26
}

#[test]
fn triple_filter_test()
{
    for &(k0, k1, k2, _) in TRIPLE_METRIC.iter() {
        assert!(triple_filter(k0, k1, k2),
                "Triple filter has a false negative for {}, {}, {}", k0, k1, k2);
    }
}

// Custom objective function
struct LayoutObjectiveFunction
{
    words:         Vec<u8>,
    freqs:         Vec<f32>,
    double_scores: [f32; 9025],
    triple_scores: HashMap<(u8, u8, u8), f32>,
}

impl LayoutObjectiveFunction
{
    // Assemble the objective function for scoring layouts. It is a linear combination of single
    // key score, double key score, triple key score, a hand-alternation penalty, a shift-holding
    // penalty, and a reversed triple-penalty.
    fn new() -> LayoutObjectiveFunction {
        let (words, freqs) = load_texts_directory("texts");

        let mut double_scores = [0f32; 9025]; // (9025 = 95*95)

        // Add in single key scores and shift penalties, removing repeat penalties
        let min_single_metric = SINGLE_METRIC.iter().fold(std::f32::INFINITY, |m, &x| m.min(x));
        for i in 0..95 {
            for j in 0..48 {
                if j != i {
                    let s = (SINGLE_METRIC[j  ] - min_single_metric) * SINGLE_METRIC_COEFFICIENT;
                    double_scores[i*95+j   ] = s;
                }
            }
            for j in 0..47 {
                if j+48 != i {
                    let s = (SINGLE_METRIC[j+1] - min_single_metric) * SINGLE_METRIC_COEFFICIENT;
                    double_scores[i*95+j+48] = s + SHIFT_HOLDING_PENALTY;
                }
            }
        }

        // Add in the double key scores
        let min_double_metric = DOUBLE_METRIC.iter().fold(std::f32::INFINITY, |m,&x| m.min(x.2));
        for &(ki, kj, ks) in DOUBLE_METRIC.iter() {
            let i  = ki as usize;
            let j  = kj as usize;
            let s  = (ks - min_double_metric) * DOUBLE_METRIC_COEFFICIENT;
            double_scores[ i    *95+j   ] += s;
            double_scores[ i    *95+j+47] += s;
            double_scores[(i+47)*95+j   ] += s;
            double_scores[(i+47)*95+j+47] += s;
        }

        // Add in alternating hand penalties
        for i in 0..95 {
            for j in 0..95 {
                let ai = if i < 48 { i } else { i - 47 };
                let aj = if j < 48 { j } else { j - 47 };
                let fi = FINGER_ASSIGNMENT[ai];
                let fj = FINGER_ASSIGNMENT[aj];
                if (fi != 0) && (fj != 0) && ((fi < 5 && fj >= 5) || (fj < 5 && fi >= 5)) {
                    double_scores[i*95+j] += HAND_ALTERNATION_PENALTY;
                }
            }
        }

        let mut triple_scores: HashMap<(u8, u8, u8), f32> = HashMap::new();
        for &(k1, k2, k3, ks) in TRIPLE_METRIC.iter() {
            let s = ks * TRIPLE_METRIC_COEFFICIENT;
            triple_scores.insert((k1,k2,k3), s);
            triple_scores.insert((k3,k2,k1), s+REVERSED_TRIPLE_PENALTY*TRIPLE_METRIC_COEFFICIENT);
        }

        LayoutObjectiveFunction{
            words: words,
            freqs: freqs,
            double_scores: double_scores,
            triple_scores: triple_scores
        }
    }

    // Assign a score to a word in adjusted byte format
    fn word_score(&self, layout: &[u8; 190], word: &[u8]) -> f32 {
        let mut score = 0f32;
        let mut k0 = 0u8;
        let mut k1 = 0u8;
        for c2 in word.iter() {
            let k2 = layout[*c2 as usize];
            score += self.double_scores[(k1 as usize) * 95 + (k2 as usize)];
            if k0 != 0 && k1 != 0 && triple_filter(k0, k1, k2) {
                if let Some(&triple_score) = self.triple_scores.get(&(k0, k1, k2)) {
                    score += triple_score;
                }
            }
            k0 = k1;
            k1 = k2;
        }
        score
    }

    // Assign a score to a layout based on the metric scores applied to a word frequency list.
    fn score(&self, layout: &[u8; 190]) -> LayoutScore {
        let mut score = LayoutScore::zero();
        for (word, freq) in self.words.split(|x| { *x == 0 }).zip(self.freqs.iter()) {
            score.add_f32(self.word_score(layout, word) * *freq);
        }
        score
    }

    // Calculate the equivalent of a count for each character, sort them, and print them.
    fn print_char_counts(&self) {
        let char_count = |c: char| -> f32 {
            let mut count = 0.0;
            for (word, freq) in self.words.split(|x| { *x == 0 }).zip(self.freqs.iter()) {
                for i in word.iter() {
                    if (i +32) as char == c {
                        count += *freq;
                    }
                }
            }
            count
        };
        let mut char_counts = Vec::new();
        for byte in 33u8..127 {
            if ('a' as u8) <= byte && byte <= ('z' as u8) {
                continue;
            }
            let c = byte as char;
            let count = if ('A' as u8) <= byte && byte <= ('Z' as u8) {
                char_count(c) + char_count(((c as u8) + 32) as char)
            } else {
                char_count(c)
            };
            char_counts.push((c, count));
        }
        char_counts.sort_by(|a: &(char, f32), b: &(char, f32)| -> std::cmp::Ordering {
            let (_, a_count) = *a;
            let (_, b_count) = *b;
            b_count.partial_cmp(&a_count).unwrap()
         });
        println!("Character counts for the current objective function:");
        for &(c, count) in char_counts.iter() {
            println!("{}  {}", c, count);
        }
    }

    // Calculate how much each finger is used as a percentage for each hand.
    fn print_layout_finger_usage(&self, layout: &[u8]) {
        let mut fu = [0f32; 8]; // finger usage
        for (word, freq) in self.words.split(|x| { *x == 0 }).zip(self.freqs.iter()) {
            for c in word.iter() {
                let k = layout[*c as usize] as usize;
                let i = if k < 48 { k } else { k - 47 };
                let finger_index = FINGER_ASSIGNMENT[i];
                assert!(finger_index > 0);
                fu[(finger_index - 1) as usize] += *freq;
            }
        }
        let  left_hand = fu[0] + fu[1] + fu[2] + fu[3];
        let right_hand = fu[4] + fu[5] + fu[6] + fu[7];
        let total      = (left_hand + right_hand) / 100f32;
        println!(" Left hand: {:4.1}% + {:4.1}% + {:4.1}% + {:4.1}% = {:4.1}%",
                 fu[0] / total, fu[1] / total, fu[2] / total, fu[3] / total,  left_hand / total);
        println!("Right hand: {:4.1}% + {:4.1}% + {:4.1}% + {:4.1}% = {:4.1}%  (listed backwards)",
                 fu[7] / total, fu[6] / total, fu[5] / total, fu[4] / total, right_hand / total);
    }
}

#[test]
fn objective_function_word_score_test()
{
    let objective = LayoutObjectiveFunction::new();
    let layout = layout_from_string(_QWERTY_STRING);
    let word = "asdf".chars().map(|x| (x as u8) - 32).collect::<Vec<u8>>();
    let min_single_metric = SINGLE_METRIC.iter().fold(std::f32::INFINITY, |m,&x| m.min(x));
    let single_score = ( SINGLE_METRIC[layout[word[0] as usize] as usize] - min_single_metric
                       + SINGLE_METRIC[layout[word[1] as usize] as usize] - min_single_metric
                       + SINGLE_METRIC[layout[word[2] as usize] as usize] - min_single_metric
                       + SINGLE_METRIC[layout[word[3] as usize] as usize] - min_single_metric)
                       * SINGLE_METRIC_COEFFICIENT;
    let min_double_metric = DOUBLE_METRIC.iter().fold(std::f32::INFINITY, |m,&x| m.min(x.2));
    let calculate_double_score = |layout: &[u8], c0: u8, c1: u8| -> f32 {
        let k0 = layout[c0 as usize];
        let k1 = layout[c1 as usize];
        if let Some(&(_,_,s)) = DOUBLE_METRIC.iter().find(|&&(x0, x1, _)| x0 == k0 && x1 == k1) {
            return s - min_double_metric;
        }
        if let Some(&(_,_,s)) = DOUBLE_METRIC.iter().find(|&&(x0, x1, _)| x0 == k1 && x1 == k0) {
            return s - min_double_metric;
        }
        return -min_double_metric;
    };
    let double_score = ( calculate_double_score(&layout, word[0], word[1])
                       + calculate_double_score(&layout, word[1], word[2])
                       + calculate_double_score(&layout, word[2], word[3]))
                       * DOUBLE_METRIC_COEFFICIENT;
    let calculate_triple_score = |layout: &[u8], c0: u8, c1: u8, c2: u8| -> f32 {
        let k0 = layout[c0 as usize];
        let k1 = layout[c1 as usize];
        let k2 = layout[c2 as usize];
        if let Some(&(_,_,_,s)) = TRIPLE_METRIC.iter().find(|&&(x0, x1, x2, _)|
            x0 == k0 && x1 == k1 && x2 == k2) {
            return s;
        }
        if let Some(&(_,_,_,s)) = TRIPLE_METRIC.iter().find(|&&(x0, x1, x2, _)|
            x0 == k2 && x1 == k1 && x2 == k0) {
            return s + REVERSED_TRIPLE_PENALTY;
        }
        return 0.0;
    };
    let triple_score = ( calculate_triple_score(&layout, word[0], word[1], word[2])
                       + calculate_triple_score(&layout, word[1], word[2], word[3]))
                       * TRIPLE_METRIC_COEFFICIENT;
    let ws0 = single_score + double_score + triple_score;
    let ws1 = objective.word_score(&layout, &word);
    assert_eq!(ws0, ws1);
}




fn main()
{
    // Handle the optional command line argument to specify the layout output filename
    let mut args = std::env::args();
    assert!(args.next().is_some());
    let maybe_arg = args.next();
    let output_prefix = match maybe_arg {
        Some(os_string) => os_string,
        None            => "layout".to_string()
    };

    let objective  = LayoutObjectiveFunction::new();
    if PRINT_OBJECTIVE_FUNCTION {
        print_single_metric();
        for i in 1u8..48 {
            print_double_metric(i);
        }
        objective.print_char_counts();
        print!("\n");
    }

    let mut layout = read_layout_file("optimal_layout.txt");
    let mut score  = objective.score(&layout);

    // Display the starting layout
    print_layout(&layout);
    print!("\n     Score: {}\n", score.to_f64());
    objective.print_layout_finger_usage(&layout);
    print!("\n");

    // Perform several simulated annealing cycles
    let mut cycle_iteration   = 0u64;
    let mut cycle_temperature = CYCLE_TEMPERATURE_START;

    while cycle_temperature > CYCLE_TEMPERATURE_FINAL {
        let mut new_layout:     [u8; 190];
        let mut best_layout     = layout;
        let mut best_score      = score;
        let     prev_best_score = score;
        let mut random_key_swap = LayoutSwapper::new(&layout);
        let mut temperature     = cycle_temperature;
        let mut iteration       = 0u64;

        println!("Iteration {}", cycle_iteration);
        while temperature > TEMPERATURE_FINAL {

            // Make new layout
            new_layout = layout;
            random_key_swap.swap(&mut new_layout);

            // Test new layout
            let new_score = objective.score(&new_layout);

            // Possibly switch current layout to new one based on probability
            if probability(score, new_score, temperature) > rand::random::<f64>() {
                layout = new_layout;
                score  = new_score;
            }

            // Save layout if it's the best one yet
            if score < best_score {
                best_layout = layout;
                best_score  = score;
            }

            // Display diagnostic information
            if iteration % 100000 == 0 {
                println!("{:9}    T: {:9.2}    C: {:12.2}    B: {:12.2}",
                         iteration, temperature, score.to_f64(), best_score.to_f64());
            }

            iteration   += 1;
            temperature *= TEMPERATURE_FACTOR;
        }

		// Output new best layout if different than previous best
		print!("\n");
        if best_score != prev_best_score {
            print_layout(&best_layout);
            print!("\n     Score: {}\n", best_score.to_f64());
            objective.print_layout_finger_usage(&best_layout);
	        print!("\n");
			write_layout_file(&best_layout, &format!("layouts/{}_{}_{}.txt",
									                output_prefix,
                                                    best_score.to_f64() as i32,
                                                    cycle_iteration)[..]);
            write_layout_file(&best_layout, "optimal_layout.txt");
        }

        // Prepare for next cycle
        score = best_score;
        cycle_iteration   += 1;
        cycle_temperature *= CYCLE_TEMPERATURE_FACTOR;
    }
}
