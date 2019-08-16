use commit_analysis::*;

use chrono::{DateTime, Utc, Datelike};

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

#[test]
fn test_day() {
    let utc: DateTime<Utc> = Utc::now();
    println!("{:?}", utc.weekday())
}

#[test]
fn test_add_commit_changes() {
    let one = CommitChanges::new(1, 2);
    let two = CommitChanges::new(2, 3);
    let three = one + two;
    assert_eq!(5, three.get_num_deletions().to_owned());
}