//! gungraun benchmarks for the CS-D5 measurement. Inputs are built in
//! `setup` functions, so only the kernel is counted.

use std::hint::black_box;

use cs_d5_bench as k;
use gungraun::prelude::*;

#[library_benchmark]
#[bench::stylesheet(setup = k::make_css, args = (2000))]
fn css_tokenize(css: String) -> u64 {
    black_box(k::css_tokenize(black_box(&css)))
}

#[library_benchmark]
#[bench::dom2000_sel200(setup = k::make_match_input)]
fn selector_match(input: (Vec<k::Elem>, Vec<k::Selector>)) -> u64 {
    black_box(k::selector_match(black_box(&input.0), black_box(&input.1)))
}

#[library_benchmark]
#[bench::urls(setup = k::make_urls, args = (2000))]
fn url_parse(urls: Vec<String>) -> u64 {
    black_box(k::url_parse(black_box(&urls)))
}

#[library_benchmark]
#[bench::document(setup = k::make_html, args = (1000))]
fn html_tokenize(html: String) -> u64 {
    black_box(k::html_tokenize(black_box(&html)))
}

#[library_benchmark]
#[bench::keys5000(setup = k::make_keys, args = (5000))]
fn hashmap_random(keys: Vec<String>) -> u64 {
    black_box(k::hashmap_random(black_box(&keys)))
}

#[library_benchmark]
#[bench::keys5000(setup = k::make_keys, args = (5000))]
fn hashmap_fixed(keys: Vec<String>) -> u64 {
    black_box(k::hashmap_fixed(black_box(&keys)))
}

library_benchmark_group!(name = hashing, benchmarks = [hashmap_random, hashmap_fixed]);

library_benchmark_group!(
    name = kernels,
    benchmarks = [css_tokenize, selector_match, url_parse, html_tokenize]
);

main!(library_benchmark_groups = kernels, hashing);
