CREATE VIEW daily_leaderboard_channel AS
SELECT 
    e.channel_id,
    c.login,
    c.name,
    c.color,
    c.image,
    b.channel_total AS total,
    COUNT(*) AS daily_total
FROM score_event e
JOIN chatter c ON e.channel_id = c.id
JOIN channel b ON e.channel_id = b.id
WHERE e.earned_at >= date_trunc('day', CURRENT_TIMESTAMP)
GROUP BY e.channel_id, c.login, c.name, c.color, c.image, b.channel_total
ORDER BY daily_total DESC;

CREATE VIEW weekly_leaderboard_channel AS
SELECT 
    e.channel_id,
    c.login,
    c.name,
    c.color,
    c.image,
    b.channel_total AS total,
    COUNT(*) AS weekly_total
FROM score_event e
JOIN chatter c ON e.channel_id = c.id
JOIN channel b ON e.channel_id = b.id
WHERE e.earned_at >= date_trunc('week', CURRENT_TIMESTAMP)
GROUP BY e.channel_id, c.login, c.name, c.color, c.image, b.channel_total
ORDER BY weekly_total DESC;

CREATE VIEW monthly_leaderboard_channel AS
SELECT 
    e.channel_id,
    c.login,
    c.name,
    c.color,
    c.image,
    b.channel_total AS total,
    COUNT(*) AS weekly_total
FROM score_event e
JOIN chatter c ON e.channel_id = c.id
JOIN channel b ON e.channel_id = b.id
WHERE e.earned_at >= date_trunc('month', CURRENT_TIMESTAMP)
GROUP BY e.channel_id, c.login, c.name, c.color, c.image, b.channel_total
ORDER BY weekly_total DESC;

CREATE VIEW yearly_leaderboard_channel AS
SELECT 
    e.channel_id,
    c.login,
    c.name,
    c.color,
    c.image,
    b.channel_total AS total,
    COUNT(*) AS weekly_total
FROM score_event e
JOIN chatter c ON e.channel_id = c.id
JOIN channel b ON e.channel_id = b.id
WHERE e.earned_at >= date_trunc('year', CURRENT_TIMESTAMP)
GROUP BY e.channel_id, c.login, c.name, c.color, c.image, b.channel_total
ORDER BY weekly_total DESC;

CREATE OR REPLACE FUNCTION get_channel_daily_total(
    channel_id_param varchar(16)
)
RETURNS INT8 AS $$
BEGIN
    RETURN (
        SELECT COALESCE(COUNT(*), 0)
        FROM score_event
        WHERE channel_id = channel_id_param
        AND earned_at >= date_trunc('day', CURRENT_TIMESTAMP)
    );
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION get_channel_weekly_total(
    channel_id_param varchar(16)
)
RETURNS INT8 AS $$
BEGIN
    RETURN (
        SELECT COALESCE(COUNT(*), 0)
        FROM score_event
        WHERE channel_id = channel_id_param
        AND earned_at >= date_trunc('week', CURRENT_TIMESTAMP)
    );
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION get_channel_monthly_total(
    channel_id_param varchar(16)
)
RETURNS INT8 AS $$
BEGIN
    RETURN (
        SELECT COALESCE(COUNT(*), 0)
        FROM score_event
        WHERE channel_id = channel_id_param
        AND earned_at >= date_trunc('month', CURRENT_TIMESTAMP)
    );
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION get_channel_yearly_total(
    channel_id_param varchar(16)
)
RETURNS INT8 AS $$
BEGIN
    RETURN (
        SELECT COALESCE(COUNT(*), 0)
        FROM score_event
        WHERE channel_id = channel_id_param
        AND earned_at >= date_trunc('year', CURRENT_TIMESTAMP)
    );
END;
$$ LANGUAGE plpgsql;
