WITH RECURSIVE json_tree AS (
  SELECT
    uniform_resource_transform_id,
    json_each.key,
    json_each.value
  FROM
    uniform_resource_transform,
    json_each(uniform_resource_transform.content)
)
SELECT
  value AS a_tag_href
FROM
  json_tree
WHERE
  key = 'href';