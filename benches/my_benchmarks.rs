#[macro_use]
extern crate criterion;

#[macro_use]
extern crate lazy_static;
extern crate serde_derive;

extern crate rayon;

use regex::Regex;
use criterion::Criterion;
use criterion::black_box;
use std::collections::HashMap;
use std::thread;
use git2::{Repository, Diff};

use rayon::prelude::*;

use commit_analysis::CommitChanges;

fn extract_pr_from_commit_message_alternative(commit_message: &str) -> Option<&str> {
    let pr_re: Regex = Regex::new(r"\(#(\d+)\)").unwrap();

    pr_re.captures(commit_message).map(|pr_number| {
        pr_number.get(1).unwrap().as_str()
    })
}

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

fn extract_names_and_sum(diff_files: Vec<String>) {
    let component_name_occurrences: Vec<String> = diff_files.par_iter().map(|file_name| {
        extract_component_name_from_diff_summary(&file_name).unwrap_or_else(|| "unknown".to_owned())
    }).collect();

    let comp_name_thread = thread::spawn(|| {
        let mut component_map: HashMap<String, u32> = HashMap::new();
        for comp_name in component_name_occurrences {
            *component_map.entry(comp_name).or_insert(0) += 1;
        }
        component_map
    });

    comp_name_thread.join();
}

fn extract_names_and_sum2(diff_files: Vec<String>) {
    let comp_name_thread = thread::spawn(|| {
        let mut component_map: HashMap<String, u32> = HashMap::new();

        for diff_file in diff_files {
            let comp_name = extract_component_name_from_diff_summary(&diff_file).unwrap_or_else(|| "unknown".to_owned());
            *component_map.entry(comp_name).or_insert(0) += 1;
        }
        component_map
    });

    comp_name_thread.join();
}

fn manipulate_hashmap(mm: &mut HashMap<String, u32>, cn: String) {
    let comp_count = mm.entry(cn).or_insert(1);
    *comp_count += 1;
}

fn bench_extract_component_name_from_diff_summary(c: &mut Criterion) {
    c.bench_function("extract component name",
                     |b| b.iter(|| extract_component_name_from_diff_summary(black_box(&"component-a/src/test/java/Thing.java".to_owned()))));
}

fn bench_extract_component_name_from_diff_summary2(c: &mut Criterion) {
    c.bench_function("extract component name2",
                     |b| b.iter(|| extract_component_name_from_diff_summary2(black_box(&"component-a/src/test/java/Thing.java".to_owned()))));
}

fn bench_extract_language_from_diff_summary(c: &mut Criterion) {
    c.bench_function("extract language",
                     |b| b.iter(|| extract_language_from_diff_summary(black_box(&"component-a/src/test/java/Thing.java".to_owned()))));
}

fn bench_extract_language_from_diff_summary2(c: &mut Criterion) {
    c.bench_function("extract language2",
                     |b| b.iter(|| extract_language_from_diff_summary2(black_box(&"component-a/src/test/java/Thing.java".to_owned()))));
}

fn bench_hashmap(c: &mut Criterion) {
    let mut mm: HashMap<String, u32> = HashMap::new();
    c.bench_function("hashmap",
                        move |b| b.iter(|| manipulate_hashmap(&mut mm, black_box("component-b".to_owned()))));

}

fn bench_extract_changes(c: &mut Criterion) {
    c.bench_function("extract_changes",
                     move |b| b.iter(|| extract_changes()));

}

fn bench_revwalk(c: &mut Criterion) {
    c.bench_function("revwalk",
                     move |b| b.iter(|| revwalk()));
}

fn bench_pr_from_commit_message_alternative(c: &mut Criterion) {
    c.bench_function("extract pr from commit message without lazy_static",
                     |b| b.iter(|| extract_pr_from_commit_message_alternative(black_box(&"VGR-XXXX - Lorem ipsum dolor sit amet, consectetur adipiscing elit. (#4729)"))));
}

fn bench_pr_from_commit_message(c: &mut Criterion) {
    c.bench_function("extract pr from commit message with lazy_static",
                     |b| b.iter(|| commit_analysis::extract_pr_from_commit_message(black_box(&"VGR-XXXX - Lorem ipsum dolor sit amet, consectetur adipiscing elit. (#4729)"))));
}

fn bench_extract_names_and_sum_v1(c: &mut Criterion) {
    let mut diff_files = vec![];
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

    for diff in diffs {
        let deltas = diff.deltas();
        for d in deltas {
            let file_name = d.new_file().path().unwrap().to_str().unwrap().to_owned();
            if !file_name.starts_with("master") {
                diff_files.push(file_name);
            }
        }
    }

    c.bench_function("extract names and sum v1",
    move |b| b.iter(|| extract_names_and_sum(diff_files.clone())));
}

fn bench_extract_names_and_sum_v2(c: &mut Criterion) {
    let mut diff_files = vec![];
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

    for diff in diffs {
        let deltas = diff.deltas();
        for d in deltas {
            let file_name = d.new_file().path().unwrap().to_str().unwrap().to_owned();
            if !file_name.starts_with("master") {
                diff_files.push(file_name);
            }
        }
    }

    c.bench_function("extract names and sum v2",
                     move |b| b.iter(|| extract_names_and_sum2(diff_files.clone())));
}

criterion_group!(benches,
    bench_extract_component_name_from_diff_summary,
    bench_extract_component_name_from_diff_summary2,
    bench_hashmap,
    bench_extract_changes,
    bench_revwalk,
    bench_extract_language_from_diff_summary,
    bench_extract_language_from_diff_summary2,
    bench_pr_from_commit_message_alternative,
    bench_pr_from_commit_message,
    bench_extract_names_and_sum_v1,
    bench_extract_names_and_sum_v2);

criterion_main!(benches);