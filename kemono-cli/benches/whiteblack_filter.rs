#![feature(test)]

extern crate test;

#[cfg(test)]
mod bench {
    use kemono_cli::utils::whiteblack_regex_filter;
    use regex::RegexSet;
    use test::Bencher;

    #[bench]
    fn filterout_psd(bench: &mut Bencher) {
        let white = RegexSet::empty();
        let black = RegexSet::new(["PSD"]).expect("failed to compile regex");
        let heytrack1 = "胡桃まんぐりっクス-高画質版";

        bench.iter(std::hint::black_box(|| {
            whiteblack_regex_filter(&white, &black, heytrack1)
        }));
    }
}
