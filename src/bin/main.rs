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