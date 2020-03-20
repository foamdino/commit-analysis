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
extern crate serde_json;

use std::fs;
use docopt::Docopt;

fn main() {
    const USAGE: &str = "
Usage: commit-analysis <git_repo_path>
";

    let args = Docopt::new(USAGE)
        .and_then(|d| d.parse())
        .unwrap_or_else(|e| e.exit());

    let output_file = "/tmp/commit-analysis.json";
    let analysis = commit_analysis::walk_entire_history(args.get_str("<git_repo_path>"));
    let json = serde_json::to_string(&analysis.unwrap());
    fs::write(&output_file, json.unwrap()).unwrap_or_else(|_| panic!("couldn't write to file: {}", &output_file));
}