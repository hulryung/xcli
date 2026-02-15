const BASIC_LATIN_MAX: u32 = 0x10FF;
const MAX_WEIGHTED_LEN: usize = 280;
const SEPARATOR: &str = "\n---\n";

/// Compute X API weighted character count.
/// - Basic Latin (U+0000-U+10FF): weight 1
/// - Everything else (CJK, Korean, emoji, etc.): weight 2
pub fn weighted_len(text: &str) -> usize {
    text.chars()
        .map(|c| if (c as u32) <= BASIC_LATIN_MAX { 1 } else { 2 })
        .sum()
}

/// Split text into tweet-sized chunks.
/// 1. If text contains the separator "---" (on its own line), split on it.
/// 2. If no separator but text exceeds 280 weighted chars, auto-split:
///    - paragraph breaks (\n\n) first
///    - then sentence boundaries (. ! ?)
///    - then word boundaries
/// 3. If text fits in one tweet, return it as-is.
pub fn split_text(text: &str) -> Vec<String> {
    // 1. Check for separator
    if text.contains(SEPARATOR) {
        let parts: Vec<String> = text
            .split(SEPARATOR)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !parts.is_empty() {
            return parts;
        }
    }

    // 2. If fits in one tweet, return as-is
    if weighted_len(text) <= MAX_WEIGHTED_LEN {
        return vec![text.to_string()];
    }

    // 3. Auto-split
    auto_split(text)
}

fn auto_split(text: &str) -> Vec<String> {
    // Try paragraph split first
    let paragraphs: Vec<&str> = text.split("\n\n").collect();
    if paragraphs.len() > 1 {
        let mut result = Vec::new();
        for p in paragraphs {
            let trimmed = p.trim();
            if trimmed.is_empty() {
                continue;
            }
            if weighted_len(trimmed) <= MAX_WEIGHTED_LEN {
                result.push(trimmed.to_string());
            } else {
                result.extend(split_by_sentences(trimmed));
            }
        }
        return result;
    }

    // No paragraph breaks — split by sentences
    let sentence_chunks = split_by_sentences(text);
    if sentence_chunks.len() > 1 {
        return sentence_chunks;
    }

    // No sentence breaks — split by words
    split_by_words(text)
}

fn split_by_sentences(text: &str) -> Vec<String> {
    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();

    for part in SentenceIter::new(text) {
        if current.is_empty() {
            current = part;
        } else if weighted_len(&format!("{current} {part}")) <= MAX_WEIGHTED_LEN {
            current = format!("{current} {part}");
        } else {
            chunks.push(current);
            current = part;
        }
    }
    if !current.is_empty() {
        if weighted_len(&current) <= MAX_WEIGHTED_LEN {
            chunks.push(current);
        } else {
            chunks.extend(split_by_words(&current));
        }
    }
    chunks
}

/// Iterator that splits text on sentence-ending punctuation followed by a space.
/// Keeps the punctuation with the preceding sentence.
struct SentenceIter<'a> {
    remaining: &'a str,
}

impl<'a> SentenceIter<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            remaining: text.trim(),
        }
    }
}

impl<'a> Iterator for SentenceIter<'a> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        if self.remaining.is_empty() {
            return None;
        }

        // Find the next sentence boundary: ". " or "! " or "? "
        let terminators = [". ", "! ", "? "];
        let mut earliest: Option<(usize, usize)> = None;

        for t in &terminators {
            if let Some(pos) = self.remaining.find(t) {
                match earliest {
                    None => earliest = Some((pos, t.len())),
                    Some((ep, _)) if pos < ep => earliest = Some((pos, t.len())),
                    _ => {}
                }
            }
        }

        match earliest {
            Some((pos, tlen)) => {
                let sentence = self.remaining[..pos + tlen - 1].trim().to_string();
                self.remaining = self.remaining[pos + tlen..].trim();
                Some(sentence)
            }
            None => {
                let rest = self.remaining.trim().to_string();
                self.remaining = "";
                Some(rest)
            }
        }
    }
}

fn split_by_words(text: &str) -> Vec<String> {
    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else {
            let candidate = format!("{current} {word}");
            if weighted_len(&candidate) <= MAX_WEIGHTED_LEN {
                current = candidate;
            } else {
                chunks.push(current);
                current = word.to_string();
            }
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    // weighted_len tests
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

    // split_text tests
    #[test]
    fn short_text_no_split() {
        let result = split_text("hello world");
        assert_eq!(result, vec!["hello world"]);
    }

    #[test]
    fn separator_split() {
        let result = split_text("first tweet\n---\nsecond tweet");
        assert_eq!(result, vec!["first tweet", "second tweet"]);
    }

    #[test]
    fn separator_trims_whitespace() {
        let result = split_text("  first  \n---\n  second  ");
        assert_eq!(result, vec!["first", "second"]);
    }

    #[test]
    fn auto_split_on_paragraphs() {
        let p1 = "a".repeat(200);
        let p2 = "b".repeat(200);
        let text = format!("{p1}\n\n{p2}");
        let result = split_text(&text);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], p1);
        assert_eq!(result[1], p2);
    }

    #[test]
    fn auto_split_on_sentences() {
        let s1 = "a".repeat(200);
        let s2 = "b".repeat(200);
        let text = format!("{s1}. {s2}.");
        let result = split_text(&text);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], format!("{s1}."));
        assert_eq!(result[1], format!("{s2}."));
    }

    #[test]
    fn auto_split_on_words() {
        let word = "abcdefghij"; // 10 chars
        let words: Vec<&str> = std::iter::repeat(word).take(30).collect();
        let text = words.join(" ");
        let result = split_text(&text);
        assert!(result.len() >= 2);
        for chunk in &result {
            assert!(weighted_len(chunk) <= 280);
        }
    }

    #[test]
    fn separator_empty_parts_filtered() {
        let result = split_text("only part\n---\n\n---\n");
        assert_eq!(result, vec!["only part"]);
    }
}
