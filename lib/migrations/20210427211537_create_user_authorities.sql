CREATE TABLE user_authorities (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL,
    authority_id UUID NOT NULL,
    realm_id UUID NOT NULL,
    params JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT user_authorities_realms_fk FOREIGN KEY(realm_id) REFERENCES realms(id)
);

CREATE INDEX user_authorities_user_id_idx ON user_authorities(user_id);
CREATE INDEX user_authorities_authority_id_idx ON user_authorities(authority_id);
CREATE INDEX user_authorities_realm_id_idx ON user_authorities(realm_id);
