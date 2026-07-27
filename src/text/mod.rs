use unicode_segmentation::{self, UnicodeSegmentation};
use unicode_width::{self, UnicodeWidthStr};

pub fn grapheme_len(s: &str) -> usize {
    s.graphemes(true).count()
}

pub fn grapheme_width(g: &str) -> usize {
    g.graphemes(true).next().unwrap().width()
}

pub fn grapheme_to_byte_offset(s: &str, grapheme_idx: usize) -> usize {
    let mut byte_offset = 0;
    for (i, g) in s.graphemes(true).enumerate() {
        if i == grapheme_idx {
            return byte_offset;
        }
        byte_offset += g.len();
    }
    byte_offset
}

pub fn byte_to_grapheme_offset(s: &str, byte_offset: usize) -> usize {
    let mut current_byte_position = 0;
    for (i, g) in s.graphemes(true).enumerate() {
        if byte_offset >= current_byte_position && byte_offset < current_byte_position + g.len() {
            return i;
        }
        current_byte_position += g.len();
    }
    current_byte_position
}

pub fn display_width(s: &str) -> usize {
    s.graphemes(true).map(grapheme_width).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grapheme_len_combining_chars() {
        assert_eq!(grapheme_len("a̐éö̲\r\n"), 4);
    }

    #[test]
    fn grapheme_len_ascii() {
        assert_eq!(grapheme_len("hello"), 5);
    }

    #[test]
    fn grapheme_len_precomposed() {
        assert_eq!(grapheme_len("café"), 4);
    }

    #[test]
    fn grapheme_len_crlf() {
        assert_eq!(grapheme_len("\r\n"), 1);
    }

    #[test]
    fn grapheme_width_ascii() {
        assert_eq!(grapheme_width("a"), 1);
    }

    #[test]
    fn grapheme_width_cjk() {
        assert_eq!(grapheme_width("中"), 2);
    }

    #[test]
    fn grapheme_width_precomposed() {
        assert_eq!(grapheme_width("é"), 1);
    }

    #[test]
    fn grapheme_width_flag_emoji() {
        assert_eq!(grapheme_width("🇺🇸"), 2);
    }

    #[test]
    fn grapheme_to_byte_offset_ascii() {
        assert_eq!(grapheme_to_byte_offset("hello", 3), 3);
    }

    #[test]
    fn grapheme_to_byte_offset_multi_byte() {
        assert_eq!(grapheme_to_byte_offset("café", 3), 3);
    }

    #[test]
    fn grapheme_to_byte_offset_combining() {
        assert_eq!(grapheme_to_byte_offset("a̐é", 1), 3);
    }

    #[test]
    fn grapheme_to_byte_offset_past_end() {
        assert_eq!(grapheme_to_byte_offset("hi", 4), 2);
    }

    #[test]
    fn byte_to_grapheme_offset_ascii() {
        assert_eq!(byte_to_grapheme_offset("hello", 3), 3);
    }

    #[test]
    fn byte_to_grapheme_offset_multi_byte() {
        assert_eq!(byte_to_grapheme_offset("café", 3), 3);
    }

    #[test]
    fn byte_to_grapheme_offset_combining() {
        assert_eq!(byte_to_grapheme_offset("a̐é", 2), 0);
    }

    #[test]
    fn byte_to_grapheme_offset_start() {
        assert_eq!(byte_to_grapheme_offset("hello", 0), 0);
    }

    #[test]
    fn byte_to_grapheme_offset_past_end() {
        assert_eq!(byte_to_grapheme_offset("hello", 10), 5);
    }
}
