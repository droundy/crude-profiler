#![feature(test)]

extern crate test;
extern crate crude_profiler;

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn push(b: &mut Bencher) {
        b.iter(|| {
            crude_profiler::push("silly")
        });
        crude_profiler::clear();
    }

    #[bench]
    fn replace(b: &mut Bencher) {
        let _g = crude_profiler::push("silly");
        b.iter(|| {
            _g.replace("whatever");
        });
        crude_profiler::clear();
    }

    #[bench]
    fn report(b: &mut Bencher) {
        let _g = crude_profiler::push("silly");
        _g.replace("whatever");
        b.iter(|| {
            crude_profiler::report()
        });
    }
}
