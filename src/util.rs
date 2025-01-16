use vidyut_lipi::{transliterate, Mapping, Scheme};

pub(crate) fn dev(x: impl AsRef<str>) -> String {
    transliterate(x.as_ref(), &Mapping::new(Scheme::Slp1, Scheme::Devanagari))
}

pub(crate) fn slp(x: impl AsRef<str>) -> String {
    transliterate(x.as_ref(),  &Mapping::new(Scheme::Devanagari, Scheme::Slp1))
}
