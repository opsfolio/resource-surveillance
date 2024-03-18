
## How to Publish a New Release for `surveilr`

1. Compile a comprehensive list of new features, bug fixes, enhancements, and detailed instructions for usage.
2. Run `just test-regression` to run all regression tests to make sure nothing broke.
3. Bump the version.
4. Run `just help-markdown` to generate CLI docs.
5. Navigate to the release page on GitHub to initiate a new release.
6. Increment the minor number in the semver to generate a new tag for the release.
7. Use the "Generate Release Notes" feature on GitHub to create an initial draft of the release notes.
8. Copy the documentation prepared in the first step with the generated release notes by pasting it at the beginning of the auto-generated content.
9. Thoroughly examine the combined release notes to ensure accuracy and completeness.
10. Complete the process by clicking on “Publish Release”, to activate the release action in the workflow.
