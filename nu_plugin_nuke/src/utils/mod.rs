pub mod collections;

pub mod path {
    use std::path::Path;
    use std::time::SystemTime;

    use easy_ext::ext;

    #[ext(PathExt)]
    pub impl Path {
        fn timestamp(&self) -> Option<SystemTime> {
            self.metadata()
                .and_then(|metadata| metadata.modified())
                .ok()
        }
    }
}
