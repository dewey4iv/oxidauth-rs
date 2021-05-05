CREATE TABLE role_role_grants (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    realm_id UUID NOT NULL,
    parent_id UUID NOT NULL,
    child_id UUID NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT role_role_grants_realms_fk FOREIGN KEY(realm_id) REFERENCES realms(id),
    CONSTRAINT role_role_grants_parent_fk FOREIGN KEY(parent_id) REFERENCES roles(id),
    CONSTRAINT role_role_grants_child_fk FOREIGN KEY(child_id) REFERENCES roles(id)
);

CREATE INDEX role_role_grants_realm_id_idx ON role_role_grants(realm_id);
CREATE INDEX role_role_grants_parent_id_idx ON role_role_grants(parent_id);
CREATE INDEX role_role_grants_child_id_idx ON role_role_grants(child_id);
