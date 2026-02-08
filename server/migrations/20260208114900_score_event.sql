CREATE TABLE score_event (
    id SERIAL PRIMARY KEY,
    chatter_id varchar(16) NOT NULL,
    channel_id varchar(16) NOT NULL,
    points INT8 NOT NULL,
    earned_at timestamp DEFAULT now() NOT NULL,
    CONSTRAINT score_event_chatter_fk 
        FOREIGN KEY(chatter_id) REFERENCES chatter(id),
    CONSTRAINT score_event_channel_fk 
        FOREIGN KEY(channel_id) REFERENCES channel(id)
);

CREATE INDEX idx_score_event_earned_at ON score_event(earned_at);
CREATE INDEX idx_score_event_chatter_channel ON score_event(chatter_id, channel_id);

CREATE VIEW weekly_leaderboard AS
SELECT 
    chatter_id,
    channel_id,
    SUM(points) as weekly_score
FROM score_event
WHERE earned_at >= date_trunc('week', CURRENT_TIMESTAMP)
GROUP BY chatter_id, channel_id;

CREATE VIEW daily_leaderboard AS
SELECT 
    chatter_id,
    channel_id,
    SUM(points) as daily_score
FROM score_event
WHERE earned_at >= date_trunc('day', CURRENT_TIMESTAMP)
GROUP BY chatter_id, channel_id;
