macro_rules! unicode {
    ($utf8: literal, $ascii: literal) => {
        if cfg!(feature = "unicode") {
            $utf8
        } else {
            $ascii
        }
    };
}

pub(crate) use unicode;
