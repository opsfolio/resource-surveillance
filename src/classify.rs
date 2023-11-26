use std::collections::HashMap;

use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::{Regex, RegexSet};

/// Type alias for the classifier function.
///
/// `Target` - Type of item to be classified, must implement `Classifiable`.
/// `Class` - Type holding classification results.
/// `Context` - Custom context provided by classification rules.
pub type Classifier<Target, Class, Context> =
    Box<dyn Fn(&mut Class, &Target, &Context, &str) -> bool + Sync + Send>;

/// Type alias for classifier initialization HashMaps when each classfier is just a closure/fn.
pub type ClassifiersFnInit<Target, Class, Context> =
    HashMap<String, Classifier<Target, Class, Context>>;

/// Type alias for classifier initialization HashMaps when each classfier has multi-text and associated closure/fn.
pub type ClassifiersTextVecFnInit<Target, Class, Context> =
    HashMap<String, (Vec<String>, Classifier<Target, Class, Context>)>;

/// Type alias for classifier initialization HashMaps when each classifier has a single ext and associated closure/fn.
pub type ClassifiersTextFnInit<Target, Class, Context> =
    HashMap<String, (String, Classifier<Target, Class, Context>)>;

/// Trait for items that can be classified based on GlobSet patterns.
pub trait GlobSetClassifiable {
    /// Determines whether the item matches a given GlobSet pattern set.
    fn is_match(&self, globset: &GlobSet) -> bool;
}

/// Type alias for cached classifier HashMaps and their GlobSets.
pub type GlobSetClassifiers<Target, Class, Context> =
    HashMap<String, (GlobSet, Classifier<Target, Class, Context>)>;

/// Struct for managing GlobSet classification rules.
///
/// `Class` - Type to hold classification results.
/// `Target` - Type of item to be classified.
/// `Context` - Anything to pass into each classifier in case it needs more context.
pub struct GlobSetClassificationRules<Target, Class, Context>
where
    Target: GlobSetClassifiable + ?Sized,
{
    pub empty_class_fn: Box<dyn Fn() -> Class>,
    pub candidates_globset: GlobSet,
    pub classifiers: Option<GlobSetClassifiers<Target, Class, Context>>,
}

impl<Target, Class, Context> GlobSetClassificationRules<Target, Class, Context>
where
    Target: GlobSetClassifiable + ?Sized,
{
    /// Creates a new `ClassificationRules` instance.
    pub fn new(
        empty_class_fn: Box<dyn Fn() -> Class>,
        candidates_globs: &[String],
        classifier_globs: ClassifiersTextVecFnInit<Target, Class, Context>,
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

        Ok(GlobSetClassificationRules {
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

/// Trait for items that can be classified based on Regex pattern.
pub trait RegexClassifiable {
    /// Determines whether the item matches a given Regex pattern.
    fn is_match(&self, set: &Regex) -> bool;
}

/// Type alias for cached classifier HashMaps and their Regex.
pub type RegexClassifiers<Target, Class, Context> =
    HashMap<String, (Regex, Classifier<Target, Class, Context>)>;

/// Struct for managing Regex classification rules.
///
/// `Class` - Type to hold classification results.
/// `Target` - Type of item to be classified.
/// `Context` - Anything to pass into each classifier in case it needs more context.
#[allow(dead_code)]
pub struct RegexClassificationRules<Target, Class, Context>
where
    Target: RegexClassifiable + ?Sized,
{
    pub empty_class_fn: Box<dyn Fn() -> Class>,
    pub classifiers: RegexClassifiers<Target, Class, Context>,
}

#[allow(dead_code)]
impl<Target, Class, Context> RegexClassificationRules<Target, Class, Context>
where
    Target: RegexClassifiable + ?Sized,
{
    /// Creates a new `ClassificationRules` instance.
    pub fn new(
        empty_class_fn: Box<dyn Fn() -> Class>,
        classifier_regexs: ClassifiersTextFnInit<Target, Class, Context>,
    ) -> anyhow::Result<Self, regex::Error> {
        let classifiers: HashMap<String, (Regex, Classifier<Target, Class, Context>)> =
            classifier_regexs
                .into_iter()
                .map(|(key, (regex, classifier))| (key, (Regex::new(&regex).unwrap(), classifier)))
                .collect();

        Ok(RegexClassificationRules {
            empty_class_fn,
            classifiers,
        })
    }

    /// Classifies a given item.
    pub fn classify(&self, item: &Target, ctx: &Context) -> (Class, bool, usize) {
        let mut class = (self.empty_class_fn)();
        let mut interrupted = false;
        let mut classified_count = 0;

        for (key, (regex, classifier)) in &self.classifiers {
            if item.is_match(regex) {
                let proceed = classifier(&mut class, item, ctx, key);
                if !proceed {
                    interrupted = true;
                    break;
                }
                classified_count += 1;
            }
        }

        (class, interrupted, classified_count)
    }
}

/// Trait for items that can be classified based on RegexSet patterns.
pub trait RegexSetClassifiable {
    /// Determines whether the item matches a given RegexSet pattern set.
    fn is_match(&self, set: &RegexSet) -> bool;
}
/// Type alias for cached classifier HashMaps and their RegexSets.
pub type RegexSetClassifiers<Target, Class, Context> =
    HashMap<String, (RegexSet, Classifier<Target, Class, Context>)>;

/// Struct for managing RegexSet classification rules.
///
/// `Class` - Type to hold classification results.
/// `Target` - Type of item to be classified.
/// `Context` - Anything to pass into each classifier in case it needs more context.
#[allow(dead_code)]
pub struct RegexSetClassificationRules<Target, Class, Context>
where
    Target: RegexSetClassifiable + ?Sized,
{
    pub empty_class_fn: Box<dyn Fn() -> Class>,
    pub classifiers: RegexSetClassifiers<Target, Class, Context>,
}

#[allow(dead_code)]
impl<Target, Class, Context> RegexSetClassificationRules<Target, Class, Context>
where
    Target: RegexSetClassifiable + ?Sized,
{
    /// Creates a new `ClassificationRules` instance.
    pub fn new(
        empty_class_fn: Box<dyn Fn() -> Class>,
        classifier_regexs: ClassifiersTextVecFnInit<Target, Class, Context>,
    ) -> anyhow::Result<Self, regex::Error> {
        let classifiers: HashMap<String, (RegexSet, Classifier<Target, Class, Context>)> =
            classifier_regexs
                .into_iter()
                .map(|(key, (regexes, classifier))| {
                    (key, (RegexSet::new(regexes).unwrap(), classifier))
                })
                .collect();

        Ok(RegexSetClassificationRules {
            empty_class_fn,
            classifiers,
        })
    }

    /// Classifies a given item.
    pub fn classify(&self, item: &Target, ctx: &Context) -> (Class, bool, usize) {
        let mut class = (self.empty_class_fn)();
        let mut interrupted = false;
        let mut classified_count = 0;

        for (key, (regex_set, classifier)) in &self.classifiers {
            if item.is_match(regex_set) {
                let proceed = classifier(&mut class, item, ctx, key);
                if !proceed {
                    interrupted = true;
                    break;
                }
                classified_count += 1;
            }
        }

        (class, interrupted, classified_count)
    }
}

/// Struct for managing closure-based classification rules. Unlike the
/// GlobSetClassifier and other classifiers this object allows traversal
/// over arbitrary number of named classification functions.
///
/// `Class` - Type to hold classification results.
/// `Target` - Type of item to be classified.
/// `Context` - Anything to pass into each classifier in case it needs more context.
#[allow(dead_code)]
pub struct FnClassificationRules<Target, Class, Context> {
    pub empty_class_fn: Box<dyn Fn() -> Class>,
    pub classifiers: ClassifiersFnInit<Target, Class, Context>,
}

#[allow(dead_code)]
impl<Target, Class, Context> FnClassificationRules<Target, Class, Context> {
    /// Creates a new `ClassificationRules` instance.
    pub fn new(
        empty_class_fn: Box<dyn Fn() -> Class>,
        classifiers: ClassifiersFnInit<Target, Class, Context>,
    ) -> anyhow::Result<Self, regex::Error> {
        Ok(FnClassificationRules {
            empty_class_fn,
            classifiers,
        })
    }

    /// Classifies a given item.
    pub fn classify(&self, item: &Target, ctx: &Context) -> (Class, bool, usize) {
        let mut class = (self.empty_class_fn)();
        let mut interrupted = false;
        let mut classified_count = 0;

        for (key, classifier) in &self.classifiers {
            let proceed = classifier(&mut class, item, ctx, key);
            if !proceed {
                interrupted = true;
                break;
            }
            classified_count += 1;
        }

        (class, interrupted, classified_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::Path;

    impl GlobSetClassifiable for Path {
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
    fn test_globset_classification() {
        let empty_class_fn = Box::new(|| PathClassification {
            is_executable: false,
        });
        let candidates_globs = vec![String::from("*")];
        let classifier_globs = HashMap::from([(
            String::from("executable"),
            (vec![String::from("*.exe")], executable_classifier()),
        )]);

        let classification_rules =
            GlobSetClassificationRules::<Path, PathClassification, bool>::new(
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

    fn _test_regexset_classification() {
        // TODO
    }

    fn _test_regex_classification() {
        // TODO
    }

    fn _test_fn_classification() {
        // TODO
    }
}
