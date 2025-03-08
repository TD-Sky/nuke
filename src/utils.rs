pub mod fs {
    use std::io;
    use std::path::Path;

    /// The path points to:
    /// - existing file | directory | symlink => Ok(true)
    /// - broken symlink => Ok(true)
    /// - nothing => Ok(false)
    /// - I/O Error => Err(Io)
    pub fn virtually_exists(path: impl AsRef<Path>) -> io::Result<bool> {
        virtually_exists_impl(path.as_ref())
    }

    #[inline]
    fn virtually_exists_impl(path: &Path) -> io::Result<bool> {
        Ok(path.try_exists()? || path.is_symlink())
    }
}
