const BASIC_LATIN_MAX: u32 = 0x10FF;

/// Compute X API weighted character count.
/// - Basic Latin (U+0000-U+10FF): weight 1
/// - Everything else (CJK, Korean, emoji, etc.): weight 2
pub fn weighted_len(text: &str) -> usize {
    text.chars()
        .map(|c| if (c as u32) <= BASIC_LATIN_MAX { 1 } else { 2 })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_only() {
        assert_eq!(weighted_len("hello"), 5);
    }

    #[test]
    fn korean_only() {
        assert_eq!(weighted_len("안녕하세요"), 10);
    }

    #[test]
    fn mixed() {
        assert_eq!(weighted_len("hi안녕"), 6);
    }

    #[test]
    fn empty() {
        assert_eq!(weighted_len(""), 0);
    }

    #[test]
    fn emoji() {
        assert_eq!(weighted_len("😀"), 2);
    }
}
