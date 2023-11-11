# Governance, Planning, and Management for `surveilr`

## Part 1: Outcomes-Focused Labels for GitHub Issues

Using outcomes-focused labels in GitHub issues allows for prioritizing tasks
based on their impact and aligns the development work with strategic goals. Here
are some recommended labels:

- **Security Impact**: Labels such as "Blocker Security Risk" and "Threat
  Surface Increase" for prioritizing security-related issues.
- **Performance Enhancement**: Tags like "Performance" and "Resource
  Optimization" for performance-related tasks.
- **Usability Improvement**: For user experience enhancements, use labels such
  as `OpEX` (operator experience) or `UX` (user experience) or `DX` (developer
  experience).
- **Compliance**: Labels like "Legislative Compliance" when tied to a law,
  "Regulatory Compliance" for executive branch regulations, "Standards
  Compliance" when a compliance is required for a specific external standard,
  "Privacy Compliance" for encryption and other data privacy issues.
- **Reliability Increase**: Tags like "Stability", "Data Reliability" and "Fault
  Tolerance" for reliability-focused tasks.

Only use label for expressing the impact of the issue. Do not use labels to
merely categorize the issue because searching using text is easier than
searching by labels. Also, we never want to assign work based on arbitrary
categories but expected outcomes and desired results.

## Part 2: Aligning with Conventional Commits

Aligning GitHub issue labels with Conventional Commit messages enhances the
traceability and readability of changes. Below are some examples that should be
updated based on the final selection of tags/labels defined above.

1. **Security Impact**:
   `fix(security): [Blocker Risk/Threat Surface] - brief description`
2. **Performance Enhancement**:
   `perf(enhancement): [Speed/Resource] - description`
3. **Usability Improvement**: `feat(usability): [OpEX/DX/UX] - description`
4. **Compliance**:
   `chore(compliance): [Legislative/Regulatory/Privacy/Standards] - description`
5. **Reliability Increase**:
   `fix(reliability): [Stability/Fault Tolerance] - description`
6. **Data Protection**:
   `feat(security): [Data Encryption/Privacy Compliance] - description`
