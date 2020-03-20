/*
   Copyright 2019-2020 foamdino@gmail.com

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate chrono;

use std::str;
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use std::thread;
use std::hash::Hash;
use std::cmp::Eq;
use git2::{Repository, Error, Time, Delta};
use chrono::{DateTime, Utc, Datelike};
use chrono::offset::TimeZone;

mod commit_changes;
pub use commit_changes::CommitChanges;

mod stats;
pub use stats::Stats;

const PATH_SPLIT: &str = "/";
const EXT_SPLIT: &str = ".";
const COMMIT_MESSAGE_START: &str = "(#";
const EMPTY_CHANGES: CommitChanges = CommitChanges::new(0 as u32, 0 as u32, 0 as u32);
const INTERESTING_LANGS: [&str; 15] = ["java", "js", "css", "clj", "scala", "kt", "groovy", "j2", "properties", "sh", "xsd", "xml", "yaml", "yml", "py"];

pub trait CountBy : Iterator {
    fn count_by_key<K, V, FA>(self, f: FA) -> HashMap<K, u32>
        where Self: Sized + Iterator<Item=V>,
              K: Hash + Eq,
              FA: Fn(&V) -> K
    {
        self.map(|v| (f(&v), v)).count_by()
    }

    fn count_by<K, V>(self) -> HashMap<K, u32>
        where Self: Sized + Iterator<Item=(K, V)>,
              K: Hash + Eq
    {
        let mut map = HashMap::new();

        for (key, _val) in self {
            *map.entry(key).or_insert(0) += 1;
        }

        map
    }
}

impl<T: Iterator> CountBy for T {}

pub fn extract_component_name_from_filename(filename: &str) -> Option<String> {
    let name_parts = filename.split(PATH_SPLIT).collect::<Vec<&str>>();
    Some(name_parts[0].to_owned())
}

pub fn extract_language_from_filename(filename: &str) -> Option<String> {
    let name_parts = filename.split(EXT_SPLIT).collect::<Vec<&str>>();
    let candidate = name_parts[name_parts.len() - 1].to_owned();
    if !candidate.contains('/') {
        Some(candidate)
    } else {
        None
    }
}

fn convert_git_time_to_datetime(git_time: &Time) -> DateTime<Utc> {
    Utc.timestamp(git_time.seconds() + i64::from(git_time.offset_minutes()) * 60, 0)
}

pub fn walk_entire_history(git_repo_path: &str) -> Result<Stats, Error> {
    let repo = Repository::open(git_repo_path)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let before_revwalk = Instant::now();

    let mut commits_by_month: HashMap<String, Vec<u32>> = HashMap::new();
    let mut commits_by_day_of_week: HashMap<String, u32> = HashMap::new();
    let mut changes_by_component: HashMap<String, CommitChanges> = HashMap::new();
    let mut component_name_occurrences: Vec<String> = vec![];
    let mut lang_name_occurrences: Vec<String> = vec![];
    let mut num_file_changes: u32 = 0;
    let mut num_commits_to_master: u32 = 0;
    let mut num_prs: u32 = 0;
    let mut missing_prs: u32 = 0;

    revwalk.for_each(|step| {
        let oid = step.unwrap();
        if let Ok(commit) = repo.find_commit(oid) {
            num_commits_to_master += 1;

            if commit.summary().unwrap().contains(COMMIT_MESSAGE_START) {
                num_prs += 1;
            } else {
                missing_prs += 1;
            }
            // record changes by time
            {
                let dt = convert_git_time_to_datetime(&commit.author().when());
                let year = dt.year().to_string();
                let month = (dt.month() - 1) as usize;
                let month_vec = commits_by_month.entry(year).or_insert_with(|| vec![0; 12]);
                month_vec[month] += 1;
                let commit_day = format!("{:?}", dt.weekday());
                *commits_by_day_of_week.entry(commit_day).or_insert(0) += 1;
            }

            let a = if commit.parents().len() == 1 {
                let parent = commit.parent(0).unwrap();
                Some(parent.tree().unwrap())
            } else {
                None
            };
            let b = commit.tree().unwrap();
            let diff = repo.diff_tree_to_tree(a.as_ref(), Some(&b), None).unwrap();
            let ds = diff.deltas();

            let mut added = 0;
            let mut deleted = 0;
            let mut modified = 0;

            let mut local_langs: HashSet<String> = HashSet::new();
            let mut local_comps: HashSet<String> = HashSet::new();
            for d in ds {

                let file_name = d.new_file().path().unwrap().to_str().unwrap().to_owned();
                // we should only consider files in the diff which are changes to the component code
                if !file_name.starts_with("master") && file_name.contains('/') {
                    num_file_changes += 1;
                    let comp_name= extract_component_name_from_filename(&file_name).unwrap_or_else(|| "unknown".to_owned());
                    let lang_name = extract_language_from_filename(&file_name).unwrap_or_else(|| "unknown".to_owned());

                    // only count the language once / diff
                    if !local_langs.contains(&lang_name) && INTERESTING_LANGS.contains(&lang_name.as_str()) {
                        local_langs.insert(lang_name.clone());
                        lang_name_occurrences.push(lang_name);
                    }

                    // only count first occurrence of component / diff
                    if !local_comps.contains(&comp_name) {
                        local_comps.insert(comp_name.clone());

                        match d.status() {
                            Delta::Added => added+=1,
                            Delta::Deleted => deleted+=1,
                            Delta::Modified => modified+=1,
                            _ => ()
                        }
                        let changes = CommitChanges::new(added, deleted, modified);
                        *changes_by_component.entry(comp_name.clone()).or_insert(EMPTY_CHANGES) += changes;
                        component_name_occurrences.push(comp_name);
                    }
                }
            }
        }
    });

    let after_revwalk = Instant::now();
    println!("Revwalk time: {:?}", after_revwalk.duration_since(before_revwalk));

    // count the component_names and languages used
    let before_counts = Instant::now();
    let before_lang_map = Instant::now();
    let lang_name_thread = thread::spawn(move|| {
        lang_name_occurrences.iter().count_by_key(|l| l.to_owned().to_owned())
    });
    let lang_stats = lang_name_thread.join().unwrap();
    let after_lang_map = Instant::now();
    println!("Lang names map creation time: {:?}", after_lang_map.duration_since(before_lang_map));

    let before_comp_map = Instant::now();
    let comp_name_thread = thread::spawn(move|| {
        component_name_occurrences.iter().count_by_key(|c| c.to_owned().to_owned())
    });
    let component_stats = comp_name_thread.join().unwrap();
    let after_comp_map = Instant::now();
    println!("Comp names map creation time: {:?}", after_comp_map.duration_since(before_comp_map));

    let after_counts = Instant::now();
    println!("Processing counts: {:?}", after_counts.duration_since(before_counts));

    Ok(Stats::new(num_commits_to_master,
                  num_prs,
                  missing_prs,
                  num_file_changes,
                  component_stats,
                  lang_stats,
                  commits_by_month,
                  commits_by_day_of_week,
                  changes_by_component
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_component_name_from_diff_summary() {
        let r = extract_component_name_from_filename(&"component-a/src/test/java/Thing.java".to_owned());
        assert!(r.is_some());
        assert_eq!(r.unwrap(), "component-a");
    }

    #[test]
    fn test_extract_language_from_diff_summary() {
        let r = extract_language_from_filename(&"component-a/src/test/java/Thing.java".to_owned());
        assert!(r.is_some());
        assert_eq!(r.unwrap(), "java");
    }

    #[test]
    fn test_day() {
        let utc: DateTime<Utc> = Utc::now();
        println!("{:?}", utc.weekday())
    }

    #[test]
    fn test_count_by_key() {
        let items = vec!["a", "a", "b", "c", "c", "d"];
        let m = items.iter().count_by_key(|i| i.to_owned().to_owned());
        let count = m.get("a").unwrap().to_owned();
        assert_eq!(count, 2);
    }
}