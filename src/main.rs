extern crate rayon;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate chrono;

use std::str;
use std::collections::BTreeMap;
use std::fs;
use std::time::Instant;
use std::thread;

use regex::Regex;

use git2::{Repository, Error, Diff, Time};
use docopt::Docopt;
use rayon::prelude::*;
use chrono::{DateTime, Utc, Datelike};
use chrono::offset::TimeZone;

#[derive(Debug, Deserialize, Serialize)]
struct Stats {
    num_commits_to_master: i32,
    num_prs: i32,
    num_files: i32,
    component_stats: BTreeMap<String, i32>,
    lang_stats: BTreeMap<String, i32>,
    commits_by_month: BTreeMap<String, Vec<i32>>
}

fn extract_pr_from_commit_message(commit_message: &str) -> Option<&str> {
    lazy_static! {
        static ref PR_RE: Regex = Regex::new(r"\(#(\d+)\)").unwrap();
    }
    PR_RE.captures(commit_message).map(|pr_number| {
        pr_number.get(1).unwrap().as_str()
    })
}

fn extract_component_name_from_diff_summary(filename: &str) -> Option<String> {
    lazy_static! {
        static ref CN_RE: Regex = Regex::new(r"([\w-]+)/.+").unwrap();
    }
    CN_RE.captures(&filename).map(|component_name| {
        component_name.get(1).unwrap().as_str().to_owned()
    })
}

fn extract_language_from_diff_summary(filename: &str) -> Option<String> {
    lazy_static! {
        static ref LANG_RE: Regex = Regex::new(r"\.(\w+)$").unwrap();
    }
    LANG_RE.captures(&filename).map(|lang_name| {
        lang_name.get(1).unwrap().as_str().to_owned()
    })
}

fn convert_git_time_to_datetime(git_time: &Time) -> DateTime<Utc> {
    Utc.timestamp(git_time.seconds() + i64::from(git_time.offset_minutes()) * 60, 0)
}

fn walk_entire_history(git_repo_path: &str) -> Result<Stats, Error> {
    let repo = Repository::open(git_repo_path)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let before_revwalk = Instant::now();

    let mut commits_by_month: BTreeMap<String, Vec<i32>> = BTreeMap::new();

    // create list of diffs we're interested in
    let diffs: Vec<Diff> = revwalk.map(|step| {
        let oid = step.unwrap();
        let commit = repo.find_commit(oid).unwrap();

        // record changes by time
        let dt = convert_git_time_to_datetime(&commit.author().when());
        let year = format!("{}", dt.year());
        let month = (dt.month() - 1) as usize;
        let month_vec = commits_by_month.entry(year).or_insert(vec![0; 12]);
        month_vec[month] += 1;

        let a = if commit.parents().len() == 1 {
            let parent = commit.parent(0).unwrap();
            Some(parent.tree().unwrap())
        } else {
            None
        };
        let b = commit.tree().unwrap();
        repo.diff_tree_to_tree(a.as_ref(), Some(&b), None).unwrap()
    }).collect();

    let after_revwalk = Instant::now();
    println!("Revwalk time: {}", after_revwalk.duration_since(before_revwalk).as_secs());

    let num_files = diffs.iter().map(|diff| {
        diff.deltas().len() as i32
    }).sum();

    let before_diffs = Instant::now();

    let mut diff_deltas = vec![];
    for diff in &diffs {
        let ds = diff.deltas();
        for d in ds {
            let file_name = d.new_file().path().unwrap().to_str().unwrap().to_owned();
            if !file_name.starts_with("master") {
                diff_deltas.push(file_name);
            }
        }
    }

    let after_diffs = Instant::now();
    println!("Creating diff vec: {}", after_diffs.duration_since(before_diffs).as_secs());

    // count the component_names and languages used
    let before_counts = Instant::now();
    let component_name_occurrences: Vec<String> = diff_deltas.par_iter().map(|file_name| {
        extract_component_name_from_diff_summary(&file_name).unwrap_or_else(|| "unknown".to_owned())
    }).collect();

    let lang_name_occurrences: Vec<String> = diff_deltas.par_iter().map(|file_name| {
        extract_language_from_diff_summary(&file_name).unwrap_or_else(|| "unknown".to_owned())
    }).collect();

    let comp_name_thread = thread::spawn(|| {
        let mut component_map: BTreeMap<String, i32> = BTreeMap::new();
        for comp_name in component_name_occurrences {
            *component_map.entry(comp_name).or_insert(0) += 1;
        }
        component_map
    });

    let lang_name_thread = thread::spawn( || {
        let mut lang_map: BTreeMap<String, i32> = BTreeMap::new();
        for lang_name in lang_name_occurrences {
            *lang_map.entry(lang_name).or_insert(0) += 1;
        }
        lang_map
    });

    let component_map = comp_name_thread.join().unwrap();
    let lang_map = lang_name_thread.join().unwrap();

    let after_counts = Instant::now();
    println!("Processing counts: {}", after_counts.duration_since(before_counts).as_secs());

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

    Ok(Stats{num_commits_to_master: num_commits, num_prs: num_prs, num_files: num_files, component_stats: component_map, lang_stats: lang_map, commits_by_month: commits_by_month})
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
        let r = walk_entire_history("/Users/jacksonke/projects/voyager");
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

    #[test]
    fn test_convert_git_time_to_datetime() {

    }
}