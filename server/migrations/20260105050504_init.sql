-- db piss_fan_testing;

CREATE TABLE chatter (
	id varchar(16) PRIMARY KEY NOT NULL,
	login varchar(25) NOT NULL,
	name varchar(25) NOT NULL,
	color varchar(8) DEFAULT '#000000' NOT NULL,
	image varchar NOT NULL,
	total INT8 DEFAULT 0 NOT NULL,
	private boolean DEFAULT false NOT NULL,
	created_at timestamp DEFAULT now() NOT NULL,
	updated_at timestamp DEFAULT now() NOT NULL
);

CREATE TABLE channel (
	id varchar(16) NOT NULL,
	channel_total INT8 DEFAULT 0 NOT NULL,
	created_at timestamp DEFAULT now() NOT NULL,
	updated_at timestamp DEFAULT now() NOT NULL,
	CONSTRAINT channel_id_pk PRIMARY KEY(id)
);


CREATE TABLE score ( 
	chatter_id varchar(16) NOT NULL,
	channel_id varchar(16) NOT NULL,
	score INT8 DEFAULT 0 NOT NULL,
	created_at timestamp DEFAULT now() NOT NULL,
	updated_at timestamp DEFAULT now() NOT NULL,
	CONSTRAINT score_chatter_id_channel_id_pk 
    PRIMARY KEY(chatter_id, channel_id)
);

CREATE VIEW ranked_scores_view_chatters AS
SELECT 
    id,
    login,
    name,
    color,
    image,
    total,
    private,
    created_at, 
    updated_at,
    ROW_NUMBER() OVER (
        ORDER BY total DESC, updated_at ASC
    ) AS ranking
FROM chatter;

CREATE VIEW ranked_scores_view_channels AS
SELECT 
    b.id,
    c.login,
    c.name,
    c.color,
    c.image,
    b.channel_total,
    b.created_at,
    b.updated_at,
    ROW_NUMBER() OVER (
        ORDER BY b.channel_total DESC, b.updated_at ASC
    ) AS ranking
FROM channel b 
JOIN chatter c ON b.id = c.id;


CREATE VIEW ranked_scores_view_per_channel AS
SELECT 
    s.channel_id,
    s.chatter_id,
    s.score,
    s.created_at,
    s.updated_at,
    ROW_NUMBER() OVER (
        PARTITION BY s.channel_id
        ORDER BY s.score DESC, s.created_at ASC
    ) AS ranking
FROM score s;

CREATE VIEW chatter_leaderboard AS
SELECT 
    c.id,
    c.login,
    c.name,
    c.color,
    c.image,
    c.total,
    c.private,
    c.created_at,
    c.updated_at,
    ROW_NUMBER() OVER (
        ORDER BY
            c.total DESC,
            c.created_at ASC
    ) AS ranking
FROM chatter c;

CREATE VIEW channel_leaderboard AS
SELECT 
    ch.id,
    ch.created_at,
    ch.updated_at,
    c.name,
    c.login,
    c.color, 
    c.image,
    ch.channel_total as total_channel,
    c.total as total_chatter,
    ROW_NUMBER() OVER (
        ORDER BY
            ch.channel_total DESC,
            ch.created_at ASC
    ) AS ranking
FROM channel ch
JOIN chatter c ON ch.id = c.id;

CREATE OR REPLACE FUNCTION recalc_chatter_total(chatter_id_param varchar(16))
RETURNS INT8 AS $$
BEGIN
    UPDATE chatter
    SET 
        total = (SELECT COALESCE(SUM(score), 0) FROM score WHERE chatter_id = chatter_id_param),
        updated_at = NOW()
    WHERE id = chatter_id_param;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION recalc_channel_total(channel_id_param varchar(16))
RETURNS INT8 AS $$
BEGIN
    UPDATE channel
    SET
        total = (SELECT COALESCE(SUM(score), 0) FROM score WHERE channel_id = channel_id_param),
        updated_at = NOW()
    WHERE id = channel_id_param;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION get_chatter_rank(chatter_id_param varchar(16))
RETURNS INT8 AS $$
DECLARE
    chatter_total INT8;
    chatter_created timestamp;
    rank_result INT8;
BEGIN
    SELECT total, created_at INTO chatter_total, chatter_created
    FROM chatter
    WHERE id = chatter_id_param;

    IF NOT FOUND THEN 
        RETURN NULL;
    END IF;

    SELECT COUNT(*) + 1 INTO rank_result
    FROM chatter
    WHERE total > chatter_total 
        OR (total = chatter_total AND created_at < chatter_created);

    RETURN rank_result;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION get_channel_rank(channel_id_param varchar(16))
RETURNS INT8 AS $$
DECLARE
    chan_total INT8;
    chan_created timestamp;
    rank_result INT8;
BEGIN
    SELECT channel_total, created_at INTO chan_total, chan_created
    FROM channel
    WHERE id = channel_id_param;

    IF NOT FOUND THEN
        RETURN NULL;
    END IF;
    
    SELECT COUNT(*) + 1 INTO rank_result
    FROM channel
    WHERE channel_total > chan_total
        OR (total = channel_total AND created_at < channel_created);

    RETURN rank_result;
END;
$$ LANGUAGE plpgsql;
    
ALTER TABLE channel ADD CONSTRAINT channel_id_chatter_id_fk
FOREIGN KEY (id) REFERENCES public.chatter(id);

ALTER TABLE score ADD CONSTRAINT score_chatter_id_chatter_id_fk 
FOREIGN KEY (chatter_id) REFERENCES public.chatter(id);

ALTER TABLE score ADD CONSTRAINT score_channel_id_channel_id_fk 
FOREIGN KEY (channel_id) REFERENCES public.channel(id);

CREATE UNIQUE INDEX idx_chatter_id ON chatter USING btree (id);

CREATE INDEX idx_channel_channel_id_user_id ON channel USING btree (id);
CREATE INDEX idx_channel_ranks ON channel USING btree (id, channel_total DESC, created_at ASC);
CREATE INDEX idx_chatter_login ON chatter USING btree (login);
CREATE INDEX idx_score_chatter_id_channel_id ON score USING btree (chatter_id, channel_id);
CREATE INDEX idx_chatter_all_channel_ranks ON score USING btree (chatter_id, score DESC, created_at ASC);
CREATE INDEX idx_channel_all_chatter_ranks ON score USING btree (channel_id, score DESC, created_at ASC);
CREATE INDEX idx_chatter_ranking ON chatter USING btree (total,created_at);


