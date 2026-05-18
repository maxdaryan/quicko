//! Seed phrase generation and QuickoKey recovery.
//!
//! Uses a simple scheme: 256-word list, each word = 1 byte (8 bits).
//! 128 bits = 16 words. Deterministic and lossless roundtrip.

use rand::rngs::OsRng;
use rand::RngCore;

use crate::error::{QuickoError, Result};
use super::key::QuickoKey;

/// Embedded wordlist — 256 common English words.
/// Each word encodes exactly 8 bits (1 byte), so 16 words = 128 bits.
const WORDLIST: &[&str] = &[
    "abandon", "ability", "able", "about", "above", "absent", "absorb", "abstract",
    "absurd", "abuse", "access", "accident", "account", "accuse", "achieve", "acid",
    "across", "act", "action", "actor", "address", "adjust", "admit", "adult",
    "advance", "advice", "afraid", "again", "age", "agent", "agree", "ahead",
    "aim", "air", "airport", "aisle", "alarm", "album", "alert", "alien",
    "allow", "almost", "alone", "alpha", "already", "also", "alter", "always",
    "among", "amount", "amused", "anchor", "ancient", "anger", "angle", "animal",
    "annual", "answer", "antenna", "antique", "anxiety", "apart", "apology", "appear",
    "apple", "approve", "april", "arch", "arctic", "arena", "argue", "armor",
    "army", "arrange", "arrest", "arrive", "arrow", "artist", "asset", "assist",
    "assume", "attack", "attend", "attract", "auction", "audit", "august", "aunt",
    "author", "auto", "autumn", "average", "avocado", "avoid", "awake", "aware",
    "balance", "ball", "bamboo", "banana", "banner", "barely", "bargain", "barrel",
    "basket", "battle", "beach", "bean", "beauty", "because", "become", "beef",
    "before", "begin", "behind", "believe", "below", "bench", "benefit", "best",
    "betray", "between", "beyond", "bicycle", "bird", "birth", "bitter", "blade",
    "blame", "blanket", "blast", "bleak", "bless", "blind", "blood", "blossom",
    "blue", "blur", "blush", "board", "boat", "body", "boil", "bomb",
    "bone", "bonus", "book", "boost", "border", "boring", "borrow", "boss",
    "bottom", "bounce", "box", "bracket", "brain", "brand", "brass", "brave",
    "bread", "breeze", "brick", "bridge", "bright", "bring", "brisk", "broken",
    "bronze", "broom", "brother", "brown", "brush", "bubble", "buddy", "budget",
    "buffalo", "build", "bulk", "bullet", "bundle", "burger", "burst", "bus",
    "busy", "butter", "buyer", "cabin", "cable", "cactus", "cage", "cake",
    "call", "calm", "camera", "camp", "canal", "cancel", "candy", "cannon",
    "canoe", "canvas", "canyon", "capable", "capital", "captain", "carbon", "card",
    "cargo", "carpet", "carry", "cart", "case", "cash", "castle", "catalog",
    "catch", "cattle", "cause", "caution", "cave", "ceiling", "celery", "cement",
    "census", "century", "cereal", "certain", "chair", "chalk", "champion", "change",
    "chaos", "chapter", "charge", "chase", "cheap", "check", "cheese", "cherry",
    "chicken", "chief", "child", "chimney", "choice", "chunk", "circle", "citizen",
    "claim", "clap", "clarify", "claw", "clean", "clerk", "clever", "climb",
];

/// Generate a 16-word seed phrase from 128 bits of OS randomness.
///
/// Each byte of entropy maps to one word (256-word list).
/// The seed phrase can recover the QuickoKey at any time.
pub fn generate_seed_phrase() -> (Vec<String>, QuickoKey) {
    let mut entropy = [0u8; 16]; // 128 bits
    OsRng.fill_bytes(&mut entropy);

    let words = entropy_to_words(&entropy);
    let key = QuickoKey::from_bytes(entropy);

    (words, key)
}

/// Convert 16 bytes of entropy into 16 words (1 byte per word).
fn entropy_to_words(entropy: &[u8; 16]) -> Vec<String> {
    entropy
        .iter()
        .map(|&byte| WORDLIST[byte as usize].to_string())
        .collect()
}

/// Recover a QuickoKey from a 16-word seed phrase.
pub fn seed_phrase_to_key(words: &[&str]) -> Result<QuickoKey> {
    if words.len() != 16 {
        return Err(QuickoError::InvalidQuickoKey(format!(
            "Seed phrase must be 16 words, got {}",
            words.len()
        )));
    }

    let mut entropy = [0u8; 16];
    for (i, word) in words.iter().enumerate() {
        let word_lower = word.to_lowercase();
        let idx = WORDLIST
            .iter()
            .position(|w| *w == word_lower)
            .ok_or_else(|| {
                QuickoError::InvalidQuickoKey(format!("Unknown word: '{}'", word))
            })?;
        entropy[i] = idx as u8;
    }

    Ok(QuickoKey::from_bytes(entropy))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_seed_phrase_length() {
        let (words, _key) = generate_seed_phrase();
        assert_eq!(words.len(), 16);
        for word in &words {
            assert!(!word.is_empty());
        }
    }

    #[test]
    fn test_seed_phrase_roundtrip() {
        let (words, original_key) = generate_seed_phrase();
        let word_refs: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
        let recovered = seed_phrase_to_key(&word_refs).unwrap();
        assert_eq!(original_key, recovered);
    }

    #[test]
    fn test_deterministic_recovery() {
        let (words, key1) = generate_seed_phrase();
        let word_refs: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
        let key2 = seed_phrase_to_key(&word_refs).unwrap();
        let key3 = seed_phrase_to_key(&word_refs).unwrap();
        assert_eq!(key1, key2);
        assert_eq!(key2, key3);
    }

    #[test]
    fn test_wrong_word_count() {
        let result = seed_phrase_to_key(&["hello", "world"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_word() {
        let words = vec!["zzzznotaword"; 16];
        let result = seed_phrase_to_key(&words);
        assert!(result.is_err());
    }

    #[test]
    fn test_known_entropy() {
        // Byte 0 = "abandon", byte 1 = "ability", etc.
        let entropy = [0u8; 16]; // All zeros → 16× "abandon"
        let words = entropy_to_words(&entropy);
        assert!(words.iter().all(|w| w == "abandon"));

        let word_refs: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
        let key = seed_phrase_to_key(&word_refs).unwrap();
        assert_eq!(*key.as_bytes(), [0u8; 16]);
    }
}
