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