use hypher::{hyphenate, Lang};

pub fn buttify_sentence(sentence: &str) -> Option<String> {
    let mut words: Vec<&str> = sentence.split_ascii_whitespace().collect();

    let len = words.len();
    let target_index = (rand::random::<f32>() * len as f32).floor() as usize;

    let buttified = random_buttify(words.get(target_index)?)?;

    *(words.get_mut(target_index)?) = buttified.as_str();

    Some(words.join(" "))
}

/// Buttify a given word by replacing a random syllable with butt.
pub fn random_buttify(word: &str) -> Option<String> {
    let mut syllables = hyphenate(word, Lang::English)
        .map(ToString::to_string)
        .collect::<Vec<String>>();

    let target_syllable = (rand::random::<f32>() * syllables.len() as f32).floor() as usize;

    if let Some(butt_target) = syllables.get_mut(target_syllable) {
        *butt_target = "butt".to_string();
    }

    let word = syllables.join("");

    Some(word)
}

/// Splits a word into (mostly) correct syllable boundaries.
/// Most simple cases are fine, such as "banana" -> "ba", "na", "na"
/// and "moon" -> "moon"
pub fn syllables(word: &str) -> Vec<&str> {
    let mut syllables = Vec::new();
    let mut last_syl_bound = 0;
    let mut was_last_vow = false;

    for (i, ch) in word.chars().enumerate() {
        if was_last_vow && !is_vowel(ch) {
            // this is the end of a syllable
            if word.len() - i < 2 {
                // we're close enough to the end of the word, so
                // we treat the rest of the word as a single syllable
                break;
            }
            syllables.push(&word[last_syl_bound..i]);
            last_syl_bound = i;
            was_last_vow = false;
        } else {
            was_last_vow = is_vowel(ch);
        }
    }
    syllables.push(&word[last_syl_bound..(word.len())]);

    syllables
}

/// Returns if the vowel is a vowel or not.
/// 'y' will be treated as a consonent.
pub fn is_vowel(x: char) -> bool {
    match x.to_ascii_lowercase() {
        'a' | 'e' | 'i' | 'o' | 'u' => true,
        _ => false,
    }
}

#[test]
fn syllables_test() {
    for (word, expected) in [
        ("banana", vec!["ba", "na", "na"]),
        ("moon", vec!["moon"]),
        ("lemon", vec!["le", "mon"]),
    ] {
        assert_eq!(syllables(word).to_vec(), expected);
    }
}

#[test]
fn wont_panic_on_0_len_str() {
    syllables("");
}
