CREATE TABLE key_pairs (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    realm_id UUID NOT NULL,
    public_key TEXT not null,
    private_key TEXT not null,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT key_pairs_realm_fk FOREIGN KEY(realm_id) REFERENCES realms(id)
);

CREATE INDEX key_pairs_realm_id_idx ON key_pairs(realm_id);
