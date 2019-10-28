use crate::iter::ChangelogIter;
use chrono::{DateTime, Utc};
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

macro_rules! strfmt {
    ($f:expr, $($expression:expr),* ) => {{
        $( $f.write_str($expression)?; )*
    }}
}

#[derive(Debug, Error)]
pub enum EntryError {
    #[error("expected an email to be assigned to an author")]
    AuthorWithoutEmail,
    #[error("parsed date ({}) is not in the RFC2822 format", _0)]
    BadDate(Box<str>),
    #[error("metadata pair ({}) lacks value", _0)]
    BadMetadata(Box<str>),
    #[error("author field contains an email which lacks the closing '>'")]
    EmailNotEnclosed,
    #[error("expected to see a double-spaced separator between author and date")]
    NoDate,
    #[error("expected to find a footer, which starts with ` --`")]
    NoFooter,
    #[error("expected to find a header, but none were found")]
    NoHeader,
    #[error("expected package field in header")]
    NoPackage,
    #[error("expected a version field in header")]
    NoVersion,
    #[error("version field in header requires parenthesis")]
    VersionRequiresParenthesis,
}

#[derive(Debug, SmartDefault)]
pub struct Entry<'a> {
    pub author: &'a str,
    pub email: &'a str,
    pub changes: Vec<&'a str>,
    pub distributions: Vec<&'a str>,
    pub package: &'a str,
    pub version: &'a str,
    pub metadata: HashMap<&'a str, &'a str>,
    #[default(default_date())]
    pub date: DateTime<Utc>,
}

// NOTE: If this could be generated statically, that would be great.
fn default_date() -> DateTime<Utc> { Utc::now() }

impl<'a> Entry<'a> {
    pub fn changes<C: IntoIterator<Item = &'a str>>(&'a mut self, changes: C) -> &'a mut Self {
        self.changes.clear();

        changes.into_iter().for_each(|change| self.changes.push(change));
        self
    }

    pub fn distributions<C: IntoIterator<Item = &'a str>>(
        &'a mut self,
        distributions: C,
    ) -> &'a mut Self {
        self.distributions.clear();
        distributions.into_iter().for_each(|dist| self.distributions.push(dist));
        self
    }

    pub fn iter_from(&'a mut self, log: &'a str) -> ChangelogIter<'a> {
        ChangelogIter::new(self, log)
    }

    pub fn parse_from_str(&mut self, string: &'a str) -> Result<usize, EntryError> {
        self.changes.clear();
        self.distributions.clear();
        self.metadata.clear();

        let mut read = 0;
        let mut lines = string.lines();

        let header = lines.next().ok_or(EntryError::NoHeader)?;
        read += header.len() + 1;
        parse_header(self, dbg!(header))?;

        while let Some(line) = lines.next() {
            read += line.len() + 1;
            if line.trim().is_empty() {
                continue;
            } else if line.starts_with(" --") {
                let footer = &line[3..].trim_start();
                parse_footer(self, dbg!(footer))?;

                while let Some(line) = lines.next() {
                    if line.trim_start().len() == 0 {
                        read += line.len() + 1;
                    } else {
                        break;
                    }
                }
                return Ok(read);
            } else if line.starts_with("  ") {
                self.changes.push(line[2..].trim_end());
            }
        }

        Err(EntryError::NoFooter)
    }
}

/// $PACKAGE ($VERSION) $DIST1 $DIST2 $DIST3; urgency=$URGENCY
fn parse_header<'a>(block: &mut Entry<'a>, header: &'a str) -> Result<(), EntryError> {
    let mut fields = header.split_ascii_whitespace();
    block.package = fields.next().ok_or(EntryError::NoPackage)?;

    let version = fields.next().ok_or(EntryError::NoVersion)?;

    if !(version.starts_with('(') && version.ends_with(')')) {
        return Err(EntryError::VersionRequiresParenthesis);
    }
    block.version = &version[1..version.len() - 1];

    while let Some(distribution) = fields.next() {
        if distribution.ends_with(';') {
            block.distributions.push(&distribution[..distribution.len() - 1]);
            break;
        }

        block.distributions.push(distribution);
    }

    if let Some(metadata) = fields.next() {
        for pair in metadata.split(',') {
            if let Some(pos) = pair.find('=') {
                let (key, value) = pair.split_at(pos);
                block.metadata.insert(key, &value[1..]);
            } else {
                return Err(EntryError::BadMetadata(pair.into()));
            }
        }
    }

    Ok(())
}

/// $AUTHOR <$EMAIL>  $DATE_RFC2822
fn parse_footer<'a>(block: &mut Entry<'a>, footer: &'a str) -> Result<(), EntryError> {
    if let Some(pos) = footer.find("  ") {
        let (author, mut date) = footer.split_at(pos);
        date = &date[2..];
        block.date = DateTime::parse_from_rfc2822(date)
            .map_err(|_| EntryError::BadDate(date.into()))
            .map(|fixed| fixed.into())?;

        return if let Some(pos) = author.find('<') {
            let (author, email) = author.split_at(pos);
            if !email.ends_with('>') {
                return Err(EntryError::EmailNotEnclosed);
            }
            block.author = author.trim_end();
            block.email = email[1..email.len() - 1].trim();
            Ok(())
        } else {
            Err(EntryError::AuthorWithoutEmail)
        };
    }

    Err(EntryError::NoDate)
}

impl<'a> Display for Entry<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        strfmt!(f, self.package, " (", self.version, ")");

        for distribution in &*self.distributions {
            f.write_str(" ")?;
            f.write_str(distribution)?;
        }

        f.write_str("; ")?;

        for (key, value) in &self.metadata {
            f.write_str(key)?;
            f.write_str("=")?;
            f.write_str(value)?;
        }

        f.write_str("\n\n")?;

        for line in &*self.changes {
            f.write_str("  ")?;
            f.write_str(line)?;
            f.write_str("\n")?;
        }

        strfmt!(f, "\n -- ", self.author, " <", self.email, ">  ", self.date.to_rfc2822().as_str());

        Ok(())
    }
}
