pub fn is_cjk(c: char) -> bool {
    match c {
        // CJK Unified Ideographs Extension A
        '\u{3400}'..='\u{4DBF}' |
        // CJK Unified Ideographs
        '\u{4E00}'..='\u{9FFF}' |
        // CJK Compatibility Ideographs
        '\u{F900}'..='\u{FAFF}' |
        // CJK Unified Ideographs Extension B
        '\u{20000}'..='\u{2A6DF}' |
        // CJK Unified Ideographs Extension C
        '\u{2A700}'..='\u{2B73F}' |
        // CJK Unified Ideographs Extension D
        '\u{2B740}'..='\u{2B81F}' |
        // CJK Unified Ideographs Extension E
        '\u{2B820}'..='\u{2CEAF}' |
        // CJK Compatibility Ideographs Supplement
        '\u{2F800}'..='\u{2FA1F}' => true,
        _ => false,
    }
}
