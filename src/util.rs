use vidyut_lipi::{transliterate, Scheme};

fn dev(x: impl AsRef<str>) -> String {
    transliterate(x.as_ref(), Scheme::Slp1, Scheme::Devanagari)
}

fn slp(x: impl AsRef<str>) -> String {
    transliterate(x.as_ref(), Scheme::Devanagari, Scheme::Slp1)
}
