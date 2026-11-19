#[macro_export]
macro_rules! repo_url {
    ($path:literal) => {
        ::const_format::concatcp!(
            crate::MANIFEST.package.repository,
            "/blob/",
            crate::git::BRANCH,
            $path
        )
    };
}
