
### Executing shell tasks

Persist the output of the shell task via `surveilr ingest tasks`  in to the RSSD SQLite DB. example:


```bash
curl -sL https://raw.githubusercontent.com/opsfolio/resource-surveillance/main/support/tasks/typical/device-security.jsonl | surveilr ingest tasks
```