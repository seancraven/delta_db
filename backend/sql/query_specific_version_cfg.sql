WITH TmpDelta(delta, created_at, cfg_hash) AS (
    SELECT
        delta,
        created_at,
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
    created_at = (
        SELECT
            MAX(created_at)
        FROM
            TmpDelta
    );
