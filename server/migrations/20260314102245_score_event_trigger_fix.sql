CREATE OR REPLACE FUNCTION increment_score_totals()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE chatter
    SET total = total + 1,
        updated_at = NOW()
    WHERE id = NEW.chatter_id;

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
