//! Reading-time estimation for prose that does not put spaces between words.
//!
//! A whitespace-token count works for English but undercounts Thai (and CJK)
//! by a large factor: Thai is written without inter-word spaces, so a long
//! post can look like a handful of "words". This estimator counts Latin-script
//! words and Thai/CJK characters separately, then applies documented reading
//! speeds:
//!
//! - Latin: 200 words/min (the conventional estimate).
//! - Thai: 900 characters/min, roughly 200 words/min at ~4.5 characters per
//!   word, since Thai has no inter-word spaces and combining marks are part of
//!   the character count.
//! - CJK: 500 characters/min, where one character is close to one word.
//!
//! The constants are deliberately simple and testable; the goal is a
//! defensible "min read" label, not a scientific reading model.

const LATIN_WORDS_PER_MINUTE: f64 = 200.0;
const THAI_CHARS_PER_MINUTE: f64 = 900.0;
const CJK_CHARS_PER_MINUTE: f64 = 500.0;

/// Estimate reading time in minutes, never less than one.
///
/// Markdown syntax and punctuation are not counted as words; alphanumeric
/// runs in spaced scripts are, and every Thai/CJK character contributes to the
/// character-based speeds.
pub fn estimate_minutes(body: &str) -> usize {
  let mut latin_words = 0usize;
  let mut in_latin_word = false;
  let mut thai_chars = 0usize;
  let mut cjk_chars = 0usize;

  for c in body.chars() {
    if is_thai(c) {
      thai_chars += 1;
      if in_latin_word {
        latin_words += 1;
        in_latin_word = false;
      }
    } else if is_cjk(c) {
      cjk_chars += 1;
      if in_latin_word {
        latin_words += 1;
        in_latin_word = false;
      }
    } else if c.is_alphanumeric() {
      in_latin_word = true;
    } else if in_latin_word {
      latin_words += 1;
      in_latin_word = false;
    }
  }
  if in_latin_word {
    latin_words += 1;
  }

  let minutes = latin_words as f64 / LATIN_WORDS_PER_MINUTE
    + thai_chars as f64 / THAI_CHARS_PER_MINUTE
    + cjk_chars as f64 / CJK_CHARS_PER_MINUTE;
  (minutes.ceil() as usize).max(1)
}

/// Thai script block, including vowels, tone marks, digits, and punctuation.
fn is_thai(c: char) -> bool {
  ('\u{0E00}'..='\u{0E7F}').contains(&c)
}

/// The CJK blocks the estimator treats as one reading unit per character.
fn is_cjk(c: char) -> bool {
  matches!(
    c,
    '\u{3040}'..='\u{30FF}'   // Hiragana + Katakana
      | '\u{3400}'..='\u{4DBF}' // CJK Extension A
      | '\u{4E00}'..='\u{9FFF}' // CJK Unified Ideographs
      | '\u{F900}'..='\u{FAFF}' // CJK Compatibility Ideographs
      | '\u{AC00}'..='\u{D7AF}' // Hangul Syllables
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn english_uses_the_conventional_200_words_per_minute() {
    assert_eq!(estimate_minutes(&"word ".repeat(200)), 1);
    assert_eq!(estimate_minutes(&"word ".repeat(201)), 2);
    assert_eq!(estimate_minutes(&"word ".repeat(1000)), 5);
  }

  #[test]
  fn thai_with_no_spaces_is_not_undercounted() {
    // The regression from #19: no whitespace at all, but 9,000 real characters.
    assert_eq!(estimate_minutes(&"ก".repeat(9000)), 10);
  }

  #[test]
  fn thai_estimation_ignores_whitespace_between_characters() {
    let no_spaces = "ก".repeat(4500);
    let spaced = "ก ".repeat(4500);
    assert_eq!(estimate_minutes(&no_spaces), estimate_minutes(&spaced));
  }

  #[test]
  fn thai_with_no_spaces_beats_the_old_whitespace_token_formula() {
    let body = "ก".repeat(9000);
    let whitespace_tokens = body.split_whitespace().count();
    let old = ((whitespace_tokens as f64 / 200.0).ceil() as usize).max(1);

    assert_eq!(whitespace_tokens, 1, "the whole body is a single token");
    assert_eq!(old, 1, "the old formula saw a one-minute post");
    assert_eq!(estimate_minutes(&body), 10);
  }

  #[test]
  fn cjk_counts_characters_as_reading_units() {
    assert_eq!(estimate_minutes(&"漢".repeat(500)), 1);
    assert_eq!(estimate_minutes(&"漢".repeat(1000)), 2);
    assert_eq!(estimate_minutes(&"あ".repeat(500)), 1);
  }

  #[test]
  fn mixed_scripts_add_up() {
    let body = format!("{} {}", "word ".repeat(100), "ก".repeat(450));
    // 100 / 200 + 450 / 900 = 0.5 + 0.5 = 1 minute.
    assert_eq!(estimate_minutes(&body), 1);
  }

  #[test]
  fn empty_text_still_reads_as_one_minute() {
    assert_eq!(estimate_minutes(""), 1);
    assert_eq!(estimate_minutes("   \n\t"), 1);
  }

  #[test]
  fn markdown_syntax_and_punctuation_are_not_words() {
    assert_eq!(estimate_minutes("--- *** ### ---"), 1);
  }
}
