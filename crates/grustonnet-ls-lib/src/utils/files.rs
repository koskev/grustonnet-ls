use itertools::Itertools;
use language_server::utils::UriHelper;
use lsp_types::Uri;
use walkdir::WalkDir;

pub fn get_all_jsonnnet_files(search_paths: &[String]) -> Vec<Uri> {
    search_paths
        .iter()
        .flat_map(|p| {
            WalkDir::new(p)
                .into_iter()
                .filter_map(|r| r.ok())
                .filter(|f| {
                    f.path().is_file()
                        && f.path()
                            .extension()
                            .map(|e| e == "jsonnet" || e == "libsonnet")
                            .unwrap_or(false)
                })
                .map(|f| f.into_path())
        })
        .filter_map(|path| Uri::from_path(path.to_str()?).ok())
        .unique()
        .collect()
}
