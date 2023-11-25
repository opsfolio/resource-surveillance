use globset::GlobSet;

use crate::classify::*;

impl GlobSetClassifiable for ignore::DirEntry {
    fn is_match(&self, globset: &GlobSet) -> bool {
        globset.is_match(self.path())
    }
}

/// `ignore` crate based classification rules to walk through file system
/// directories and classify entries based on glob patterns that honor
/// `.ignore` and `.gitignore` files. This object can ignore hidden files
/// and directories as well.
pub struct IgnorableFileSysEntries<Class> {
    pub rules: GlobSetClassificationRules<ignore::DirEntry, Class, String>,
    pub include_hidden: bool,
}

impl<Class> IgnorableFileSysEntries<Class> {
    /// Constructs a new `IgnorableFileSysEntries`.
    ///
    /// # Arguments
    ///
    /// * `empty_class_fn` - A function that returns an empty classification instance
    /// * `candidates_globs` - Primary glob patterns to filter the files.
    /// * `classifier_globs` - A map of named glob patterns to their corresponding flags and flagging functions.
    ///
    /// # Returns
    ///
    /// A result containing the new instance or an error if the glob patterns are invalid.
    pub fn new(
        empty_class_fn: Box<dyn Fn() -> Class>,
        candidates_globs: &[String],
        classifier_globs: ClassifiersInit<ignore::DirEntry, Class, String>,
        include_hidden: bool,
    ) -> anyhow::Result<Self, globset::Error> {
        Ok(IgnorableFileSysEntries {
            rules: GlobSetClassificationRules::new(
                empty_class_fn,
                candidates_globs,
                classifier_globs,
            )
            .unwrap(),
            include_hidden,
        })
    }

    /// Walks through the file system based on the specified root paths and
    /// classifies each entry. This code will recursively traverse the given
    /// directories while automatically filtering out hidden files and
    /// directories plus any filtering entries according to ignore globs found
    /// in files like `.ignore` and `.gitignore`.
    ///
    /// # Arguments
    ///
    /// * `walk_paths` - A vector of root file system paths to walk through.
    /// * `handle_entry` - A closure to process each file. It receives the root path, a directory entry, and the calculated flags.
    ///
    /// # Returns
    ///
    /// A result indicating success or containing an error.
    pub fn walk<F>(&self, walk_paths: &[String], mut handle_entry: F)
    where
        F: FnMut(&str, &ignore::DirEntry, &Class),
    {
        for root_path in walk_paths {
            let ignorable_walk = if self.include_hidden {
                ignore::WalkBuilder::new(root_path).hidden(false).build()
            } else {
                ignore::Walk::new(root_path)
            };
            for entry in ignorable_walk {
                match entry {
                    Ok(entry) => {
                        if self.rules.candidates_globset.is_match(entry.path()) {
                            let (class, _processed_all, _count) =
                                self.rules.classify(&entry, root_path);
                            handle_entry(root_path, &entry, &class);
                        }
                    }
                    Err(err) => {
                        eprintln!("[TODO move to Otel] walk error in {}: {}", root_path, err);
                    }
                }
            }
        }
    }
}

impl GlobSetClassifiable for walkdir::DirEntry {
    fn is_match(&self, globset: &GlobSet) -> bool {
        globset.is_match(self.path())
    }
}

/// `walkdir` crate based classification rules to walk through file system
/// directories and classify entries based on glob patterns. Differs from
/// IgnorableFileSysEntries in that it walks all entries without regard to
/// any `.ignore` or `.gitignore` filters.
pub struct WalkableFileSysEntries<Class> {
    pub rules: GlobSetClassificationRules<walkdir::DirEntry, Class, String>,
}

// TODO: remove #[allow(dead_code)] after code reviews
#[allow(dead_code)]
impl<Class> WalkableFileSysEntries<Class> {
    /// Constructs a new `WalkableFileSysEntries`.
    ///
    /// # Arguments
    ///
    /// * `empty_class_fn` - A function that returns an empty classification instance
    /// * `candidates_globs` - Primary glob patterns to filter the files.
    /// * `classifier_globs` - A map of named glob patterns to their corresponding flags and flagging functions.
    ///
    /// # Returns
    ///
    /// A result containing the new instance or an error if the glob patterns are invalid.
    pub fn new(
        empty_class_fn: Box<dyn Fn() -> Class>,
        candidates_globs: &[String],
        classifier_globs: ClassifiersInit<walkdir::DirEntry, Class, String>,
    ) -> anyhow::Result<Self, globset::Error> {
        Ok(WalkableFileSysEntries {
            rules: GlobSetClassificationRules::new(
                empty_class_fn,
                candidates_globs,
                classifier_globs,
            )
            .unwrap(),
        })
    }

    /// Walks through the file system based on the specified root paths and
    /// classifies each entry. This code will recursively traverse the given
    /// directories while automatically filtering out hidden files and
    /// directories plus any filtering entries according to ignore globs found
    /// in files like `.ignore` and `.gitignore`.
    ///
    /// # Arguments
    ///
    /// * `walk_paths` - A vector of root file system paths to walk through.
    /// * `handle_entry` - A closure to process each file. It receives the root path, a directory entry, and the calculated flags.
    ///
    /// # Returns
    ///
    /// A result indicating success or containing an error.
    pub fn walk<F>(&self, walk_paths: &[String], mut handle_entry: F)
    where
        F: FnMut(&str, &walkdir::DirEntry, &Class),
    {
        for root_path in walk_paths {
            for entry in walkdir::WalkDir::new(root_path) {
                match entry {
                    Ok(entry) => {
                        if self.rules.candidates_globset.is_match(entry.path()) {
                            let (class, _processed_all, _count) =
                                self.rules.classify(&entry, root_path);
                            handle_entry(root_path, &entry, &class);
                        }
                    }
                    Err(err) => {
                        eprintln!("[TODO move to Otel] walk error in {}: {}", root_path, err);
                    }
                }
            }
        }
    }
}

pub struct FileSysTypicalClass {
    pub is_executable: bool,
}

pub fn empty_fs_typical_class() -> Box<dyn Fn() -> FileSysTypicalClass> {
    Box::new(|| FileSysTypicalClass {
        is_executable: false,
    })
}
