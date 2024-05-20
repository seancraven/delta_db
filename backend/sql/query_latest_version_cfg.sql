WITH TmpDelta(delta, id, cfg_hash) AS (
    SELECT
        delta,
        id,
        cfg_hash
    FROM
        Deltas
    WHERE
        cfg_hash = (
            WITH HashVer(cfg_hash, version) AS (
                SELECT
                    cfg_hash,
                    version
                FROM
                    BaseCfgs
                WHERE
                    name = $1
            )
            SELECT
                cfg_hash
            FROM
                HashVer
            WHERE
                version = (
                    SELECT
                        MAX(version)
                    FROM
                        HashVer
                )
            LIMIT
                1
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
