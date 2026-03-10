CREATE TABLE score_event (
    id SERIAL PRIMARY KEY,
    chatter_id varchar(16) NOT NULL,
    channel_id varchar(16) NOT NULL,
    earned_at timestamp DEFAULT now() NOT NULL,

    CONSTRAINT score_event_chatter_fk 
        FOREIGN KEY(chatter_id) REFERENCES chatter(id),
    CONSTRAINT score_event_channel_fk 
        FOREIGN KEY(channel_id) REFERENCES channel(id)
);

CREATE INDEX idx_score_event_earned_at ON score_event(earned_at);
CREATE INDEX idx_score_event_chatter ON score_event(chatter_id);
CREATE INDEX idx_score_event_channel ON score_event(channel_id);

CREATE OR REPLACE FUNCTION increment_score_totals()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE chatter
    SET total = total + 1,
        updated_at = NOW()
    WHERE id = NEW.channel_id;

    UPDATE channel
    SET channel_total = channel_total + 1,
        updated_at = NOW()
    WHERE id = NEW.channel_id;

    INSERT INTO score (chatter_id, channel_id, score, updated_at)
    VALUES (NEW.chatter_id, NEW.channel_id, 1, NOW())
    ON CONFLICT (chatter_id, channel_id)
    DO UPDATE SET
        score = score.score + 1,
        updated_at = NOW();

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER score_event_increment_trigger
AFTER INSERT ON score_event
FOR EACH ROW 
EXECUTE FUNCTION increment_score_totals();

-- recalculate a chatter's total
CREATE OR REPLACE FUNCTION recalc_chatter_total(chatter_id_param varchar(16))
RETURNS INT8 AS $$
DECLARE 
    new_total INT8;
BEGIN
    SELECT COALESCE(COUNT(*), 0) INTO new_total
    FROM score_event
    WHERE chatter_id = chatter_id_param;

    UPDATE chatter 
    SET total = new_total,
        updated_at = NOW()
    WHERE id = chatter_id_param;

    RETURN new_total;
END;
$$ LANGUAGE plpgsql;

-- recalculate a channel's total
CREATE OR REPLACE FUNCTION recalc_channel_total(channel_id_param varchar(16))
RETURNS INT8 AS $$
DECLARE 
    new_total INT8;
BEGIN
    SELECT COALESCE(COUNT(*), 0) INTO new_total
    FROM score_event
    WHERE channel_id = channel_id_param;

    UPDATE channel 
    SET channel_total = new_total,
        updated_at = NOW()
    WHERE id = channel_id_param;

    RETURN new_total;
END;
$$ LANGUAGE plpgsql;

-- recalculate a specific score
CREATE OR REPLACE FUNCTION recalc_score(
    chatter_id_param varchar(16),
    channel_id_param varchar(16)
) 
RETURNS INT8 AS $$
DECLARE 
    new_score INT8;
BEGIN
    SELECT COALESCE(COUNT(*), 0) INTO new_score
    FROM score_event
    WHERE chatter_id = chatter_id_param
    AND channel_id = channel_id_param;

    INSERT INTO score (chatter_id, channel_id, score, updated_at)
    VALUES (chatter_id_param, channel_id_param, new_score, NOW())
    ON CONFLICT (chatter_id, channel_id)
    DO UPDATE SET
        score = new_score,
        updated_at = NOW();

    RETURN new_score;
END;
$$ LANGUAGE plpgsql;
