#![feature(test)]
extern crate test;

#[cfg(test)]
mod tests {

    mod human_units {
        use human_units::FormatDuration;
        use human_units::FormatSize;
        use test::Bencher;

        use super::max_duration;
        use super::max_size;

        #[bench]
        fn format_size(b: &mut Bencher) {
            b.iter(|| test::black_box(max_size().format_size()));
        }

        #[bench]
        fn format_size_then_to_string(b: &mut Bencher) {
            b.iter(|| test::black_box(max_size().format_size().to_string()));
        }

        #[bench]
        fn format_duration(b: &mut Bencher) {
            b.iter(|| test::black_box(max_duration().format_duration()));
        }

        #[bench]
        fn format_duration_then_to_string(b: &mut Bencher) {
            b.iter(|| test::black_box(max_duration().format_duration().to_string()));
        }
    }

    mod human_bytes {
        use human_bytes::human_bytes;
        use test::Bencher;

        use super::max_size;

        #[bench]
        fn format_size_then_to_string(b: &mut Bencher) {
            b.iter(|| test::black_box(human_bytes(max_size() as f64)));
        }
    }

    mod human_repr {
        use human_repr::HumanCount;
        use human_repr::HumanDuration;
        use test::Bencher;

        use super::max_duration;
        use super::max_size;

        #[bench]
        fn format_duration_then_to_string(b: &mut Bencher) {
            b.iter(|| test::black_box(max_duration().human_duration().to_string()));
        }

        #[bench]
        fn format_size_then_to_string(b: &mut Bencher) {
            b.iter(|| test::black_box(max_size().human_count_bytes().to_string()));
        }
    }

    fn max_duration() -> core::time::Duration {
        test::black_box(core::time::Duration::new(u64::MAX, 999_999_999_u32))
    }

    fn max_size() -> u64 {
        test::black_box(u64::MAX)
    }
}
