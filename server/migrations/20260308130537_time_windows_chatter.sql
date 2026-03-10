CREATE VIEW daily_leaderboard_chatter AS
SELECT 
    e.chatter_id,
    c.login,
    c.name,
    c.color,
    c.image,
    c.total,
    COUNT(*) AS daily_total
FROM score_event e
JOIN chatter c ON e.chatter_id = c.id
WHERE e.earned_at >= date_trunc('day', CURRENT_TIMESTAMP)
GROUP BY e.chatter_id, c.login, c.name, c.color, c.image, c.total
ORDER BY daily_total DESC;

CREATE VIEW weekly_leaderboard_chatter AS
SELECT 
    e.chatter_id,
    c.login,
    c.name,
    c.color,
    c.image,
    c.total,
    COUNT(*) AS weekly_total
FROM score_event e
JOIN chatter c ON e.chatter_id = c.id
WHERE e.earned_at >= date_trunc('week', CURRENT_TIMESTAMP)
GROUP BY e.chatter_id, c.login, c.name, c.color, c.image, c.total
ORDER BY weekly_total DESC;

CREATE VIEW monthly_leaderboard_chatter AS
SELECT 
    e.chatter_id,
    c.login,
    c.name,
    c.color,
    c.image,
    c.total,
    COUNT(*) AS weekly_total
FROM score_event e
JOIN chatter c ON e.chatter_id = c.id
WHERE e.earned_at >= date_trunc('month', CURRENT_TIMESTAMP)
GROUP BY e.chatter_id, c.login, c.name, c.color, c.image, c.total
ORDER BY weekly_total DESC;

CREATE VIEW yearly_leaderboard_chatter AS
SELECT 
    e.chatter_id,
    c.login,
    c.name,
    c.color,
    c.image,
    c.total,
    COUNT(*) AS weekly_total
FROM score_event e
JOIN chatter c ON e.chatter_id = c.id
WHERE e.earned_at >= date_trunc('year', CURRENT_TIMESTAMP)
GROUP BY e.chatter_id, c.login, c.name, c.color, c.image, c.total
ORDER BY weekly_total DESC;

CREATE OR REPLACE FUNCTION get_chatter_daily_total(
    chatter_id_param varchar(16)
)
RETURNS INT8 AS $$
BEGIN
    RETURN (
        SELECT COALESCE(COUNT(*), 0)
        FROM score_event
        WHERE chatter_id = chatter_id_param
        AND earned_at >= date_trunc('day', CURRENT_TIMESTAMP)
    );
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION get_chatter_weekly_total(
    chatter_id_param varchar(16)
)
RETURNS INT8 AS $$
BEGIN
    RETURN (
        SELECT COALESCE(COUNT(*), 0)
        FROM score_event
        WHERE chatter_id = chatter_id_param
        AND earned_at >= date_trunc('week', CURRENT_TIMESTAMP)
    );
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION get_chatter_monthly_total(
    chatter_id_param varchar(16)
)
RETURNS INT8 AS $$
BEGIN
    RETURN (
        SELECT COALESCE(COUNT(*), 0)
        FROM score_event
        WHERE chatter_id = chatter_id_param
        AND earned_at >= date_trunc('month', CURRENT_TIMESTAMP)
    );
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION get_chatter_yearly_total(
    chatter_id_param varchar(16)
)
RETURNS INT8 AS $$
BEGIN
    RETURN (
        SELECT COALESCE(COUNT(*), 0)
        FROM score_event
        WHERE chatter_id = chatter_id_param
        AND earned_at >= date_trunc('year', CURRENT_TIMESTAMP)
    );
END;
$$ LANGUAGE plpgsql;
