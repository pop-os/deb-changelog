use crate::entry::{Entry, EntryError};

pub struct ChangelogIter<'a> {
    data:  &'a str,
    entry: &'a mut Entry<'a>,
}

impl<'a> ChangelogIter<'a> {
    pub fn new(entry: &'a mut Entry<'a>, data: &'a str) -> Self { Self { data, entry } }

    pub fn next<'b>(&'b mut self) -> Option<Result<&'b mut Entry<'a>, EntryError>> {
        if self.data.is_empty() {
            return None;
        }

        let result = match self.entry.parse_from_str(self.data) {
            Ok(read) => {
                self.data = &self.data[read..];
                Ok(&mut *self.entry)
            }
            Err(why) => {
                self.data = "";
                Err(why)
            }
        };

        Some(result)
    }
}
