CREATE TABLE users
(
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username      TEXT UNIQUE NOT NULL,
    password_hash TEXT        NOT NULL,
    display_name  TEXT
);

CREATE TABLE roles
(
    id   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT UNIQUE NOT NULL
);

CREATE TABLE user_role
(
    user_id UUID REFERENCES users (id) ON DELETE CASCADE,
    role_id UUID REFERENCES roles (id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

CREATE TABLE categories
(
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    id_category UUID REFERENCES categories (id) ON DELETE CASCADE,

    name        TEXT NOT NULL
);

CREATE TABLE channels
(
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    id_category UUID REFERENCES categories (id) ON DELETE CASCADE,

    type        TEXT NOT NULL CHECK (type IN ('TEXT', 'VOICE')),
    name        TEXT NOT NULL
);