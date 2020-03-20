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

use std::collections::HashMap;
use crate::CommitChanges;

#[derive(Debug, Deserialize, Serialize)]
pub struct Stats {
    num_commits_to_master: u32,
    num_prs: u32,
    missing_prs: u32,
    num_file_changes: u32,
    component_stats: HashMap<String, u32>,
    lang_stats: HashMap<String, u32>,
    commits_by_month: HashMap<String, Vec<u32>>,
    commits_by_day_of_week: HashMap<String, u32>,
    changes_by_component: HashMap<String, CommitChanges>,
}

impl Stats {

    pub fn new(num_commits_to_master: u32,
               num_prs: u32,
               missing_prs: u32,
               num_file_changes: u32,
               component_stats: HashMap<String, u32>,
               lang_stats: HashMap<String, u32>,
               commits_by_month: HashMap<String, Vec<u32>>,
               commits_by_day_of_week: HashMap<String, u32>,
               changes_by_component: HashMap<String, CommitChanges>) -> Stats {

        Stats{
            num_commits_to_master,
            num_prs,
            missing_prs,
            num_file_changes,
            component_stats,
            lang_stats,
            commits_by_month,
            commits_by_day_of_week,
            changes_by_component
        }
    }
}