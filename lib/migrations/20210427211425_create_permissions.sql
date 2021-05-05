CREATE TABLE permissions (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    realm VARCHAR(128) NOT NULL,
    resource VARCHAR(128) NOT NULL,
    action VARCHAR(128) NOT NULL,
    realm_id UUID NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT unique_grant_parts UNIQUE(realm, resource, action),
    CONSTRAINT permissions_realms_fk FOREIGN KEY(realm_id) REFERENCES realms(id)
);

CREATE INDEX permissions_realm_id_idx ON permissions(realm_id);
CREATE INDEX permissions_realm_idx ON permissions(realm);
CREATE INDEX permissions_resource_idx ON permissions(resource);
CREATE INDEX permissions_action_idx ON permissions(action);
