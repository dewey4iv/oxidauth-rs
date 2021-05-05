CREATE TABLE user_permission_grants (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    realm_id UUID NOT NULL,
    user_id UUID NOT NULL,
    permission_id UUID NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT user_permission_grants_realms_fk FOREIGN KEY(realm_id) REFERENCES realms(id),
    CONSTRAINT user_permission_grants_users_fk FOREIGN KEY(user_id) REFERENCES users(id),
    CONSTRAINT user_permission_grants_permissions_fk FOREIGN KEY(permission_id) REFERENCES permissions(id)
);

CREATE INDEX user_permission_grants_realm_id_idx ON user_permission_grants(realm_id);
CREATE INDEX user_permission_grants_user_id_idx ON user_permission_grants(user_id);
CREATE INDEX user_permission_grants_permission_id_idx ON user_permission_grants(permission_id);
