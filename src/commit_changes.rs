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

use std::ops::{Add, AddAssign};

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub struct CommitChanges {
    files_added: u32,
    files_deleted: u32,
    files_modified: u32,
}

impl CommitChanges {

    pub const fn new(fa: u32, fd: u32, fm: u32) -> CommitChanges {
        CommitChanges{files_added: fa, files_deleted: fd, files_modified: fm}
    }
}

impl Add for CommitChanges {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            files_added: self.files_added + other.files_added,
            files_deleted: self.files_added + other.files_deleted,
            files_modified: self.files_modified + other.files_modified
        }
    }
}

impl AddAssign for CommitChanges {
    fn add_assign(&mut self, other: CommitChanges) {
        *self = CommitChanges {
            files_added: self.files_added + other.files_added,
            files_deleted: self.files_added + other.files_deleted,
            files_modified: self.files_modified + other.files_modified
        }
    }
}