#[macro_use]
extern crate criterion;

#[macro_use]
extern crate lazy_static;
extern crate serde_derive;

use regex::Regex;
use criterion::Criterion;
use criterion::black_box;
use std::collections::HashMap;
use std::ops::{Add, AddAssign};
use git2::{Repository, Diff};

use commit_analysis::CommitChanges;

fn extract_component_name_from_diff_summary(filename: &String) -> Option<String> {
    lazy_static! {
        static ref CN_RE: Regex = Regex::new(r"([\w-]+)/.+").unwrap();
    }
    CN_RE.captures(&filename).map(|component_name| {
        component_name.get(1).unwrap().as_str().to_owned()
    })
}

fn extract_component_name_from_diff_summary2(filename: &str) -> Option<String> {
    let name_parts = filename.split("/").collect::<Vec<&str>>();
    Some(name_parts[0].to_owned())
}

fn extract_language_from_diff_summary(filename: &str) -> Option<String> {
    lazy_static! {
        static ref LANG_RE: Regex = Regex::new(r"\.(\w+)$").unwrap();
    }
    LANG_RE.captures(&filename).map(|lang_name| {
        lang_name.get(1).unwrap().as_str().to_owned()
    })
}

fn extract_language_from_diff_summary2(filename: &str) -> Option<String> {
    let name_parts = filename.split(".").collect::<Vec<&str>>();
    Some(name_parts[name_parts.len() -1].to_owned())
}

fn extract_changes() -> CommitChanges {
    let repo = Repository::open(".").unwrap();
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push_head().unwrap();
    let diffs: Vec<Diff> = revwalk.map( |step| {
        let oid = step.unwrap();
        let commit = repo.find_commit(oid).unwrap();
        let a = if commit.parents().len() == 1 {
            let parent = commit.parent(0).unwrap();
            Some(parent.tree().unwrap())
        } else {
            None
        };
        let b = commit.tree().unwrap();
        let diff = repo.diff_tree_to_tree(a.as_ref(), Some(&b), None).unwrap();
        diff
    }).collect();
    let stats = diffs.first().unwrap().stats().unwrap();
    CommitChanges{num_additions: stats.insertions() as i32, num_deletions: stats.deletions() as i32}
}

fn revwalk() {
    let repo = Repository::open(".").unwrap();
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push_head().unwrap();
    let diffs: Vec<Diff> = revwalk.map( |step| {
        let oid = step.unwrap();
        let commit = repo.find_commit(oid).unwrap();
        let a = if commit.parents().len() == 1 {
            let parent = commit.parent(0).unwrap();
            Some(parent.tree().unwrap())
        } else {
            None
        };
        let b = commit.tree().unwrap();
        let diff = repo.diff_tree_to_tree(a.as_ref(), Some(&b), None).unwrap();
        diff
    }).collect();
    let d = diffs.first().unwrap();
}

fn manipulate_hashmap(mm: &mut HashMap<String, i32>, cn: String) {
    let comp_count = mm.entry(cn).or_insert(1);
    *comp_count += 1;
}

fn bench_extract_component_name_from_diff_summary(c: &mut Criterion) {
    c.bench_function("extract component name",
                     |b| b.iter(|| extract_component_name_from_diff_summary(black_box(&"voyager-workflow-manager/src/test/java/com/thehutgroup/voyager/workflow/domain/kitting/RestKittingRegistrationServiceTest.java".to_owned()))));
}

fn bench_extract_component_name_from_diff_summary2(c: &mut Criterion) {
    c.bench_function("extract component name2",
                     |b| b.iter(|| extract_component_name_from_diff_summary2(black_box(&"voyager-workflow-manager/src/test/java/com/thehutgroup/voyager/workflow/domain/kitting/RestKittingRegistrationServiceTest.java".to_owned()))));
}

fn bench_extract_language_from_diff_summary(c: &mut Criterion) {
    c.bench_function("extract language",
                     |b| b.iter(|| extract_language_from_diff_summary(black_box(&"voyager-workflow-manager/src/test/java/com/thehutgroup/voyager/workflow/domain/kitting/RestKittingRegistrationServiceTest.java".to_owned()))));
}

fn bench_extract_language_from_diff_summary2(c: &mut Criterion) {
    c.bench_function("extract language2",
                     |b| b.iter(|| extract_language_from_diff_summary2(black_box(&"voyager-workflow-manager/src/test/java/com/thehutgroup/voyager/workflow/domain/kitting/RestKittingRegistrationServiceTest.java".to_owned()))));
}

fn bench_hashmap(c: &mut Criterion) {
    let mut mm: HashMap<String, i32> = HashMap::new();
    c.bench_function("hashmap",
                        move |b| b.iter(|| manipulate_hashmap(&mut mm, black_box("voyager-stock-management".to_owned()))));

}

fn bench_extract_changes(c: &mut Criterion) {
    c.bench_function("extract_changes",
                     move |b| b.iter(|| extract_changes()));

}

fn bench_revwalk(c: &mut Criterion) {
    c.bench_function("revwalk",
                     move |b| b.iter(|| revwalk()));
}


criterion_group!(benches, bench_extract_component_name_from_diff_summary, bench_extract_component_name_from_diff_summary2, bench_hashmap, bench_extract_changes, bench_revwalk, bench_extract_language_from_diff_summary, bench_extract_language_from_diff_summary2);
criterion_main!(benches);