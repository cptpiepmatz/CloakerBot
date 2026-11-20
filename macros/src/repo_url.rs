#[macro_export]
#[allow(clippy::crate_in_macro_def, reason = "repository metadata belongs to the calling crate")]
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
