WITH expected_counts AS (
    SELECT
        'device' AS table_name, 1 AS expected_count
    UNION ALL
    SELECT 'uniform_resource', 13
    UNION ALL
    SELECT 'ur_ingest_session', 1
    UNION ALL
    SELECT 'ur_ingest_session_fs_path_entry', 18
),
actual_counts AS (
    SELECT 'device' AS table_name, COUNT(*) AS actual_count FROM device
    UNION ALL
    SELECT 'uniform_resource', COUNT(*) FROM uniform_resource
    UNION ALL
    SELECT 'ur_ingest_session', COUNT(*) FROM ur_ingest_session
    UNION ALL
    SELECT 'ur_ingest_session_fs_path_entry', COUNT(*) FROM ur_ingest_session_fs_path_entry
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
-- Generate TAP output, including test plan line at the beginning.
SELECT tap_output FROM tap_plan
UNION ALL
SELECT tap_output FROM results;
