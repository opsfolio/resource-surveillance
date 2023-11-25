use std::collections::HashMap;

use globset::{Glob, GlobSet, GlobSetBuilder};

/// Trait for items that can be classified based on GlobSet patterns.
pub trait Classifiable {
    /// Determines whether the item matches a given glob pattern set.
    fn is_match(&self, globset: &GlobSet) -> bool;
}

pub enum ClassifiableContent<Class, T> {
    Ignored(Class, String),
    NotFound(Class, String),
    NotFile(Class, String),
    Resource(T, Class),
    Error(Class, Box<dyn std::error::Error>),
}

pub trait ClassifiableContentSupplier<Resource, Class> {
    fn content(&self) -> ClassifiableContent<Resource, Class>;
}

/// Type alias for the classifier function.
///
/// `Target` - Type of item to be classified, must implement `Classifiable`.
/// `Class` - Type holding classification results.
/// `Context` - Custom context provided by classification rules.
pub type Classifier<Target, Class, Context> =
    Box<dyn Fn(&mut Class, &Target, &Context, &str) -> bool + Sync + Send>;

/// Type alias for classifier initialization HashMaps.
pub type ClassifiersInit<Target, Class, Context> =
    HashMap<String, (Vec<String>, Classifier<Target, Class, Context>)>;

/// Type alias for cached classifier HashMaps and their GlobSets.
pub type Classifiers<Target, Class, Context> =
    HashMap<String, (GlobSet, Classifier<Target, Class, Context>)>;

/// Struct for managing classification rules.
///
/// `Class` - Type to hold classification results.
/// `Target` - Type of item to be classified.
pub struct ClassificationRules<Target, Class, Context>
where
    Target: Classifiable + ?Sized,
{
    pub empty_class_fn: Box<dyn Fn() -> Class>,
    pub candidates_globset: GlobSet,
    pub classifiers: Option<Classifiers<Target, Class, Context>>,
}

impl<Target, Class, Context> ClassificationRules<Target, Class, Context>
where
    Target: Classifiable + ?Sized,
{
    /// Creates a new `ClassificationRules` instance.
    pub fn new(
        empty_class_fn: Box<dyn Fn() -> Class>,
        candidates_globs: &[String],
        classifier_globs: ClassifiersInit<Target, Class, Context>,
    ) -> anyhow::Result<Self, globset::Error> {
        let mut builder = GlobSetBuilder::new();
        for glob_pattern in candidates_globs {
            builder.add(Glob::new(glob_pattern)?);
        }
        let candidates = builder.build()?;

        let classifiers: HashMap<String, (GlobSet, Classifier<Target, Class, Context>)> =
            classifier_globs
                .into_iter()
                .map(|(key, (globs, classifier))| {
                    let mut builder = GlobSetBuilder::new();
                    for glob_pattern in globs {
                        builder.add(Glob::new(&glob_pattern).unwrap());
                    }
                    let glob_set = builder.build().unwrap();
                    (key, (glob_set, classifier))
                })
                .collect();

        Ok(ClassificationRules {
            empty_class_fn,
            candidates_globset: candidates,
            classifiers: if classifiers.is_empty() {
                None
            } else {
                Some(classifiers)
            },
        })
    }

    /// Classifies a given item.
    pub fn classify(&self, item: &Target, ctx: &Context) -> (Class, bool, usize) {
        let mut class = (self.empty_class_fn)();
        let mut interrupted = false;
        let mut classified_count = 0;

        if let Some(classifiers) = &self.classifiers {
            for (key, (glob_set, classifier)) in classifiers {
                if item.is_match(glob_set) {
                    let proceed = classifier(&mut class, item, ctx, key);
                    if !proceed {
                        interrupted = true;
                        break;
                    }
                    classified_count += 1;
                }
            }
        }

        (class, interrupted, classified_count)
    }
}

impl Classifiable for ignore::DirEntry {
    fn is_match(&self, globset: &GlobSet) -> bool {
        globset.is_match(self.path())
    }
}

/// `ignore` crate based classification rules to walk through file system
/// directories and classify entries based on glob patterns that honor
/// `.ignore` and `.gitignore` files. This object can ignore hidden files
/// and directories as well.
pub struct IgnorableFileSysEntries<Class> {
    pub rules: ClassificationRules<ignore::DirEntry, Class, String>,
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
            rules: ClassificationRules::new(empty_class_fn, candidates_globs, classifier_globs)
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

impl Classifiable for walkdir::DirEntry {
    fn is_match(&self, globset: &GlobSet) -> bool {
        globset.is_match(self.path())
    }
}

/// `walkdir` crate based classification rules to walk through file system
/// directories and classify entries based on glob patterns. Differs from
/// IgnorableFileSysEntries in that it walks all entries without regard to
/// any `.ignore` or `.gitignore` filters.
pub struct WalkableFileSysEntries<Class> {
    pub rules: ClassificationRules<walkdir::DirEntry, Class, String>,
}

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
            rules: ClassificationRules::new(empty_class_fn, candidates_globs, classifier_globs)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::Path;

    impl Classifiable for Path {
        fn is_match(&self, globset: &GlobSet) -> bool {
            globset.is_match(self)
        }
    }

    /// Classification result for files.
    struct PathClassification {
        is_executable: bool,
    }

    /// Return a executable_classifier that will check a DirEntry path has proper file system
    /// permissions to execute a file.
    fn executable_classifier() -> Classifier<Path, PathClassification, bool> {
        Box::new(|class, item, _ctx, _purpose| -> bool {
            class.is_executable = item.extension().unwrap() == "exe";
            true
        })
    }

    #[test]
    fn test_executable_classification() {
        let empty_class_fn = Box::new(|| PathClassification {
            is_executable: false,
        });
        let candidates_globs = vec![String::from("*")];
        let classifier_globs = HashMap::from([(
            String::from("executable"),
            (vec![String::from("*.exe")], executable_classifier()),
        )]);

        let classification_rules = ClassificationRules::<Path, PathClassification, bool>::new(
            empty_class_fn,
            &candidates_globs,
            classifier_globs,
        )
        .unwrap();

        let executable_file = Path::new("program.exe");
        let (classification, _, _) = classification_rules.classify(executable_file, &true);
        assert!(classification.is_executable);

        let non_executable_file = Path::new("document.txt");
        let (classification, _, _) = classification_rules.classify(non_executable_file, &true);
        assert!(!classification.is_executable);
    }
}
