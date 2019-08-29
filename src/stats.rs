use std::collections::BTreeMap;

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

impl Stats {

    pub fn new(num_commits_to_master: i32,
               num_prs: i32,
               num_files: i32,
               component_stats: BTreeMap<String, i32>,
               lang_stats: BTreeMap<String, i32>,
               commits_by_month: BTreeMap<String, Vec<i32>>,
               commits_by_day_of_week: BTreeMap<String, i32>,
               filtered_languages: BTreeMap<String, i32>) -> Stats {

        Stats{
            num_commits_to_master: num_commits_to_master,
            num_prs: num_prs,
            num_files: num_files,
            component_stats: component_stats,
            lang_stats: lang_stats,
            filtered_languages: filtered_languages,
            commits_by_month: commits_by_month,
            commits_by_day_of_week: commits_by_day_of_week
        }
    }
}