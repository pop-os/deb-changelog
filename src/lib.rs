#[macro_use]
extern crate smart_default;
#[macro_use]
extern crate thiserror;

mod entry;
mod iter;

pub use self::{
    entry::{Entry, EntryError},
    iter::ChangelogIter,
};

use std::io;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to append original changelog to temporary changelog")]
    Append(#[source] io::Error),
    #[error("failed to create temporary changelog file")]
    CreateTemporary(#[source] io::Error),
    #[error("failed to flush I/O to temporary changelog")]
    Flush(#[source] io::Error),
    #[error("failed to open original changelog file")]
    OpenOriginal(#[source] io::Error),
    #[error("path is not UTF-8")]
    PathNotUtf8,
    #[error("failed to replace changelog with new updated changelog")]
    Replace(#[source] io::Error),
    #[error("failed to write new entry to temporary file")]
    Write(#[source] io::Error),
}

pub mod sync {
    use crate::{Entry, Error};
    use std::{
        fs::{rename, File},
        io::{self, Write},
        path::Path,
    };

    pub fn append(path: &Path, entry: Entry) -> Result<(), Error> {
        let src_path: &str = path.to_str().ok_or(Error::PathNotUtf8)?;
        let dst_path: &str = &[src_path, ".bak"].concat();

        {
            let mut src_file = File::create(dst_path).map_err(Error::CreateTemporary)?;
            let mut dst_file = File::open(src_path).map_err(Error::OpenOriginal)?;

            writeln!(&mut src_file, "{}\n", entry).map_err(Error::Write)?;
            io::copy(&mut dst_file, &mut src_file).map_err(Error::Append)?;
            src_file.flush().map_err(Error::Flush)?;
        }

        rename(dst_path, src_path).map_err(Error::Replace)
    }
}

#[cfg(any(feature = "std-async", feature = "tokio-async"))]
pub mod r#async {
    #[cfg(feature = "std-async")]
    use async_std::{
        fs::{rename, File},
        io,
    };
    #[cfg(feature = "std-async")]
    use futures::io::AsyncWriteExt;

    #[cfg(feature = "tokio-async")]
    use tokio::{
        fs::{rename, File},
        io::{self, AsyncWriteExt},
    };

    use crate::{Entry, Error};
    use futures::try_join;
    use std::path::Path;

    pub async fn append<'a>(path: &Path, entry: Entry<'a>) -> Result<(), Error> {
        let src_path: &str = path.to_str().ok_or(Error::PathNotUtf8)?;
        let dst_path: &str = &[src_path, ".bak"].concat();

        {
            let src_file = async { File::create(dst_path).await.map_err(Error::CreateTemporary) };

            let dst_file = async { File::open(src_path).await.map_err(Error::OpenOriginal) };
            let (mut src_file, mut dst_file) = try_join!(src_file, dst_file)?;

            let formatted = format!("{}\n", entry);
            src_file.write(formatted.as_bytes()).await.map_err(Error::Write)?;

            #[cfg(feature = "std-async")]
            io::copy(&mut dst_file, &mut src_file).await.map_err(Error::Append)?;
            #[cfg(feature = "tokio-async")]
            io_copy(&mut dst_file, &mut src_file).await.map_err(Error::Append)?;

            src_file.flush().await.map_err(Error::Flush)?;
        }

        rename(dst_path, src_path).await.map_err(Error::Replace)
    }

    #[cfg(feature = "tokio-async")]
    async fn io_copy(src: &mut File, dst: &mut File) -> io::Result<()> {
        use tokio::io::AsyncReadExt;
        let mut buffer = [0u8; 8 * 1024];

        loop {
            let read = src.read(&mut buffer).await?;

            if read == 0 {
                break Ok(())
            }

            dst.write(&mut buffer[..read]).await?;
        }
    }
}
