# uniform_resource

## Description

Records file system content information. On multiple executions,  
uniform_resource are inserted only if the the file content (see unique   
index for details). For historical logging, uniform_resource has foreign  
key references to both ur_walk_session and ur_walk_session_path  
tables to indicate which particular session and walk path the  
resourced was inserted during.

<details>
<summary><strong>Table Definition</strong></summary>

```sql
CREATE TABLE "uniform_resource" (
    "uniform_resource_id" ULID PRIMARY KEY NOT NULL,
    "device_id" ULID NOT NULL,
    "walk_session_id" ULID NOT NULL,
    "walk_path_id" ULID NOT NULL,
    "uri" TEXT NOT NULL,
    "content_digest" TEXT NOT NULL,
    "content" BLOB,
    "nature" TEXT,
    "size_bytes" INTEGER,
    "last_modified_at" INTEGER,
    "content_fm_body_attrs" TEXT CHECK(json_valid(content_fm_body_attrs) OR content_fm_body_attrs IS NULL),
    "frontmatter" TEXT CHECK(json_valid(frontmatter) OR frontmatter IS NULL),
    "elaboration" TEXT CHECK(json_valid(elaboration) OR elaboration IS NULL),
    "created_at" TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    "created_by" TEXT DEFAULT 'UNKNOWN',
    "updated_at" TIMESTAMP,
    "updated_by" TEXT,
    "deleted_at" TIMESTAMP,
    "deleted_by" TEXT,
    "activity_log" TEXT,
    FOREIGN KEY("device_id") REFERENCES "device"("device_id"),
    FOREIGN KEY("walk_session_id") REFERENCES "ur_walk_session"("ur_walk_session_id"),
    FOREIGN KEY("walk_path_id") REFERENCES "ur_walk_session_path"("ur_walk_session_path_id"),
    UNIQUE("device_id", "content_digest", "uri", "size_bytes", "last_modified_at")
)
```

</details>

## Columns

| Name                  | Type      | Default           | Nullable | Children                                                          | Parents                                         | Comment                                                 |
| --------------------- | --------- | ----------------- | -------- | ----------------------------------------------------------------- | ----------------------------------------------- | ------------------------------------------------------- |
| uniform_resource_id   | ULID      |                   | false    | [ur_walk_session_path_fs_entry](ur_walk_session_path_fs_entry.md) |                                                 | {"isSqlDomainZodDescrMeta":true,"isUlid":true}          |
| device_id             | ULID      |                   | false    |                                                                   | [device](device.md)                             | {"isSqlDomainZodDescrMeta":true,"isUlid":true}          |
| walk_session_id       | ULID      |                   | false    |                                                                   | [ur_walk_session](ur_walk_session.md)           | {"isSqlDomainZodDescrMeta":true,"isUlid":true}          |
| walk_path_id          | ULID      |                   | false    |                                                                   | [ur_walk_session_path](ur_walk_session_path.md) | {"isSqlDomainZodDescrMeta":true,"isUlid":true}          |
| uri                   | TEXT      |                   | false    |                                                                   |                                                 |                                                         |
| content_digest        | TEXT      |                   | false    |                                                                   |                                                 |                                                         |
| content               | BLOB      |                   | true     |                                                                   |                                                 | {"isSqlDomainZodDescrMeta":true,"isBlobText":true}      |
| nature                | TEXT      |                   | true     |                                                                   |                                                 |                                                         |
| size_bytes            | INTEGER   |                   | true     |                                                                   |                                                 |                                                         |
| last_modified_at      | INTEGER   |                   | true     |                                                                   |                                                 |                                                         |
| content_fm_body_attrs | TEXT      |                   | true     |                                                                   |                                                 | {"isSqlDomainZodDescrMeta":true,"isJsonText":true}      |
| frontmatter           | TEXT      |                   | true     |                                                                   |                                                 | {"isSqlDomainZodDescrMeta":true,"isJsonText":true}      |
| elaboration           | TEXT      |                   | true     |                                                                   |                                                 | {"isSqlDomainZodDescrMeta":true,"isJsonText":true}      |
| created_at            | TIMESTAMP | CURRENT_TIMESTAMP | true     |                                                                   |                                                 |                                                         |
| created_by            | TEXT      | 'UNKNOWN'         | true     |                                                                   |                                                 |                                                         |
| updated_at            | TIMESTAMP |                   | true     |                                                                   |                                                 |                                                         |
| updated_by            | TEXT      |                   | true     |                                                                   |                                                 |                                                         |
| deleted_at            | TIMESTAMP |                   | true     |                                                                   |                                                 |                                                         |
| deleted_by            | TEXT      |                   | true     |                                                                   |                                                 |                                                         |
| activity_log          | TEXT      |                   | true     |                                                                   |                                                 | {"isSqlDomainZodDescrMeta":true,"isJsonSqlDomain":true} |

## Constraints

| Name                                | Type        | Definition                                                                                                                              |
| ----------------------------------- | ----------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| uniform_resource_id                 | PRIMARY KEY | PRIMARY KEY (uniform_resource_id)                                                                                                       |
| - (Foreign key ID: 0)               | FOREIGN KEY | FOREIGN KEY (walk_path_id) REFERENCES ur_walk_session_path (ur_walk_session_path_id) ON UPDATE NO ACTION ON DELETE NO ACTION MATCH NONE |
| - (Foreign key ID: 1)               | FOREIGN KEY | FOREIGN KEY (walk_session_id) REFERENCES ur_walk_session (ur_walk_session_id) ON UPDATE NO ACTION ON DELETE NO ACTION MATCH NONE        |
| - (Foreign key ID: 2)               | FOREIGN KEY | FOREIGN KEY (device_id) REFERENCES device (device_id) ON UPDATE NO ACTION ON DELETE NO ACTION MATCH NONE                                |
| sqlite_autoindex_uniform_resource_2 | UNIQUE      | UNIQUE (device_id, content_digest, uri, size_bytes, last_modified_at)                                                                   |
| sqlite_autoindex_uniform_resource_1 | PRIMARY KEY | PRIMARY KEY (uniform_resource_id)                                                                                                       |
| -                                   | CHECK       | CHECK(json_valid(content_fm_body_attrs) OR content_fm_body_attrs IS NULL)                                                               |
| -                                   | CHECK       | CHECK(json_valid(frontmatter) OR frontmatter IS NULL)                                                                                   |
| -                                   | CHECK       | CHECK(json_valid(elaboration) OR elaboration IS NULL)                                                                                   |

## Indexes

| Name                                 | Definition                                                                                    |
| ------------------------------------ | --------------------------------------------------------------------------------------------- |
| idx_uniform_resource__device_id__uri | CREATE INDEX "idx_uniform_resource__device_id__uri" ON "uniform_resource"("device_id", "uri") |
| sqlite_autoindex_uniform_resource_2  | UNIQUE (device_id, content_digest, uri, size_bytes, last_modified_at)                         |
| sqlite_autoindex_uniform_resource_1  | PRIMARY KEY (uniform_resource_id)                                                             |

## Relations

![er](uniform_resource.svg)

---

> Generated by [tbls](https://github.com/k1LoW/tbls)