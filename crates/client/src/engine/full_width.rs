// https://www.unicode.org/charts/nameslist/n_FF00.html
// extracted with scripts/extract_fullwidth.py

use std::{collections::HashMap, sync::LazyLock};

// in azookey, fullwidth alphabet will not be processed
static HALF_FULL: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("!", "！"),
        ("\"", "＂"),
        ("#", "＃"),
        ("$", "＄"),
        ("%", "％"),
        ("&", "＆"),
        ("'", "＇"),
        ("(", "（"),
        (")", "）"),
        ("*", "＊"),
        ("+", "＋"),
        (",", "、"),
        ("-", "ー"),
        (".", "。"),
        ("/", "／"),
        // ("0", "０"),
        // ("1", "１"),
        // ("2", "２"),
        // ("3", "３"),
        // ("4", "４"),
        // ("5", "５"),
        // ("6", "６"),
        // ("7", "７"),
        // ("8", "８"),
        // ("9", "９"),
        (":", "："),
        (";", "；"),
        ("<", "＜"),
        ("=", "＝"),
        (">", "＞"),
        ("?", "？"),
        ("@", "＠"),
        // ("A", "Ａ"),
        // ("B", "Ｂ"),
        // ("C", "Ｃ"),
        // ("D", "Ｄ"),
        // ("E", "Ｅ"),
        // ("F", "Ｆ"),
        // ("G", "Ｇ"),
        // ("H", "Ｈ"),
        // ("I", "Ｉ"),
        // ("J", "Ｊ"),
        // ("K", "Ｋ"),
        // ("L", "Ｌ"),
        // ("M", "Ｍ"),
        // ("N", "Ｎ"),
        // ("O", "Ｏ"),
        // ("P", "Ｐ"),
        // ("Q", "Ｑ"),
        // ("R", "Ｒ"),
        // ("S", "Ｓ"),
        // ("T", "Ｔ"),
        // ("U", "Ｕ"),
        // ("V", "Ｖ"),
        // ("W", "Ｗ"),
        // ("X", "Ｘ"),
        // ("Y", "Ｙ"),
        // ("Z", "Ｚ"),
        ("[", "［"),
        ("\\", "＼"),
        ("]", "］"),
        ("^", "＾"),
        ("_", "＿"),
        ("`", "｀"),
        // ("a", "ａ"),
        // ("b", "ｂ"),
        // ("c", "ｃ"),
        // ("d", "ｄ"),
        // ("e", "ｅ"),
        // ("f", "ｆ"),
        // ("g", "ｇ"),
        // ("h", "ｈ"),
        // ("i", "ｉ"),
        // ("j", "ｊ"),
        // ("k", "ｋ"),
        // ("l", "ｌ"),
        // ("m", "ｍ"),
        // ("n", "ｎ"),
        // ("o", "ｏ"),
        // ("p", "ｐ"),
        // ("q", "ｑ"),
        // ("r", "ｒ"),
        // ("s", "ｓ"),
        // ("t", "ｔ"),
        // ("u", "ｕ"),
        // ("v", "ｖ"),
        // ("w", "ｗ"),
        // ("x", "ｘ"),
        // ("y", "ｙ"),
        // ("z", "ｚ"),
        ("{", "｛"),
        ("|", "｜"),
        ("}", "｝"),
        ("~", "～"),
    ])
});

#[allow(dead_code)]
pub fn to_halfwidth(s: &str) -> String {
    s.chars()
        .map(|c| {
            match c {
                // FullWidth 'a'..='z'
                '\u{FF41}'..='\u{FF5A}' => char::from_u32(c as u32 - 0xFEE0).unwrap(),
                _ => c,
            }
        })
        .collect()
}

pub fn to_fullwidth(s: &str) -> String {
    s.chars()
        .map(|c| {
            let key = c.to_string();
            if let Some(&v) = HALF_FULL.get(key.as_str()) {
                v.to_string()
            } else {
                c.to_string()
            }
        })
        .collect()
}