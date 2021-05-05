--  CREATE TYPE strategy_type AS ENUM('username_password');

CREATE TABLE authorities (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    realm_id UUID NOT NULL,
    client_key UUID UNIQUE DEFAULT uuid_generate_v4(),
    name VARCHAR(32) UNIQUE NOT NULL,
    status VARCHAR(32) NOT NULL,
    strategy VARCHAR(32) NOT NULL,
    --  strategy strategy_type NOT NULL,
    params JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT authorities_realms_fk FOREIGN KEY(realm_id) REFERENCES realms(id)
);
