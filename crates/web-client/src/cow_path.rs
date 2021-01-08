use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

pub struct CowPath(Cow<'static, Path>);

impl CowPath {
    pub fn join(&self, other: CowPath) -> Self {
        Self(Cow::Owned(self.0.join(other.0)))
    }
}

impl From<&'static Path> for CowPath {
    fn from(p: &'static Path) -> Self {
        Self(Cow::Borrowed(p))
    }
}

impl From<&'static str> for CowPath {
    fn from(s: &'static str) -> Self {
        Self::from(Path::new(s))
    }
}

impl From<PathBuf> for CowPath {
    fn from(p: PathBuf) -> Self {
        Self(Cow::Owned(p))
    }
}

impl From<String> for CowPath {
    fn from(s: String) -> Self {
        Self::from(PathBuf::from(s))
    }
}

impl std::ops::Deref for CowPath {
    type Target = Cow<'static, Path>;

    fn deref(&self) -> &Cow<'static, Path> {
        &self.0
    }
}
