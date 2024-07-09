WITH TmpDelta(delta, id, cfg_hash) AS (
    SELECT
        delta,
        id,
        cfg_hash
    FROM
        Deltas
    WHERE
        cfg_hash = (
            SELECT
                cfg_hash
            FROM
                BaseCfgs
            WHERE
                name = $1
                AND version = $2
        )
)
SELECT
    delta AS "delta: Value", cfg_hash
FROM
    TmpDelta
WHERE
    id = (
        SELECT
            MAX(id)
        FROM
            TmpDelta
    );
