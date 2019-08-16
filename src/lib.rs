extern crate rayon;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate chrono;

use std::str;
use std::collections::BTreeMap;
use std::time::Instant;
use std::thread;
use std::ops::{Add, AddAssign};

use regex::Regex;

use git2::{Repository, Error, Diff, Time};
use rayon::prelude::*;
use chrono::{DateTime, Utc, Datelike};
use chrono::offset::TimeZone;

#[derive(Debug, Deserialize, Serialize)]
pub struct Stats {
    num_commits_to_master: i32,
    num_prs: i32,
    num_files: i32,
    component_stats: BTreeMap<String, i32>,
    lang_stats: BTreeMap<String, i32>,
    commits_by_month: BTreeMap<String, Vec<i32>>,
    commits_by_day_of_week: BTreeMap<String, i32>,
    filtered_languages: BTreeMap<String, i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CommitChanges {
    num_additions: i32,
    num_deletions: i32,
}

impl CommitChanges {

    pub fn new(adds: i32, dels: i32) -> CommitChanges {
        CommitChanges{num_additions: adds, num_deletions: dels}
    }

    pub fn get_num_deletions(&self) -> &i32 {
        &self.num_deletions
    }
}

impl Add for CommitChanges {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            num_additions: self.num_additions + other.num_additions,
            num_deletions: self.num_deletions + other.num_deletions
        }
    }
}

impl AddAssign for CommitChanges {
    fn add_assign(&mut self, other: CommitChanges) {
        *self = CommitChanges {
            num_additions: self.num_additions + other.num_additions,
            num_deletions: self.num_deletions + other.num_deletions
        }
    }
}

pub fn extract_pr_from_commit_message(commit_message: &str) -> Option<&str> {
    lazy_static! {
        static ref PR_RE: Regex = Regex::new(r"\(#(\d+)\)").unwrap();
    }
    PR_RE.captures(commit_message).map(|pr_number| {
        pr_number.get(1).unwrap().as_str()
    })
}

pub fn extract_component_name_from_diff_summary(filename: &str) -> Option<String> {
    let name_parts = filename.split("/").collect::<Vec<&str>>();
    Some(name_parts[0].to_owned())
}

pub fn extract_language_from_diff_summary(filename: &str) -> Option<String> {
    lazy_static! {
        static ref LANG_RE: Regex = Regex::new(r"\.(\w+)$").unwrap();
    }
    LANG_RE.captures(&filename).map(|lang_name| {
        lang_name.get(1).unwrap().as_str().to_owned()
    })
}

pub fn convert_git_time_to_datetime(git_time: &Time) -> DateTime<Utc> {
    Utc.timestamp(git_time.seconds() + i64::from(git_time.offset_minutes()) * 60, 0)
}

// too slow to be useful :-(
pub fn extract_changes(diff: &Diff) -> CommitChanges {
    let stats = diff.stats().unwrap();
    CommitChanges{num_additions: stats.insertions() as i32, num_deletions: stats.deletions() as i32}
}

pub fn walk_entire_history(git_repo_path: &str) -> Result<Stats, Error> {
    let repo = Repository::open(git_repo_path)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let before_revwalk = Instant::now();

    let mut commits_by_month: BTreeMap<String, Vec<i32>> = BTreeMap::new();
    let mut commits_by_day_of_week: BTreeMap<String, i32> = BTreeMap::new();
    let mut changes_by_component: BTreeMap<String, CommitChanges> = BTreeMap::new();
    let mut diff_files = vec![];

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
        let commit_day = format!("{:?}", dt.weekday());
        *commits_by_day_of_week.entry(commit_day).or_insert(0) += 1;

        let a = if commit.parents().len() == 1 {
            let parent = commit.parent(0).unwrap();
            Some(parent.tree().unwrap())
        } else {
            None
        };
        let b = commit.tree().unwrap();
        let diff = repo.diff_tree_to_tree(a.as_ref(), Some(&b), None).unwrap();
        let ds = diff.deltas();
        for d in ds {
            let file_name = d.new_file().path().unwrap().to_str().unwrap().to_owned();
//            let changes = extract_changes(&diff);
//            *changes_by_component.entry(comp_name).or_insert(CommitChanges{num_additions: 0, num_deletions: 0}) += changes;

            if !file_name.starts_with("master") {
                diff_files.push(file_name);
            }
        }

        diff
    }).collect();

    let after_revwalk = Instant::now();
    println!("Revwalk time: {}", after_revwalk.duration_since(before_revwalk).as_secs());

    let num_files = diffs.iter().map(|diff| {
        diff.deltas().len() as i32
    }).sum();

    // count the component_names and languages used
    let before_counts = Instant::now();
    let component_name_occurrences: Vec<String> = diff_files.par_iter().map(|file_name| {
        extract_component_name_from_diff_summary(&file_name).unwrap_or_else(|| "unknown".to_owned())
    }).collect();

    let lang_name_occurrences: Vec<String> = diff_files.par_iter().map(|file_name| {
        extract_language_from_diff_summary(&file_name).unwrap_or_else(|| "unknown".to_owned())
    }).collect();
    let lang_occurrences = lang_name_occurrences.clone();

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

    let filtered_languages_thread = thread::spawn(  || {
        let core_langs = vec!["java", "js", "py", "scala", "yaml", "yml", "clj", "jinja", "jinja2", "gradle", "kt", "conf", "xml", "xsd"];
        let mut lang_map: BTreeMap<String, i32> = BTreeMap::new();
        for lang_name in lang_occurrences {
            if core_langs.contains(&lang_name.as_str()) {
                *lang_map.entry(lang_name).or_insert(0) += 1;
            }
        }
        lang_map
    });

    let component_map = comp_name_thread.join().unwrap();
    let lang_map = lang_name_thread.join().unwrap();
    let filtered_languages = filtered_languages_thread.join().unwrap();

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

    Ok(Stats{
        num_commits_to_master: num_commits,
        num_prs: num_prs,
        num_files: num_files,
        component_stats: component_map,
        lang_stats: lang_map,
        filtered_languages: filtered_languages,
        commits_by_month: commits_by_month,
        commits_by_day_of_week: commits_by_day_of_week
    })
}