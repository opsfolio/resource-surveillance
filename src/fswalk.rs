use std::collections::HashMap;

use bitflags::bitflags;
use globset::{Glob, GlobSet, GlobSetBuilder};
use is_executable::IsExecutable;

bitflags! {
    /// A set of flags to be applied to file system entries.
    pub struct DirEntryFlags: u32 {
        const EXECUTABLE_IN_GLOB = 0b00000001;
        const EXECUTABLE_IN_FS = 0b00000010;

        const IGNORE_IN_GLOB = 0b00000100;
    }
}

pub type DirEntryClassifier<Class> =
    Box<dyn Fn(&mut Class, &ignore::DirEntry, &str, &str) -> bool + Sync + Send>;

pub fn empty_dir_entry_flags() -> Box<dyn Fn() -> DirEntryFlags> {
    Box::new(DirEntryFlags::empty)
}

/// Return a FlaggableDirEntryFn that will check a DirEntry path has proper file system
/// permissions to execute a file.
pub fn _flag_executable_in_fs() -> DirEntryClassifier<DirEntryFlags> {
    Box::new(|flags, entry, _root_path, _purpose| -> bool {
        if entry.path().is_executable() {
            *flags |= DirEntryFlags::EXECUTABLE_IN_GLOB | DirEntryFlags::EXECUTABLE_IN_FS;
        } else {
            *flags |= DirEntryFlags::EXECUTABLE_IN_GLOB;
        }
        true
    })
}

/// Return a FlaggableDirEntryFn that will just set the IGNORE flag when encountered.
pub fn _flag_ignore() -> DirEntryClassifier<DirEntryFlags> {
    Box::new(|flags, _entry, _root_path, _purpose| -> bool {
        *flags |= DirEntryFlags::IGNORE_IN_GLOB;
        true
    })
}

/// A structure to walk through file system directories and apply flags to entries based on glob patterns.
pub struct ClassifiableFileSysEntries<Class> {
    empty_class_fn: Box<dyn Fn() -> Class>,
    include_hidden: bool,
    globset_prime: GlobSet,
    flaggable_globs: HashMap<String, (GlobSet, DirEntryClassifier<Class>)>,
}

impl<Class> ClassifiableFileSysEntries<Class> {
    /// Constructs a new `ClassifiableFileSysEntries`.
    ///
    /// # Arguments
    ///
    /// * `globs_prime` - Primary glob patterns to filter the files.
    /// * `flaggable_globs_map` - A map of named glob patterns to their corresponding flags and flagging functions.
    ///
    /// # Returns
    ///
    /// A result containing the new instance or an error if the glob patterns are invalid.
    pub fn new(
        empty_class_fn: Box<dyn Fn() -> Class>,
        globs_prime: &[String],
        flaggable_globs_map: HashMap<String, (Vec<String>, DirEntryClassifier<Class>)>,
        include_hidden: bool,
    ) -> anyhow::Result<Self, globset::Error> {
        let mut builder = GlobSetBuilder::new();
        for glob_pattern in globs_prime {
            builder.add(Glob::new(glob_pattern)?);
        }
        let globset_prime = builder.build()?;

        let flaggable_globs = flaggable_globs_map
            .into_iter()
            .map(|(key, (globs, flag_fn))| {
                let mut builder = GlobSetBuilder::new();
                for glob_pattern in globs {
                    builder.add(Glob::new(&glob_pattern).unwrap());
                }
                let glob_set = builder.build().unwrap();
                (key, (glob_set, flag_fn))
            })
            .collect();

        Ok(ClassifiableFileSysEntries {
            empty_class_fn,
            include_hidden,
            globset_prime,
            flaggable_globs,
        })
    }

    /// Walks through the file system based on the specified root paths and applies flags to each entry.
    /// This code will recursively traverse the current directory while automatically filtering out hidden
    /// files and directories plus any entries according to ignore globs found in files like `.ignore`
    /// and `.gitignore`.
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
                        if self.globset_prime.is_match(entry.path()) {
                            let (class, _processed_all) = self.classify(root_path, &entry);
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

    /// Calculates flags for a given directory entry based on the flaggable glob patterns.
    ///
    /// # Arguments
    ///
    /// * `root_path` - The root path of the file system walk.
    /// * `entry` - The directory entry to calculate flags for.
    ///
    /// # Returns
    ///
    /// A tuple containing the combined flags for the directory entry and a boolean indicating whether to continue processing.
    fn classify(&self, root_path: &str, entry: &ignore::DirEntry) -> (Class, bool) {
        let mut combined_flags = self.empty_class_fn.as_ref()();
        let mut processed_all = true;

        for (key, (glob_set, flag_fn)) in &self.flaggable_globs {
            if glob_set.is_match(entry.path()) {
                processed_all = flag_fn(&mut combined_flags, entry, root_path, key);
                if !processed_all {
                    break;
                }
            }
        }

        (combined_flags, processed_all)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Return a FlaggableDirEntryFn that will just set the EXECUTABLE flag when encountered.
    pub fn flag_executable() -> DirEntryClassifier<DirEntryFlags> {
        Box::new(|flags, _entry, _root_path, _purpose| -> bool {
            *flags |= DirEntryFlags::EXECUTABLE_IN_GLOB;
            true
        })
    }

    #[test]
    fn test_flag_calculation() {
        let flaggable_globs = HashMap::from([(
            "executables".to_string(),
            (vec!["*.sh".to_string()], flag_executable()),
        )]);

        let fs_resources = ClassifiableFileSysEntries::new(
            empty_dir_entry_flags(),
            &["*".to_string()],
            flaggable_globs,
            false,
        )
        .unwrap();

        let walk_paths = vec!["./".to_string()];
        let mut flags_set = false;

        fs_resources.walk(&walk_paths, |_root_path, _entry, flags| {
            flags_set = flags.contains(DirEntryFlags::EXECUTABLE_IN_GLOB);
        });

        // TODO: figure out how to test this module using a virtual file sys
        // assert!(flags_set);
    }
}
