CREATE TABLE IF NOT EXISTS users (
    id Uuid PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name text NOT NULL UNIQUE, -- CHECK (name <> '')
    password_hash text NOT NULL,
    role text,
);

CREATE TABLE IF NOT EXISTS sessions (
    session_token BYTEA PRIMARY KEY,
    user_id integer REFERENCES users (id) ON DELETE CASCADE
);