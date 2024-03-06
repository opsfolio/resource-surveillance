WITH expected_counts AS (
    SELECT
        'device' AS table_name, 1 AS expected_count
    UNION ALL
    SELECT 'uniform_resource', 148
    UNION ALL
    SELECT 'ur_ingest_session', 1
    UNION ALL
    SELECT 'ur_ingest_session_imap_account', 1
    UNION ALL
    SELECT 'ur_ingest_session_imap_acct_folder', 6
    UNION ALL
    SELECT 'ur_ingest_session_imap_acct_folder_message', 37
    UNION ALL
    SELECT 'uniform_resource_transform', 37
),
actual_counts AS (
    SELECT 'device' AS table_name, COUNT(*) AS actual_count FROM device
    UNION ALL
    SELECT 'uniform_resource', COUNT(*) FROM uniform_resource
    UNION ALL
    SELECT 'ur_ingest_session', COUNT(*) FROM ur_ingest_session
    UNION ALL
    SELECT 'ur_ingest_session_imap_account', COUNT(*) FROM ur_ingest_session_imap_account
    UNION ALL
    SELECT 'ur_ingest_session_imap_acct_folder', COUNT(*) FROM ur_ingest_session_imap_acct_folder
     UNION ALL
    SELECT 'ur_ingest_session_imap_acct_folder_message', COUNT(*) FROM ur_ingest_session_imap_acct_folder_message
    UNION ALL
    SELECT 'uniform_resource_transform', COUNT(*) FROM uniform_resource_transform
),
results AS (
    SELECT
        ec.table_name,
        CASE
            WHEN ac.actual_count = ec.expected_count THEN 'ok - ' || ec.table_name || ' count matches expected (' || ec.expected_count || ')'
            ELSE 'not ok - ' || ec.table_name || ' count does not match expected (' || ec.expected_count || '), found: ' || ac.actual_count
        END AS tap_output
    FROM
        expected_counts ec
    JOIN
        actual_counts ac ON ec.table_name = ac.table_name
),
tap_plan AS (
    SELECT '1..' || COUNT(*) || ' - TAP tests for database tables' AS tap_output FROM results
)
SELECT tap_output FROM tap_plan
UNION ALL
SELECT tap_output FROM results;
