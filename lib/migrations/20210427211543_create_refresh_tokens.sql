CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL,
    realm_id UUID NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,


    CONSTRAINT refresh_tokens_realm_fk FOREIGN KEY(realm_id) REFERENCES realms(id)
);

CREATE INDEX refresh_tokens_user_id_idx ON refresh_tokens(user_id);
CREATE INDEX refresh_tokens_realm_id_idx ON refresh_tokens(realm_id);
CREATE INDEX refresh_tokens_expires_at_idx ON refresh_tokens(expires_at);
