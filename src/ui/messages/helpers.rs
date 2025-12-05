//! Helper utilities for message processing.

/// Check if a message contains a mention of the given nickname.
///
/// # Mention Detection Rules
///
/// A mention is detected when:
/// 1. The nickname appears as a complete word (not part of another word)
/// 2. Case-insensitive matching (alice matches ALICE, Alice, etc.)
/// 3. Common IRC mention formats: "nick:", "nick,", "@nick"
///
/// # Examples
///
/// ```ignore
/// contains_mention("Hey alice, how are you?", "alice") == true
/// contains_mention("Hey ALICE: check this out", "alice") == true
/// contains_mention("Hey @alice", "alice") == true
/// contains_mention("alice's message", "alice") == true  // apostrophe is word boundary
/// contains_mention("malice aforethought", "alice") == false  // alice is inside word
/// ```
pub(crate) fn contains_mention(text: &str, nickname: &str) -> bool {
    if nickname.is_empty() {
        return false;
    }

    let text_lower = text.to_lowercase();
    let nick_lower = nickname.to_lowercase();

    // Find all occurrences of the nickname
    let mut search_start = 0;
    while let Some(pos) = text_lower[search_start..].find(&nick_lower) {
        let abs_pos = search_start + pos;
        let end_pos = abs_pos + nick_lower.len();

        // Check if this is a word boundary match
        let at_start = abs_pos == 0
            || !text.as_bytes()[abs_pos - 1].is_ascii_alphanumeric()
            || (abs_pos > 0 && text.as_bytes()[abs_pos - 1] == b'@'); // @mention

        let at_end = end_pos >= text.len() || !text.as_bytes()[end_pos].is_ascii_alphanumeric();

        if at_start && at_end {
            return true;
        }

        // Move past this occurrence
        search_start = abs_pos + 1;
    }

    false
}

/// Maximum time gap (in seconds) before starting a new message group.
/// Messages from the same sender within 5 minutes are grouped together.
pub(crate) const GROUP_TIME_GAP_SECONDS: u32 = 300;

/// Parse a timestamp string "HH:MM:SS" into total seconds since midnight.
/// Returns None if parsing fails.
pub(crate) fn parse_timestamp_seconds(ts: &str) -> Option<u32> {
    let parts: Vec<&str> = ts.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let hours: u32 = parts[0].parse().ok()?;
    let minutes: u32 = parts[1].parse().ok()?;
    let seconds: u32 = parts[2].parse().ok()?;
    Some(hours * 3600 + minutes * 60 + seconds)
}

/// Check if two timestamps are within the grouping window (5 minutes).
/// Handles midnight wraparound.
pub(crate) fn timestamps_within_window(ts1: &str, ts2: &str) -> bool {
    let Some(secs1) = parse_timestamp_seconds(ts1) else {
        return false;
    };
    let Some(secs2) = parse_timestamp_seconds(ts2) else {
        return false;
    };

    // Calculate difference, handling midnight wraparound
    let diff = if secs2 >= secs1 {
        secs2 - secs1
    } else {
        // Midnight crossed: add 24 hours to second timestamp
        (secs2 + 86400) - secs1
    };

    diff <= GROUP_TIME_GAP_SECONDS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timestamp_seconds() {
        assert_eq!(parse_timestamp_seconds("00:00:00"), Some(0));
        assert_eq!(parse_timestamp_seconds("00:00:01"), Some(1));
        assert_eq!(parse_timestamp_seconds("00:01:00"), Some(60));
        assert_eq!(parse_timestamp_seconds("01:00:00"), Some(3600));
        assert_eq!(parse_timestamp_seconds("12:30:45"), Some(45045));
        assert_eq!(parse_timestamp_seconds("23:59:59"), Some(86399));

        // Invalid formats
        assert_eq!(parse_timestamp_seconds("invalid"), None);
        assert_eq!(parse_timestamp_seconds("12:30"), None);
        assert_eq!(parse_timestamp_seconds(""), None);
        assert_eq!(parse_timestamp_seconds("aa:bb:cc"), None);
    }

    #[test]
    fn test_timestamps_within_window() {
        // Same timestamp
        assert!(timestamps_within_window("12:00:00", "12:00:00"));

        // Within 5 minutes
        assert!(timestamps_within_window("12:00:00", "12:04:59"));
        assert!(timestamps_within_window("12:00:00", "12:05:00"));

        // Just outside 5 minutes
        assert!(!timestamps_within_window("12:00:00", "12:05:01"));
        assert!(!timestamps_within_window("12:00:00", "12:10:00"));

        // Test midnight wraparound
        assert!(timestamps_within_window("23:58:00", "00:01:00")); // 3 minutes across midnight

        // Invalid timestamps should return false
        assert!(!timestamps_within_window("invalid", "12:00:00"));
        assert!(!timestamps_within_window("12:00:00", "invalid"));
    }

    #[test]
    fn test_contains_mention_basic() {
        // Basic word boundary matches
        assert!(contains_mention("hey alice how are you", "alice"));
        assert!(contains_mention("alice: check this out", "alice"));
        assert!(contains_mention("hello alice!", "alice"));

        // Case insensitivity
        assert!(contains_mention("Hey ALICE, how are you?", "alice"));
        assert!(contains_mention("Hey Alice, how are you?", "alice"));
        assert!(contains_mention("hey alice, how are you?", "ALICE"));

        // @ mentions
        assert!(contains_mention("@alice check this out", "alice"));
        assert!(contains_mention("hey @alice", "alice"));
    }

    #[test]
    fn test_contains_mention_word_boundaries() {
        // Should NOT match when nick is inside another word
        assert!(!contains_mention("malice aforethought", "alice"));
        assert!(!contains_mention("bobcat is cute", "bob"));
        assert!(!contains_mention("jacoby is here", "jacob"));

        // Should match at start/end of text
        assert!(contains_mention("alice", "alice"));
        assert!(contains_mention("hi alice", "alice"));
        assert!(contains_mention("alice says hi", "alice"));

        // Should match with punctuation
        assert!(contains_mention("alice's message", "alice"));
        assert!(contains_mention("(alice)", "alice"));
        assert!(contains_mention("[alice]", "alice"));
    }

    #[test]
    fn test_contains_mention_edge_cases() {
        // Empty nickname should not match
        assert!(!contains_mention("hello world", ""));

        // Empty text should not match
        assert!(!contains_mention("", "alice"));

        // Single character nick
        assert!(contains_mention("hey x what's up", "x"));
        assert!(!contains_mention("hex is cool", "x")); // x inside word
    }
}
