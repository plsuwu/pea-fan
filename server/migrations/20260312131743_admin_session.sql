CREATE TABLE session (
    id SERIAL PRIMARY KEY,
    token VARCHAR(64) NOT NULL,
    created_at timestamp DEFAULT NOW() NOT NULL,
    expires_at timestamp DEFAULT NOW() + '14 days'
);

CREATE TABLE reply (
	id varchar(16) NOT NULL,
    enabled BOOLEAN DEFAULT FALSE NOT NULL,
	updated_at timestamp DEFAULT now() NOT NULL,
	CONSTRAINT reply_channel_id_pk PRIMARY KEY(id)
);

CREATE VIEW replies_and_chatter_data AS
SELECT 
    r.id,
    r.enabled,
    c.login,
    c.name,
    c.color,
    c.image
FROM reply r 
JOIN chatter c ON r.id = c.id;
