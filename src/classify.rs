use std::collections::HashMap;

use globset::{Glob, GlobSet, GlobSetBuilder};

/// Trait for items that can be classified based on GlobSet patterns.
pub trait Classifiable {
    /// Determines whether the item matches a given glob pattern set.
    fn is_match(&self, globset: &GlobSet) -> bool;
}

/// Type alias for the classifier function.
///
/// `Class` - Type holding classification results.
/// `Target` - Type of item to be classified, must implement `Classifiable`.
pub type Classifier<Target, Class, Context> =
    Box<dyn Fn(&mut Class, &Target, &Context, &str) -> bool + Sync + Send>;

pub type ClassifiersInit<Target, Class, Context> =
    HashMap<String, (Vec<String>, Classifier<Target, Class, Context>)>;

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
    pub classifiers: HashMap<String, (GlobSet, Classifier<Target, Class, Context>)>,
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

        let classifiers = classifier_globs
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
            classifiers,
        })
    }

    /// Classifies a given item.
    pub fn classify(&self, item: &Target, ctx: &Context) -> (Class, bool) {
        let mut class = (self.empty_class_fn)();
        let mut interrupted = false;

        for (key, (glob_set, classifier)) in &self.classifiers {
            if item.is_match(glob_set) {
                let proceed = classifier(&mut class, item, ctx, key);
                if !proceed {
                    interrupted = true;
                    break;
                }
            }
        }

        (class, interrupted)
    }
}

impl Classifiable for ignore::DirEntry {
    fn is_match(&self, globset: &GlobSet) -> bool {
        globset.is_match(self.path())
    }
}

/// A structure to walk through file system directories and apply flags to entries based on glob patterns.
pub struct ClassifiableFileSysEntries<Class> {
    pub cr: ClassificationRules<ignore::DirEntry, Class, String>,
    pub include_hidden: bool,
}

impl<Class> ClassifiableFileSysEntries<Class> {
    /// Constructs a new `ClassifiableFileSysEntries`.
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
        Ok(ClassifiableFileSysEntries {
            cr: ClassificationRules::new(empty_class_fn, candidates_globs, classifier_globs)
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
                        if self.cr.candidates_globset.is_match(entry.path()) {
                            let (class, _processed_all) = self.cr.classify(&entry, root_path);
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
        let (classification, _) = classification_rules.classify(executable_file, &true);
        assert!(classification.is_executable);

        let non_executable_file = Path::new("document.txt");
        let (classification, _) = classification_rules.classify(non_executable_file, &true);
        assert!(!classification.is_executable);
    }
}
