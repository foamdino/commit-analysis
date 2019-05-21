extern crate rayon;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::str;
use std::collections::HashMap;
use std::fs;

use regex::Regex;

use git2::{Repository, Error, DiffFormat, DiffDelta, DiffHunk, DiffLine, DiffOptions, Diff, Deltas};
use docopt::Docopt;
use rayon::prelude::*;
use rayon::collections::*;
use std::collections::vec_deque::VecDeque;

#[derive(Debug, Deserialize, Serialize)]
struct Stats {
    num_commits_to_master: i32,
    num_prs: i32,
    num_files: i32,
    component_stats: HashMap<String, i32>,
    lang_stats: HashMap<String, i32>
}

fn extract_pr_from_commit_message(commit_message: &str) -> Option<&str> {
    lazy_static! {
        static ref PR_RE: Regex = Regex::new(r"\(#(\d+)\)").unwrap();
    }
    PR_RE.captures(commit_message).map(|pr_number| {
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

fn extract_language_from_diff_summary(filename: &String) -> Option<String> {
    lazy_static! {
        static ref LANG_RE: Regex = Regex::new(r"\.(\w+)$").unwrap();
    }
    LANG_RE.captures(&filename).map(|lang_name| {
        lang_name.get(1).unwrap().as_str().to_owned()
    })
}

fn walk_entire_history(git_repo_path: &str) -> Result<Stats, Error> {
    let repo = Repository::open(git_repo_path)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let mut lang_map: HashMap<String, i32> = HashMap::new();
    let mut component_map: HashMap<String, i32> = HashMap::new();

    // create list of diffs we're interested in
    let diffs: Vec<Diff> = revwalk.map(|step| {
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

    let num_files = diffs.iter().map(|diff| {
        diff.deltas().len() as i32
    }).sum();

    //TODO make this work...
//    let diff_deltas: Vec<DiffDelta> = diffs.iter().map(|diff| {
//       diff.deltas().filter(|dl| {!dl.new_file().path().unwrap().starts_with("master")})
//    }).collect();

//    diffs.iter().for_each(|diff| {
//        diff.deltas().filter(|p| { !p.new_file().path().unwrap().starts_with("master")}).for_each(|dd| {
//            let file_name = dd.new_file().path().unwrap().to_str().unwrap().to_owned();
//            println!("{}", file_name);
//            let cn = extract_component_name_from_diff_summary(&file_name).unwrap_or("unknown".to_owned());
//            let comp_count = component_map.entry(cn).or_insert(1);
//            *comp_count += 1;
//
//            let ln = extract_language_from_diff_summary(&file_name).unwrap_or("unknown".to_owned());
//            let lang_count = lang_map.entry(ln).or_insert(1);
//            *lang_count += 1;
//        });
//    });

//    for diff in diffs {
//        for dd in diff.deltas() {
//            let file_name = dd.new_file().path().unwrap().to_str().unwrap().to_owned();
//            let cn = extract_component_name_from_diff_summary(&file_name).unwrap_or("unknown".to_owned());
//            let comp_count = component_map.entry(cn).or_insert(1);
//            *comp_count += 1;
//
//            let ln = extract_language_from_diff_summary(&file_name).unwrap_or("unknown".to_owned());
//            let lang_count = lang_map.entry(ln).or_insert(1);
//            *lang_count += 1;
//        }
//    }

//    diffs.iter().map(|diff| {
//        diff.deltas().for_each(|dd| {
//            let file_name = dd.new_file().path().unwrap().to_str().unwrap().to_owned();
//            let cn = extract_component_name_from_diff_summary(&file_name).unwrap_or("unknown".to_owned());
//            let comp_count = component_map.entry(cn).or_insert(1);
//            *comp_count += 1;
//
//            let ln = extract_language_from_diff_summary(&file_name).unwrap_or("unknown".to_owned());
//            let lang_count = lang_map.entry(ln).or_insert(1);
//            *lang_count += 1;
//        })
//    });


    // count the component_names and languages used
//    for dd in diff_deltas {
//        let file_name = dd.new_file().path().unwrap().to_str().unwrap().to_owned();
//        let cn = extract_component_name_from_diff_summary(&file_name).unwrap_or("unknown".to_owned());
//        let comp_count = component_map.entry(cn).or_insert(1);
//        *comp_count += 1;
//
//        let ln = extract_language_from_diff_summary(&file_name).unwrap_or("unknown".to_owned());
//        let lang_count = lang_map.entry(ln).or_insert(1);
//        *lang_count += 1;
//    }


//    for r in revwalk {
//        let oid = r?;
//        let commit = repo.find_commit(oid)?;
////        println!("{:?}",commit);
////        println!("{}", commit.raw_header().unwrap());
//
//        let a = if commit.parents().len() == 1 {
//            let parent = commit.parent(0)?;
//            Some(parent.tree()?)
//        } else {
//            None
//        };
//        let b = commit.tree()?;
//        let diff = repo.diff_tree_to_tree(a.as_ref(), Some(&b), None)?;
//        num_files += diff.deltas().len() as i32;
//        for dd in diff.deltas() {
//            let file_name = dd.new_file().path().unwrap().to_str().unwrap().to_owned();
//            let cn = extract_component_name_from_diff_summary(&file_name).unwrap_or("unknown".to_owned());
//            let comp_count = component_map.entry(cn).or_insert(1);
//            *comp_count += 1;
//
//            let ln = extract_language_from_diff_summary(&file_name).unwrap_or("unknown".to_owned());
//            let lang_count = lang_map.entry(ln).or_insert(1);
//            *lang_count += 1;
//        }
//    }

    revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    let num_commits: i32 = revwalk.map(|step| {
        let oid = step.unwrap();
        match repo.find_commit(oid) {
            Ok(_c) => 1,
            Err(_) => 0
        }
    }).sum();

    revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    let num_prs= revwalk.map(|step| {
        let oid = step.unwrap();
        let commit = repo.find_commit(oid).unwrap();
        match extract_pr_from_commit_message(commit.summary().unwrap()) {
            Some(_pr_number) => 1,
            None => 0
        }
    }).sum();

    println!("total commits to master: {}", num_commits);
//    println!("{:?}", Stats{num_commits_to_master: num_commits, num_prs: num_prs, num_files: num_files, component_stats: HashMap::new(), lang_stats: HashMap::new()});

    Ok(Stats{num_commits_to_master: num_commits, num_prs: num_prs, num_files: num_files, component_stats: component_map, lang_stats: lang_map})
}

fn main() {
    const USAGE: &str = "
Usage: commit-analysis <git_repo_path>
";

    let args = Docopt::new(USAGE)
        .and_then(|d| d.parse())
        .unwrap_or_else(|e| e.exit());

    let output_file ="/tmp/commit-analysis.json";
    let analysis = walk_entire_history(args.get_str("<git_repo_path>"));
    let json = serde_json::to_string(&analysis.unwrap());
    fs::write(&output_file, json.unwrap()).unwrap_or_else(|_| panic!("couldn't write to file: {}", &output_file));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_walk_entire_history() {
        let r = walk_entire_history("/Users/kevj/projects/voyager");
        assert!(r.is_ok());
//        let num_commits = r.unwrap().num_commits_to_master;
//        assert!(num_commits > 0);
        print!("{:?}", r.unwrap());
    }

    #[test]
    fn test_extract_component_name_from_diff_summary() {
        let r = extract_component_name_from_diff_summary(&"voyager-workflow-manager/src/test/java/com/thehutgroup/voyager/workflow/domain/kitting/RestKittingRegistrationServiceTest.java".to_owned());
        assert!(r.is_some());
        assert_eq!(r.unwrap(), "voyager-workflow-manager");
    }

    #[test]
    fn test_extract_pr_from_commit_message() {
        let message = "VGR-8087 - Adding tests for verifying required products service is decremented (#4729)";
        let pr_number = extract_pr_from_commit_message(message);
        assert_eq!("4729", pr_number.unwrap());
    }

    #[test]
    fn test_extract_language_from_diff_summary() {
        let r = extract_language_from_diff_summary(&"voyager-workflow-manager/src/test/java/com/thehutgroup/voyager/workflow/domain/kitting/RestKittingRegistrationServiceTest.java".to_owned());
        assert!(r.is_some());
        assert_eq!(r.unwrap(), "java");
    }
}