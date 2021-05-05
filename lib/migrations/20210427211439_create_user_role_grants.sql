CREATE TABLE user_role_grants (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    realm_id UUID NOT NULL,
    user_id UUID NOT NULL,
    role_id UUID NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT user_role_grants_realms_fk FOREIGN KEY(realm_id) REFERENCES realms(id),
    CONSTRAINT user_role_grants_users_fk FOREIGN KEY(user_id) REFERENCES users(id),
    CONSTRAINT user_role_grants_roles_fk FOREIGN KEY(role_id) REFERENCES roles(id)
);

CREATE INDEX user_role_grants_realm_id_idx ON user_role_grants(realm_id);
CREATE INDEX user_role_grants_user_id_idx ON user_role_grants(user_id);
CREATE INDEX user_role_grants_role_id_idx ON user_role_grants(role_id);
