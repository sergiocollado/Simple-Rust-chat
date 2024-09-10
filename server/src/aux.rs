pub fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();
    std::str::from_utf8(first_word_u8(&bytes)).expect("fn first_word: wrong conversion u8 -> str")
    // actually, it could be used the function: https://doc.rust-lang.org/std/primitive.slice.html#method.starts_with
}

pub fn first_word_u8(s: &[u8]) -> &[u8] {
    let mut i1: usize = 0;

    for (i, &item) in s.iter().enumerate() {
        if !(item == b' ' || item == b'\t' || item == b'\n' || item == b'\r') {
            i1 = i;
            break;
        }
    }

    let s2 = &s[i1..];

    for (i, &item) in s2.iter().enumerate() {
        if item == b' ' || item == b'\t' || item == b'\n' || item == b'\r' {
            let i2 = i1 + i;
            std::str::from_utf8(&s[i1..i2]).unwrap();
            return &s[i1..i2];
        }
    }

    &s[..]
}

pub fn first_2_words(s: &str) -> (Option<&str>, Option<&str>) {
    let mut iter = s.split_ascii_whitespace();
    let word1 = iter.next();
    let word2 = iter.next();
    (word1, word2)
}
