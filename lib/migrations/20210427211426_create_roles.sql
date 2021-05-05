CREATE TABLE roles (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    realm_id UUID NOT NULL,
    name VARCHAR(32) UNIQUE NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT roles_realms_fk FOREIGN KEY(realm_id) REFERENCES realms(id)
);

CREATE INDEX roles_real_id_idx ON roles(realm_id);
