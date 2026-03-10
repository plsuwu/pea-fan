CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE INDEX IF NOT EXISTS chatter_login_trgm_idx 
ON chatter USING gist (login gist_trgm_ops);

SET pg_trgm.similarity_threshold = 0.15;
ALTER DATABASE piss_fan_testing SET pg_trgm.similarity_threshold = 0.15;

CREATE OR REPLACE FUNCTION search_chatter_by_login(search_query TEXT)
RETURNS TABLE (
    id TEXT,
    name TEXT,
    login TEXT,
    color TEXT,
    image TEXT,
    total INT8,
    ranking INT8,
    similarity_score REAL
) 
LANGUAGE sql STABLE
AS $$
    SELECT 
        id, 
        name, 
        login,
        color,
        image,
        total,
        ranking,
        similarity(login, search_query) AS similarity_score
    FROM ranked_scores_view_chatters 
    WHERE search_query % login
    ORDER BY similarity_score DESC;
$$;
