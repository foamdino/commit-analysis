#[macro_use]
extern crate criterion;

#[macro_use]
extern crate lazy_static;

use regex::Regex;
use criterion::Criterion;
use criterion::black_box;
use std::collections::HashMap;

fn extract_component_name_from_diff_summary(filename: &String) -> Option<String> {
    lazy_static! {
        static ref CN_RE: Regex = Regex::new(r"([\w-]+)/.+").unwrap();
    }
    CN_RE.captures(&filename).map(|component_name| {
        component_name.get(1).unwrap().as_str().to_owned()
    })
}

fn manipulate_hashmap(mm: &mut HashMap<String, i32>, cn: String) {
    let comp_count = mm.entry(cn).or_insert(1);
    *comp_count += 1;
}

fn bench_extract_component_name_from_diff_summary(c: &mut Criterion) {
    c.bench_function("extract component name",
                     |b| b.iter(|| extract_component_name_from_diff_summary(black_box(&"voyager-workflow-manager/src/test/java/com/thehutgroup/voyager/workflow/domain/kitting/RestKittingRegistrationServiceTest.java".to_owned()))));
}

fn bench_hashmap(c: &mut Criterion) {
    let mut mm: HashMap<String, i32> = HashMap::new();
    c.bench_function("hashmap",
                        move |b| b.iter(|| manipulate_hashmap(&mut mm, black_box("voyager-stock-management".to_owned()))));

}

criterion_group!(benches, bench_extract_component_name_from_diff_summary, bench_hashmap);
criterion_main!(benches);