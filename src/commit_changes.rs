//#[macro_use]
//extern crate serde_derive;

use std::ops::{Add, AddAssign};

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