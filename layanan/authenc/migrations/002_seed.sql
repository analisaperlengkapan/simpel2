-- F5-B squashed seed (pg_dump --data-only of the repaired set). master realm,
-- seed user 199203142014031001, roles, role_permissions, role/access/admin-level
-- reference data, oauth2_clients(perlengkapan). Runner version-tracking = idempotent.

--
-- PostgreSQL database dump
--


-- Dumped from database version 15.18
-- Dumped by pg_dump version 15.18

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', 'public, authenc', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Data for Name: realms; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.realms (id, name, display_name, description, enabled, created_at, updated_at, deleted_at, ssl_required, registration_allowed, registration_email_as_username, remember_me, verify_email, login_with_email_allowed, duplicate_emails_allowed, reset_password_allowed, edit_username_allowed, brute_force_protected, max_failure_wait_seconds, minimum_quick_login_wait_seconds, wait_increment_seconds, quick_login_check_milli_seconds, max_delta_time_seconds, failure_factor, default_signature_algorithm, revoke_refresh_token, refresh_token_max_reuse, access_token_lifespan, access_token_lifespan_for_implicit_flow, sso_session_idle_timeout, sso_session_max_lifespan, sso_session_idle_timeout_remember_me, sso_session_max_lifespan_remember_me, offline_session_idle_timeout, offline_session_max_lifespan, client_session_idle_timeout, client_session_max_lifespan, access_code_lifespan, access_code_lifespan_user_action, access_code_lifespan_login, action_token_generated_by_admin_lifespan, action_token_generated_by_user_lifespan, oauth2_device_code_lifespan, oauth2_device_polling_interval, attributes) VALUES ('00000000-0000-0000-0000-000000000000', 'master', 'Master Realm', 'Default master realm', true, '2026-06-09 09:23:17.185556+00', '2026-06-09 09:23:17.185556+00', NULL, 'external', false, false, true, false, true, false, true, false, true, 900, 60, 60, 1000, 43200, 30, 'RS256', false, 0, 300, 900, 1800, 36000, 0, 0, 2592000, 5184000, 0, 0, 60, 300, 1800, 43200, 300, 600, 5, NULL);


--
-- Data for Name: users; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.users (id, username, email, password_hash, email_verified, enabled, realm_id, federated, created_at, updated_at, deleted_at, last_login_at, login_count, mfa_enabled, mfa_setup_at, mfa_last_used, satker_code, require_mfa_setup, password_changed_at, password_expires_at, require_password_change, password_history_count, first_name, last_name, nip, nama, jabatan, phone_number, phone_verified, organization_id, totp_secret, totp_backup_codes, webauthn_enabled, account_locked, account_locked_until, failed_login_attempts, last_failed_login_at, attributes) VALUES ('00000000-0000-0000-0000-000000000001', 'admin', 'admin@authenc.local', '$argon2id$v=19$m=19456,t=2,p=1$YWJjZGVmZ2hpams$MTIzNDU2Nzg5MDEyMzQ1Njc4OTA=', true, true, '00000000-0000-0000-0000-000000000000', false, '2026-06-09 09:23:17.185556+00', '2026-06-09 09:23:18.003926+00', NULL, NULL, 0, false, NULL, NULL, 'UNKNOWN', false, '2026-06-09 09:23:17.185556+00', NULL, false, 0, NULL, NULL, NULL, NULL, NULL, NULL, false, NULL, NULL, NULL, false, false, NULL, 0, NULL, NULL);
INSERT INTO public.users (id, username, email, password_hash, email_verified, enabled, realm_id, federated, created_at, updated_at, deleted_at, last_login_at, login_count, mfa_enabled, mfa_setup_at, mfa_last_used, satker_code, require_mfa_setup, password_changed_at, password_expires_at, require_password_change, password_history_count, first_name, last_name, nip, nama, jabatan, phone_number, phone_verified, organization_id, totp_secret, totp_backup_codes, webauthn_enabled, account_locked, account_locked_until, failed_login_attempts, last_failed_login_at, attributes) VALUES ('274c04e6-a811-4fe8-a940-13f0f1098a97', '199203142014031001', '199203142014031001@kejaksaan.go.id', '$argon2id$v=19$m=65536,t=3,p=4$TG0TRGGPnVrMiDnG2RfqeQ$wwhai83/MyAlKcB8W4XLHj5iSa5ATcB/DJ/a6/5zg6M', true, true, '00000000-0000-0000-0000-000000000000', false, '2026-06-09 09:23:18.568707+00', '2026-06-09 09:23:18.609935+00', NULL, NULL, 0, false, NULL, NULL, '0100000', false, NULL, NULL, true, 0, NULL, NULL, '199203142014031001', 'Admin Perlengkapan', 'Kasubag Perlengkapan', NULL, false, NULL, NULL, NULL, false, false, NULL, 0, NULL, NULL);


--
-- Data for Name: mfa_backup_codes; Type: TABLE DATA; Schema: authenc; Owner: -
--



--
-- Data for Name: token_revocations; Type: TABLE DATA; Schema: authenc; Owner: -
--



--
-- Data for Name: totp_secrets; Type: TABLE DATA; Schema: authenc; Owner: -
--



--
-- Data for Name: access_levels; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.access_levels (id, code, name, description, numeric_level, is_system, capabilities, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('2d3a4359-8917-4445-a038-32f972188f79', 'deny', 'No Access', 'Explicitly denied access', 0, true, '[]', '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.access_levels (id, code, name, description, numeric_level, is_system, capabilities, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('9c5898d4-c3ba-4414-baa0-48732a462900', 'read', 'Read Only', 'Read-only access', 10, true, '["read"]', '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.access_levels (id, code, name, description, numeric_level, is_system, capabilities, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('1f868187-7193-4639-9b2d-17a8db83ecc2', 'write', 'Read/Write', 'Read and write access', 20, true, '["read", "write"]', '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.access_levels (id, code, name, description, numeric_level, is_system, capabilities, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('4179e851-d3d2-4290-8a57-75a465deacf2', 'manage', 'Management', 'Full management access', 30, true, '["read", "write", "delete", "manage"]', '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.access_levels (id, code, name, description, numeric_level, is_system, capabilities, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('35febaef-7d7c-460e-8738-91adb15d4f49', 'root', 'Root Access', 'Unrestricted root access', 100, true, '["*"]', '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);


--
-- Data for Name: account_linking_requests; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: actor_types; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.actor_types (id, code, name, description, is_system, is_human, metadata, created_at, updated_at) VALUES ('ca6164ae-6645-4990-ae21-1fc0fe2c0aa9', 'user', 'Human User', 'Regular human user', true, true, '{}', '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00');
INSERT INTO public.actor_types (id, code, name, description, is_system, is_human, metadata, created_at, updated_at) VALUES ('feb871b2-7213-4785-96de-620b2146f647', 'system', 'System', 'System-initiated actions', true, false, '{}', '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00');
INSERT INTO public.actor_types (id, code, name, description, is_system, is_human, metadata, created_at, updated_at) VALUES ('7e084090-ddb5-4ca9-be0a-952d60963646', 'service', 'Service Account', 'Automated service account', true, false, '{}', '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00');
INSERT INTO public.actor_types (id, code, name, description, is_system, is_human, metadata, created_at, updated_at) VALUES ('ee166a42-2e58-4a9c-aa54-7e2466fe6597', 'automated', 'Automated Process', 'Automated background process', true, false, '{}', '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00');


--
-- Data for Name: admin_audit_log; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: admin_console_preferences; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: admin_console_sessions; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: admin_dashboard_metrics; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: admin_events; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: admin_level_types; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.admin_level_types (id, code, name, description, hierarchy_level, scope_type, parent_level_id, can_manage_levels, is_system, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('89e0f8cf-47bf-4b85-8ba7-92fc98427856', 'pusat', 'Admin Pusat', 'Central/national level administrator', 100, 'global', NULL, '[]', true, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.admin_level_types (id, code, name, description, hierarchy_level, scope_type, parent_level_id, can_manage_levels, is_system, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('386f2a90-28ee-47e8-b89e-ac6a202f5976', 'eselon_i', 'Admin Eselon I', 'Eselon I level administrator', 80, 'organization', NULL, '[]', true, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.admin_level_types (id, code, name, description, hierarchy_level, scope_type, parent_level_id, can_manage_levels, is_system, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('e2df03f9-d16b-4962-a849-6df27dc679d0', 'wilayah', 'Admin Wilayah', 'Regional area administrator', 60, 'group', NULL, '[]', true, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.admin_level_types (id, code, name, description, hierarchy_level, scope_type, parent_level_id, can_manage_levels, is_system, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('55f6e564-1c55-4052-8097-e31b3879d86a', 'satker', 'Admin Satker', 'Work unit administrator', 40, 'group', NULL, '[]', true, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);


--
-- Data for Name: admin_notifications; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: audit_integrity_checks; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: audit_integrity_failures; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: devices; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: oauth2_clients; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.oauth2_clients (id, client_id, client_secret_hash, client_name, client_type, redirect_uris, scopes, grant_types, response_types, token_endpoint_auth_method, owner_id, realm_id, enabled, created_at, updated_at, deleted_at, token_exchange_enabled, logo_uri, client_uri, policy_uri, tos_uri, jwks_uri, jwks, sector_identifier_uri, subject_type, id_token_signed_response_alg, id_token_encrypted_response_alg, id_token_encrypted_response_enc, userinfo_signed_response_alg, userinfo_encrypted_response_alg, userinfo_encrypted_response_enc, request_object_signing_alg, request_object_encryption_alg, request_object_encryption_enc, token_endpoint_auth_signing_alg, default_max_age, require_auth_time, default_acr_values, initiate_login_uri, request_uris, application_type, contacts, client_id_issued_at, client_secret_expires_at, software_id, software_version, registration_access_token_hash) VALUES ('1e1d4de9-7d1c-4739-8e39-483fa7dfd9f6', 'perlengkapan', '', 'SIMPEL Perlengkapan', 'public', '{/perlengkapan/*,/dashboard/*}', '{}', '{}', '{}', 'client_secret_basic', NULL, '00000000-0000-0000-0000-000000000000', true, '2026-06-09 09:23:18.568707+00', '2026-06-09 09:23:18.568707+00', NULL, false, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'public', 'EdDSA', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, false, NULL, NULL, NULL, 'web', NULL, NULL, NULL, NULL, NULL, NULL);


--
-- Data for Name: user_sessions; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: audit_logs; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: authentication_flows; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: authentication_executions; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: authentication_sessions; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: authenticator_configs; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: authenticator_executions; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: authenticator_execution_results; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: authorization_policies; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.authorization_policies (id, name, description, policy_type, effect, path_pattern, capabilities, conditions, priority, is_system, enabled, realm_id, created_at, updated_at, deleted_at) VALUES ('e7d90798-49ff-42dc-ba6f-3536968dd627', 'root', 'Root policy - full access', 'acl', 'allow', '*', '["*"]', '{}', 1000, true, true, NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.authorization_policies (id, name, description, policy_type, effect, path_pattern, capabilities, conditions, priority, is_system, enabled, realm_id, created_at, updated_at, deleted_at) VALUES ('f27e3318-ea45-4205-820f-cd7e03c2cd62', 'default', 'Default read-only policy', 'acl', 'allow', 'self/*', '["read"]', '{}', 10, true, true, NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.authorization_policies (id, name, description, policy_type, effect, path_pattern, capabilities, conditions, priority, is_system, enabled, realm_id, created_at, updated_at, deleted_at) VALUES ('2adde885-0c15-4f56-9da3-5e592aade756', 'deny-all', 'Explicit deny-all policy', 'acl', 'deny', '*', '["*"]', '{}', 0, true, true, NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);


--
-- Data for Name: capabilities; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('14ec8561-d3a7-4d7c-8882-765f2857dd91', '*', 'Superuser', 'All capabilities (root)', NULL, '*', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('09f9f2f0-d966-4b1f-a6a4-c32e2c571c69', 'users:read', 'Read Users', 'View user information', 'user', 'read', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('770ee6a8-aa16-4cee-9789-ba27700fc98f', 'users:write', 'Write Users', 'Create and update users', 'user', 'write', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('abed8c26-1000-44d0-87a0-6d0b94962b90', 'users:delete', 'Delete Users', 'Delete users', 'user', 'delete', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('77536a7c-ff0e-47a4-98ef-ea6308233f65', 'users:admin', 'Admin Users', 'Full user administration', 'user', 'admin', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('54c32177-1cb2-4d42-9997-45314cdd3b2d', 'roles:read', 'Read Roles', 'View role information', 'role', 'read', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('4eed1b49-f56d-42e9-b111-6cec38dd9e6f', 'roles:write', 'Write Roles', 'Create and update roles', 'role', 'write', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('423bbc98-f93d-4897-b786-8254ef70eb10', 'roles:delete', 'Delete Roles', 'Delete roles', 'role', 'delete', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('b4ecac53-cb5b-4c16-b2ca-43b32891a516', 'roles:assign', 'Assign Roles', 'Assign roles to users', 'role', 'assign', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('68eb5bc5-e9b7-40a1-b6a0-b56a2730d5cf', 'policies:read', 'Read Policies', 'View policies', 'policy', 'read', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('aae797ff-45a2-4266-a1b7-ddf5393687b6', 'policies:write', 'Write Policies', 'Create and update policies', 'policy', 'write', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('6e27f64b-283f-43f8-8524-2d829492755d', 'policies:delete', 'Delete Policies', 'Delete policies', 'policy', 'delete', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('ff4da924-96e6-4b21-8dbf-74411b4f84e4', 'realms:read', 'Read Realms', 'View realm configuration', 'realm', 'read', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('3f4291e8-0ce0-4537-853e-04c21792179f', 'realms:write', 'Write Realms', 'Modify realm configuration', 'realm', 'write', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('677541af-433a-426f-8878-d0cb07bafd17', 'realms:delete', 'Delete Realms', 'Delete realms', 'realm', 'delete', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('46f77b55-94c1-4c45-8a01-1b65c5dfa434', 'clients:read', 'Read Clients', 'View OAuth2 clients', 'client', 'read', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('c8b7534e-6377-4579-a5cf-e4ffa8e0e9c4', 'clients:write', 'Write Clients', 'Create and update clients', 'client', 'write', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('edf4dac1-3bd5-48b4-8cf4-e72e87f3fcc9', 'clients:delete', 'Delete Clients', 'Delete clients', 'client', 'delete', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('20a99777-3d9e-4467-820b-686805d49813', 'audit:read', 'View Audit Logs', 'View audit logs', 'audit', 'read', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('0dbc9454-e17b-4ec8-b208-86c77f80da5c', 'audit:export', 'Export Audit Logs', 'Export audit logs', 'audit', 'export', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('6014f9ee-58bd-4f82-9464-cf1669a004b6', 'config:read', 'Read Config', 'View system configuration', 'config', 'read', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('9ce57f44-d864-4b59-b387-a2555a4fd4ea', 'config:write', 'Write Config', 'Modify system configuration', 'config', 'write', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('fe7619d1-1774-4717-bbcc-edc4ebd9ba8d', 'config:delete', 'Delete Config', 'Delete configuration entries', 'config', 'delete', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('6712bae5-4b3b-44c1-b0ae-d6d40c534f15', 'config:reload', 'Reload Config', 'Hot reload configuration', 'config', 'admin', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('37ea1b6f-c430-4927-b0b5-0fb4d4ace648', 'mfa:manage', 'Manage MFA', 'Manage MFA settings', 'mfa', 'manage', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('cea0c0cc-a82b-4809-833b-a488bbf71f90', 'mfa:admin', 'Admin MFA', 'Full MFA administration', 'mfa', 'admin', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('fb0e9d1a-8ccc-4a62-af2b-b7605e30d9f8', 'mfa:bypass', 'Bypass MFA', 'Reset and bypass user MFA', 'mfa', 'bypass', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('b80919ca-7a1e-475f-8f8b-80eb428e8723', 'mfa:view', 'View MFA', 'View MFA status', 'mfa', 'read', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('cfb29e89-6055-4a11-98c1-86b761611fe7', 'mfa:reset', 'Reset MFA', 'Reset user MFA', 'mfa', 'reset', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('b46be5c1-be1e-443d-84ba-d2fe0c920735', 'secrets:read', 'Read Secrets', 'Read secrets from vault', 'secret', 'read', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('37fcb22e-a293-4835-bf2f-af2d026d5148', 'secrets:write', 'Write Secrets', 'Write secrets to vault', 'secret', 'write', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('4f2dd068-8e13-41eb-93a3-da386c4590c3', 'secrets:delete', 'Delete Secrets', 'Delete secrets from vault', 'secret', 'delete', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('854a5e0f-3111-4897-95d4-4b1d2f0ad43d', 'secrets:admin', 'Admin Secrets', 'Full secrets administration', 'secret', 'admin', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('d5c48c23-c9f7-4bd6-a9b3-a20574638740', 'system:admin', 'System Admin', 'Full system administration (superuser)', 'system', 'admin', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('b46c19fd-7393-4a25-b590-7f137d5767ee', 'system:health', 'System Health', 'View system health metrics', 'system', 'read', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('50165db7-461a-427a-ac9c-25bba9d64871', 'sessions:read', 'Read Sessions', 'View active sessions', 'session', 'read', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.capabilities (id, code, name, description, resource_type, action, is_system, is_dangerous, requires_mfa, requires_approval, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('988db5e8-4053-42a0-be16-de6596b1a91b', 'sessions:revoke', 'Revoke Sessions', 'Terminate user sessions', 'session', 'revoke', true, false, false, false, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);


--
-- Data for Name: captcha_challenges; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: captcha_behavioral_metrics; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: captcha_bot_detection_metrics; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: captcha_difficulty_adjustments; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: captcha_performance_metrics; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: captcha_security_event_metrics; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: captcha_type_effectiveness; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.captcha_type_effectiveness (id, challenge_type, usage_count, success_rate, avg_completion_time_secs, bot_detection_rate, user_satisfaction, effectiveness_score, last_updated, created_at) VALUES ('e803ebce-9cc7-44ad-bda9-369bf7e197c8', 'Visual', 0, 0.5000, 30.00, 0.5000, 0.5000, 0.5000, '2026-06-09 09:23:18.02714+00', '2026-06-09 09:23:18.02714+00');
INSERT INTO public.captcha_type_effectiveness (id, challenge_type, usage_count, success_rate, avg_completion_time_secs, bot_detection_rate, user_satisfaction, effectiveness_score, last_updated, created_at) VALUES ('29f3e3c3-8ce6-4d0e-8258-5cf584e04d81', 'Audio', 0, 0.5000, 45.00, 0.5000, 0.5000, 0.5000, '2026-06-09 09:23:18.02714+00', '2026-06-09 09:23:18.02714+00');
INSERT INTO public.captcha_type_effectiveness (id, challenge_type, usage_count, success_rate, avg_completion_time_secs, bot_detection_rate, user_satisfaction, effectiveness_score, last_updated, created_at) VALUES ('94a58f07-2a62-4737-a9cb-462fa7474be6', 'Logical', 0, 0.5000, 35.00, 0.5000, 0.5000, 0.5000, '2026-06-09 09:23:18.02714+00', '2026-06-09 09:23:18.02714+00');
INSERT INTO public.captcha_type_effectiveness (id, challenge_type, usage_count, success_rate, avg_completion_time_secs, bot_detection_rate, user_satisfaction, effectiveness_score, last_updated, created_at) VALUES ('a07f4f84-3d15-4a47-91e4-266a44324557', 'Behavioral', 0, 0.5000, 40.00, 0.6000, 0.5000, 0.5300, '2026-06-09 09:23:18.02714+00', '2026-06-09 09:23:18.02714+00');
INSERT INTO public.captcha_type_effectiveness (id, challenge_type, usage_count, success_rate, avg_completion_time_secs, bot_detection_rate, user_satisfaction, effectiveness_score, last_updated, created_at) VALUES ('8688dd7a-aaf4-40ad-a62e-14d08437484b', 'Hybrid', 0, 0.5000, 60.00, 0.7000, 0.4000, 0.5300, '2026-06-09 09:23:18.02714+00', '2026-06-09 09:23:18.02714+00');
INSERT INTO public.captcha_type_effectiveness (id, challenge_type, usage_count, success_rate, avg_completion_time_secs, bot_detection_rate, user_satisfaction, effectiveness_score, last_updated, created_at) VALUES ('9e32de94-8505-490f-9afb-2a3904f280e8', 'ImageJigsaw', 0, 0.5000, 45.00, 0.6000, 0.5000, 0.5300, '2026-06-09 09:23:18.02714+00', '2026-06-09 09:23:18.02714+00');
INSERT INTO public.captcha_type_effectiveness (id, challenge_type, usage_count, success_rate, avg_completion_time_secs, bot_detection_rate, user_satisfaction, effectiveness_score, last_updated, created_at) VALUES ('441aab4b-476d-4d78-9c21-ded518ff13da', 'ImageRotation', 0, 0.5000, 20.00, 0.4000, 0.6000, 0.4700, '2026-06-09 09:23:18.02714+00', '2026-06-09 09:23:18.02714+00');
INSERT INTO public.captcha_type_effectiveness (id, challenge_type, usage_count, success_rate, avg_completion_time_secs, bot_detection_rate, user_satisfaction, effectiveness_score, last_updated, created_at) VALUES ('b55fe3c7-7ad3-47a8-98b2-839edc0dae32', 'ImageObjectSelection', 0, 0.5000, 35.00, 0.6000, 0.5000, 0.5300, '2026-06-09 09:23:18.02714+00', '2026-06-09 09:23:18.02714+00');
INSERT INTO public.captcha_type_effectiveness (id, challenge_type, usage_count, success_rate, avg_completion_time_secs, bot_detection_rate, user_satisfaction, effectiveness_score, last_updated, created_at) VALUES ('8afaf826-489b-4d6d-aa67-e3bfbfdf10ff', 'ImageSequence', 0, 0.5000, 50.00, 0.7000, 0.5000, 0.5700, '2026-06-09 09:23:18.02714+00', '2026-06-09 09:23:18.02714+00');
INSERT INTO public.captcha_type_effectiveness (id, challenge_type, usage_count, success_rate, avg_completion_time_secs, bot_detection_rate, user_satisfaction, effectiveness_score, last_updated, created_at) VALUES ('f8b748f7-a20f-4a1d-9f83-a8337507711c', 'AudioToneSequence', 0, 0.5000, 40.00, 0.6000, 0.5000, 0.5300, '2026-06-09 09:23:18.02714+00', '2026-06-09 09:23:18.02714+00');
INSERT INTO public.captcha_type_effectiveness (id, challenge_type, usage_count, success_rate, avg_completion_time_secs, bot_detection_rate, user_satisfaction, effectiveness_score, last_updated, created_at) VALUES ('7d14d217-1ee6-4043-a789-e8292c039f80', 'AudioSpokenDigits', 0, 0.5000, 35.00, 0.5000, 0.6000, 0.5300, '2026-06-09 09:23:18.02714+00', '2026-06-09 09:23:18.02714+00');
INSERT INTO public.captcha_type_effectiveness (id, challenge_type, usage_count, success_rate, avg_completion_time_secs, bot_detection_rate, user_satisfaction, effectiveness_score, last_updated, created_at) VALUES ('5d904e97-386c-452f-8b82-cbba98f665da', 'AudioSpokenWords', 0, 0.5000, 45.00, 0.6000, 0.5000, 0.5300, '2026-06-09 09:23:18.02714+00', '2026-06-09 09:23:18.02714+00');
INSERT INTO public.captcha_type_effectiveness (id, challenge_type, usage_count, success_rate, avg_completion_time_secs, bot_detection_rate, user_satisfaction, effectiveness_score, last_updated, created_at) VALUES ('73aa6916-1254-4ccf-9afa-60ea9a213831', 'AudioPatternRecognition', 0, 0.5000, 50.00, 0.7000, 0.4000, 0.5300, '2026-06-09 09:23:18.02714+00', '2026-06-09 09:23:18.02714+00');
INSERT INTO public.captcha_type_effectiveness (id, challenge_type, usage_count, success_rate, avg_completion_time_secs, bot_detection_rate, user_satisfaction, effectiveness_score, last_updated, created_at) VALUES ('ceca53a4-ca6d-420c-baab-21dc0ba6e7dc', 'AudioSoundIdentification', 0, 0.5000, 40.00, 0.6000, 0.5000, 0.5300, '2026-06-09 09:23:18.02714+00', '2026-06-09 09:23:18.02714+00');


--
-- Data for Name: captcha_user_experience_metrics; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: captcha_user_history; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: captcha_user_type_performance; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: captcha_validation_attempts; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: client_scopes; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.client_scopes (id, realm_id, name, display_name, description, protocol, consent_required, display_on_consent_screen, consent_screen_text, include_in_token_scope, gui_order, icon_uri, attributes, enabled, created_at, updated_at) VALUES ('2add6a3a-043d-4e9d-b229-a0a956629e5d', '00000000-0000-0000-0000-000000000000', 'openid', 'OpenID Connect', 'Access to OpenID Connect authentication', 'openid-connect', false, false, NULL, true, 0, NULL, '{}', true, '2026-06-09 09:23:18.222588', '2026-06-09 09:23:18.222588');
INSERT INTO public.client_scopes (id, realm_id, name, display_name, description, protocol, consent_required, display_on_consent_screen, consent_screen_text, include_in_token_scope, gui_order, icon_uri, attributes, enabled, created_at, updated_at) VALUES ('62e87a80-9e28-4794-b614-9856cfd5bea5', '00000000-0000-0000-0000-000000000000', 'profile', 'User Profile', 'Access to user profile information (name, username, etc.)', 'openid-connect', true, true, NULL, true, 0, NULL, '{}', true, '2026-06-09 09:23:18.222588', '2026-06-09 09:23:18.222588');
INSERT INTO public.client_scopes (id, realm_id, name, display_name, description, protocol, consent_required, display_on_consent_screen, consent_screen_text, include_in_token_scope, gui_order, icon_uri, attributes, enabled, created_at, updated_at) VALUES ('08b34c3d-40e1-4ae3-bfb0-6496ff1e0de9', '00000000-0000-0000-0000-000000000000', 'email', 'Email Address', 'Access to user email address', 'openid-connect', true, true, NULL, true, 0, NULL, '{}', true, '2026-06-09 09:23:18.222588', '2026-06-09 09:23:18.222588');
INSERT INTO public.client_scopes (id, realm_id, name, display_name, description, protocol, consent_required, display_on_consent_screen, consent_screen_text, include_in_token_scope, gui_order, icon_uri, attributes, enabled, created_at, updated_at) VALUES ('d9006d77-149e-4d46-8d20-181f9862d847', '00000000-0000-0000-0000-000000000000', 'address', 'Physical Address', 'Access to user physical address', 'openid-connect', true, true, NULL, true, 0, NULL, '{}', true, '2026-06-09 09:23:18.222588', '2026-06-09 09:23:18.222588');
INSERT INTO public.client_scopes (id, realm_id, name, display_name, description, protocol, consent_required, display_on_consent_screen, consent_screen_text, include_in_token_scope, gui_order, icon_uri, attributes, enabled, created_at, updated_at) VALUES ('010a9e66-a710-4e05-9aa0-7d8cb3354fa0', '00000000-0000-0000-0000-000000000000', 'phone', 'Phone Number', 'Access to user phone number', 'openid-connect', true, true, NULL, true, 0, NULL, '{}', true, '2026-06-09 09:23:18.222588', '2026-06-09 09:23:18.222588');
INSERT INTO public.client_scopes (id, realm_id, name, display_name, description, protocol, consent_required, display_on_consent_screen, consent_screen_text, include_in_token_scope, gui_order, icon_uri, attributes, enabled, created_at, updated_at) VALUES ('dee3d5bb-809a-488b-9ec4-7affdf3dfcda', '00000000-0000-0000-0000-000000000000', 'offline_access', 'Offline Access', 'Access to refresh tokens for offline access', 'openid-connect', true, true, NULL, true, 0, NULL, '{}', true, '2026-06-09 09:23:18.222588', '2026-06-09 09:23:18.222588');
INSERT INTO public.client_scopes (id, realm_id, name, display_name, description, protocol, consent_required, display_on_consent_screen, consent_screen_text, include_in_token_scope, gui_order, icon_uri, attributes, enabled, created_at, updated_at) VALUES ('ffbfd853-2cac-4993-accf-2a49f3977378', '00000000-0000-0000-0000-000000000000', 'roles', 'User Roles', 'Access to user role information', 'openid-connect', false, false, NULL, true, 0, NULL, '{}', true, '2026-06-09 09:23:18.222588', '2026-06-09 09:23:18.222588');
INSERT INTO public.client_scopes (id, realm_id, name, display_name, description, protocol, consent_required, display_on_consent_screen, consent_screen_text, include_in_token_scope, gui_order, icon_uri, attributes, enabled, created_at, updated_at) VALUES ('dfba6aa1-c417-4a09-ab8e-d183ebdae84f', '00000000-0000-0000-0000-000000000000', 'groups', 'User Groups', 'Access to user group membership', 'openid-connect', false, false, NULL, true, 0, NULL, '{}', true, '2026-06-09 09:23:18.222588', '2026-06-09 09:23:18.222588');
INSERT INTO public.client_scopes (id, realm_id, name, display_name, description, protocol, consent_required, display_on_consent_screen, consent_screen_text, include_in_token_scope, gui_order, icon_uri, attributes, enabled, created_at, updated_at) VALUES ('3bf52107-b227-4847-be37-36df83cdfe46', '00000000-0000-0000-0000-000000000000', 'read:aset', 'Read Assets', 'View asset information', 'openid-connect', true, true, NULL, true, 0, NULL, '{}', true, '2026-06-09 09:23:18.222588', '2026-06-09 09:23:18.222588');
INSERT INTO public.client_scopes (id, realm_id, name, display_name, description, protocol, consent_required, display_on_consent_screen, consent_screen_text, include_in_token_scope, gui_order, icon_uri, attributes, enabled, created_at, updated_at) VALUES ('dacb94f1-209a-4302-86ea-a352564b2854', '00000000-0000-0000-0000-000000000000', 'write:aset', 'Manage Assets', 'Create, update, and delete assets', 'openid-connect', true, true, NULL, true, 0, NULL, '{}', true, '2026-06-09 09:23:18.222588', '2026-06-09 09:23:18.222588');
INSERT INTO public.client_scopes (id, realm_id, name, display_name, description, protocol, consent_required, display_on_consent_screen, consent_screen_text, include_in_token_scope, gui_order, icon_uri, attributes, enabled, created_at, updated_at) VALUES ('d05b0550-1782-45c0-8783-c023c2a52610', '00000000-0000-0000-0000-000000000000', 'read:laporan', 'Read Reports', 'View reports and analytics', 'openid-connect', true, true, NULL, true, 0, NULL, '{}', true, '2026-06-09 09:23:18.222588', '2026-06-09 09:23:18.222588');
INSERT INTO public.client_scopes (id, realm_id, name, display_name, description, protocol, consent_required, display_on_consent_screen, consent_screen_text, include_in_token_scope, gui_order, icon_uri, attributes, enabled, created_at, updated_at) VALUES ('7a439cb9-88aa-4b48-b575-3a51d22c7a7a', '00000000-0000-0000-0000-000000000000', 'write:laporan', 'Manage Reports', 'Create and manage reports', 'openid-connect', true, true, NULL, true, 0, NULL, '{}', true, '2026-06-09 09:23:18.222588', '2026-06-09 09:23:18.222588');
INSERT INTO public.client_scopes (id, realm_id, name, display_name, description, protocol, consent_required, display_on_consent_screen, consent_screen_text, include_in_token_scope, gui_order, icon_uri, attributes, enabled, created_at, updated_at) VALUES ('3987c49a-40bb-49a7-a448-f575c6cef40a', '00000000-0000-0000-0000-000000000000', 'admin:satker', 'Satker Administration', 'Administrative access to satker (work unit)', 'openid-connect', true, true, NULL, true, 0, NULL, '{}', true, '2026-06-09 09:23:18.222588', '2026-06-09 09:23:18.222588');
INSERT INTO public.client_scopes (id, realm_id, name, display_name, description, protocol, consent_required, display_on_consent_screen, consent_screen_text, include_in_token_scope, gui_order, icon_uri, attributes, enabled, created_at, updated_at) VALUES ('083c3ae7-0507-4cb9-aa5e-adab2691a6d3', '00000000-0000-0000-0000-000000000000', 'admin:wilayah', 'Regional Administration', 'Administrative access to regional level', 'openid-connect', true, true, NULL, true, 0, NULL, '{}', true, '2026-06-09 09:23:18.222588', '2026-06-09 09:23:18.222588');
INSERT INTO public.client_scopes (id, realm_id, name, display_name, description, protocol, consent_required, display_on_consent_screen, consent_screen_text, include_in_token_scope, gui_order, icon_uri, attributes, enabled, created_at, updated_at) VALUES ('58795a60-da39-4285-8f0e-028538beea6e', '00000000-0000-0000-0000-000000000000', 'admin:pusat', 'Central Administration', 'Administrative access to central level', 'openid-connect', true, true, NULL, true, 0, NULL, '{}', true, '2026-06-09 09:23:18.222588', '2026-06-09 09:23:18.222588');


--
-- Data for Name: client_default_scopes; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: client_optional_scopes; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: client_policies; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.client_policies (id, realm_id, name, description, enabled, conditions, condition_config, executors, executor_config, priority, policy_type, created_at, updated_at, created_by) VALUES ('d2d679b9-5327-4cf0-8e5b-2269d7f0ce21', '00000000-0000-0000-0000-000000000000', 'PKCE Enforcement', 'Enforces Proof Key for Code Exchange (PKCE) for authorization code flow', true, '{grant-type-condition}', '{"grant-type-condition": {"allowed_grant_types": ["authorization_code"]}}', '{pkce-enforcer-executor}', '{"pkce-enforcer-executor": {"enforce_pkce": true}}', 100, 'security', '2026-06-09 09:23:18.293862+00', '2026-06-09 09:23:18.293862+00', NULL);
INSERT INTO public.client_policies (id, realm_id, name, description, enabled, conditions, condition_config, executors, executor_config, priority, policy_type, created_at, updated_at, created_by) VALUES ('b6cdcd10-2fa5-48aa-b39d-100f7cdd68de', '00000000-0000-0000-0000-000000000000', 'Secure Redirect URIs', 'Enforces HTTPS for all redirect URIs', true, '{any-client-condition}', '{}', '{secure-redirect-uris-enforcer-executor}', '{"secure-redirect-uris-enforcer-executor": {"enforce_https": true}}', 90, 'security', '2026-06-09 09:23:18.293862+00', '2026-06-09 09:23:18.293862+00', NULL);
INSERT INTO public.client_policies (id, realm_id, name, description, enabled, conditions, condition_config, executors, executor_config, priority, policy_type, created_at, updated_at, created_by) VALUES ('fe0c99b5-8644-4262-b934-3907acf2e73e', '00000000-0000-0000-0000-000000000000', 'Reject Implicit Grant', 'Rejects insecure implicit grant flow', true, '{any-client-condition}', '{}', '{reject-implicit-grant-executor}', '{}', 80, 'security', '2026-06-09 09:23:18.293862+00', '2026-06-09 09:23:18.293862+00', NULL);
INSERT INTO public.client_policies (id, realm_id, name, description, enabled, conditions, condition_config, executors, executor_config, priority, policy_type, created_at, updated_at, created_by) VALUES ('31a82766-221e-4984-a49a-8f12e0c08a96', '00000000-0000-0000-0000-000000000000', 'DPoP Binding', 'Enforces Demonstrating Proof-of-Possession (DPoP) for token binding', false, '{any-client-condition}', '{}', '{dpop-bind-enforcer-executor}', '{"dpop-bind-enforcer-executor": {"enforce_dpop": true}}', 110, 'security', '2026-06-09 09:23:18.293862+00', '2026-06-09 09:23:18.293862+00', NULL);
INSERT INTO public.client_policies (id, realm_id, name, description, enabled, conditions, condition_config, executors, executor_config, priority, policy_type, created_at, updated_at, created_by) VALUES ('f7024b76-aba7-4dc1-92f7-71ab2f1f8ce5', '00000000-0000-0000-0000-000000000000', 'Confidential Clients Only', 'Only allows confidential clients to authenticate', false, '{any-client-condition}', '{}', '{confidential-client-accept-executor}', '{"confidential-client-accept-executor": {"accept_confidential_only": true}}', 70, 'security', '2026-06-09 09:23:18.293862+00', '2026-06-09 09:23:18.293862+00', NULL);
INSERT INTO public.client_policies (id, realm_id, name, description, enabled, conditions, condition_config, executors, executor_config, priority, policy_type, created_at, updated_at, created_by) VALUES ('c2c061c1-844b-4e1e-83f9-a0ca822d0666', '00000000-0000-0000-0000-000000000000', 'Consent Required', 'Requires explicit user consent for all authorizations', false, '{any-client-condition}', '{}', '{consent-required-executor}', '{"consent-required-executor": {"require_consent": true}}', 60, 'compliance', '2026-06-09 09:23:18.293862+00', '2026-06-09 09:23:18.293862+00', NULL);
INSERT INTO public.client_policies (id, realm_id, name, description, enabled, conditions, condition_config, executors, executor_config, priority, policy_type, created_at, updated_at, created_by) VALUES ('920f1c62-35c7-4679-8168-14641e48982f', '00000000-0000-0000-0000-000000000000', 'Secure Signing Algorithm', 'Enforces secure signing algorithms (Ed25519, ES256, RS256)', false, '{any-client-condition}', '{}', '{secure-signing-algorithm-executor}', '{"secure-signing-algorithm-executor": {"allowed_algorithms": ["EdDSA", "ES256", "RS256"], "enforce_secure_algorithm": true}}', 85, 'security', '2026-06-09 09:23:18.293862+00', '2026-06-09 09:23:18.293862+00', NULL);


--
-- Data for Name: client_profiles; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.client_profiles (id, realm_id, name, description, enabled, policy_ids, profile_type, is_builtin, created_at, updated_at, created_by) VALUES ('9ddc1449-ce8f-431e-bd0b-b994e85e3035', '00000000-0000-0000-0000-000000000000', 'FAPI 1.0 Baseline', 'Financial-grade API 1.0 Baseline Security Profile - enforces PKCE, secure redirects, and rejects implicit grants', true, '{d2d679b9-5327-4cf0-8e5b-2269d7f0ce21,b6cdcd10-2fa5-48aa-b39d-100f7cdd68de,fe0c99b5-8644-4262-b934-3907acf2e73e}', 'fapi-1-baseline', true, '2026-06-09 09:23:18.293862+00', '2026-06-09 09:23:18.293862+00', NULL);
INSERT INTO public.client_profiles (id, realm_id, name, description, enabled, policy_ids, profile_type, is_builtin, created_at, updated_at, created_by) VALUES ('81b82a9d-ad3e-425f-8f72-85e261325054', '00000000-0000-0000-0000-000000000000', 'FAPI 1.0 Advanced', 'Financial-grade API 1.0 Advanced Security Profile - adds DPoP binding and enhanced security measures', true, '{d2d679b9-5327-4cf0-8e5b-2269d7f0ce21,b6cdcd10-2fa5-48aa-b39d-100f7cdd68de,fe0c99b5-8644-4262-b934-3907acf2e73e,31a82766-221e-4984-a49a-8f12e0c08a96}', 'fapi-1-advanced', true, '2026-06-09 09:23:18.293862+00', '2026-06-09 09:23:18.293862+00', NULL);


--
-- Data for Name: client_policy_assignments; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: client_registration_tokens; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: initial_access_tokens; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: client_registration_audit_log; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: client_registration_policies; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: protocol_mappers; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: client_scope_mappings; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: credential_types; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.credential_types (id, code, name, description, is_system, is_primary, requires_verification, config_schema, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('2363e52f-a9a4-4a5f-81a0-497a5ca2ddc5', 'password', 'Password', 'Password-based authentication', true, true, false, '{}', '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.credential_types (id, code, name, description, is_system, is_primary, requires_verification, config_schema, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('cecc448a-95ca-4010-b34e-20782b697a98', 'totp', 'TOTP', 'Time-based One-Time Password', true, false, true, '{}', '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.credential_types (id, code, name, description, is_system, is_primary, requires_verification, config_schema, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('6b231aa5-3af0-4f33-953a-1b90677a701a', 'webauthn', 'WebAuthn', 'WebAuthn/FIDO2 authentication', true, false, true, '{}', '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.credential_types (id, code, name, description, is_system, is_primary, requires_verification, config_schema, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('5f93f5b6-92af-4114-8b30-b52b7fa29e0c', 'recovery_code', 'Recovery Code', 'Backup recovery codes', true, false, false, '{}', '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.credential_types (id, code, name, description, is_system, is_primary, requires_verification, config_schema, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('bfe67439-1fb6-47a8-b6e4-54892f4eb154', 'magic_link', 'Magic Link', 'Email-based passwordless login', true, false, true, '{}', '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);


--
-- Data for Name: custom_themes; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: device_sessions; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: device_trust_history; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: event_listeners; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: event_log; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: event_listener_executions; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: event_webhooks; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: events; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: federated_auth_log; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: identity_providers; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: federated_identities; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: federated_identity_links; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: groups; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: group_attributes; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: roles; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.roles (id, name, description, realm_id, composite, client_role, created_at, updated_at, deleted_at) VALUES ('00000000-0000-0000-0000-000000000002', 'admin', 'Administrator role with full access', '00000000-0000-0000-0000-000000000000', false, false, '2026-06-09 09:23:17.185556+00', '2026-06-09 09:23:17.185556+00', NULL);
INSERT INTO public.roles (id, name, description, realm_id, composite, client_role, created_at, updated_at, deleted_at) VALUES ('f646c0e4-5cb4-4008-8e3b-118af6c22ede', 'operator_satker', 'Operator Satker Perlengkapan - Input data kebutuhan BMN, pemakaian, pemeliharaan, pakaian dinas di tingkat satuan kerja', '00000000-0000-0000-0000-000000000000', false, true, '2026-06-09 09:23:18.568707+00', '2026-06-09 09:23:18.568707+00', NULL);
INSERT INTO public.roles (id, name, description, realm_id, composite, client_role, created_at, updated_at, deleted_at) VALUES ('54b5d1ae-8aaa-4e31-b4b2-a77a49cfd06b', 'validator_wilayah', 'Validator Wilayah Perlengkapan - Verifikasi dan validasi data dari operator satker di tingkat wilayah/Kejaksaan Tinggi', '00000000-0000-0000-0000-000000000000', false, true, '2026-06-09 09:23:18.568707+00', '2026-06-09 09:23:18.568707+00', NULL);
INSERT INTO public.roles (id, name, description, realm_id, composite, client_role, created_at, updated_at, deleted_at) VALUES ('9377d14c-22b6-4674-b080-a6ba43969eb1', 'validator_pusat', 'Validator Pusat Perlengkapan - Persetujuan akhir di tingkat Kejaksaan Agung/pusat, termasuk penerbitan SK', '00000000-0000-0000-0000-000000000000', false, true, '2026-06-09 09:23:18.568707+00', '2026-06-09 09:23:18.568707+00', NULL);


--
-- Data for Name: group_roles; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: identity_broker_configs; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: identity_provider_mappers; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: key_rotation_audit; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: mfa_admin_actions; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: mfa_devices; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: mfa_policies; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.mfa_policies (id, policy_data, created_by, active, created_at, updated_at) VALUES ('434d6dbb-7e02-4401-9521-684d00c1943a', '{"enforce_for_all": true, "enforce_for_roles": ["admin", "security_admin"], "grace_period_days": 30, "enforce_for_satkers": [], "max_failed_attempts": 5, "backup_codes_required": true, "lockout_duration_minutes": 30}', NULL, true, '2026-06-09 09:23:17.922578+00', '2026-06-09 09:23:17.922578+00');


--
-- Data for Name: oauth2_access_tokens; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: oauth2_authorization_codes; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: oauth2_provider_configs; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: oauth2_states; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: oauth2_token_exchanges; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: offline_tokens; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: organizations; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: organization_domains; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: organization_identity_providers; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: organization_invitations; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: role_types; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.role_types (id, code, name, description, category, is_system, is_assignable, priority, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('45a9b6ae-4571-45a0-9c57-c9485f5f66a4', 'owner', 'Owner', 'Full ownership rights', 'ownership', true, true, 100, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.role_types (id, code, name, description, category, is_system, is_assignable, priority, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('7ef8623d-3e82-4af9-9790-e835f6d7ef82', 'admin', 'Administrator', 'Administrative privileges', 'administrative', true, true, 90, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.role_types (id, code, name, description, category, is_system, is_assignable, priority, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('9eaf59e5-5f33-4c88-a924-2f9ee4adb124', 'manager', 'Manager', 'Management privileges', 'management', true, true, 80, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.role_types (id, code, name, description, category, is_system, is_assignable, priority, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('bf3f213b-523c-4207-87c6-9f3a0dddc56f', 'member', 'Member', 'Standard member access', 'membership', true, true, 50, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.role_types (id, code, name, description, category, is_system, is_assignable, priority, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('f0c09220-30cb-45fa-935c-bb208110067c', 'viewer', 'Viewer', 'Read-only access', 'access', true, true, 30, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.role_types (id, code, name, description, category, is_system, is_assignable, priority, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('d911dd38-58c4-4f64-8375-3562ae1685a8', 'guest', 'Guest', 'Limited guest access', 'access', true, true, 10, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);


--
-- Data for Name: organization_members; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: password_history; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: resource_servers; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: resources; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: scopes; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: permission_tickets; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: realm_theme_settings; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: refresh_token_history; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: role_capabilities; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: role_hierarchy; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: role_permissions; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('49995531-43fc-472f-ae9e-6e976d4e16f7', 'f646c0e4-5cb4-4008-8e3b-118af6c22ede', 'manage', 'kebutuhan_bmn', '{create,read,update,delete,submit}', '{"scope": "own_satker"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('d598ab25-bdd5-4f8c-a906-9894eee43125', 'f646c0e4-5cb4-4008-8e3b-118af6c22ede', 'manage', 'pemakaian_bmn', '{create,read,update,delete,submit}', '{"scope": "own_satker"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('63159841-1792-4fc2-8e29-10ab275af4c9', 'f646c0e4-5cb4-4008-8e3b-118af6c22ede', 'manage', 'penghapusan_bmn', '{create,read,update,submit}', '{"scope": "own_satker"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('fea14242-43d8-4230-acf0-1d055b524a4d', 'f646c0e4-5cb4-4008-8e3b-118af6c22ede', 'manage', 'pemeliharaan', '{create,read,update,delete}', '{"scope": "own_satker"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('0108928e-3098-4ce0-aa28-4cd2ef547155', 'f646c0e4-5cb4-4008-8e3b-118af6c22ede', 'manage', 'pakaian_dinas', '{create,read,update}', '{"scope": "own_satker"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('766cebe4-04c8-4020-8e70-a25da7b4c21e', 'f646c0e4-5cb4-4008-8e3b-118af6c22ede', 'view', 'bank_aset', '{read}', '{"scope": "own_satker"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('16a9bb78-d14c-4dd4-b8f5-9b7100ad1168', 'f646c0e4-5cb4-4008-8e3b-118af6c22ede', 'view', 'dashboard', '{read}', '{"scope": "own_satker"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('912ddafa-d35d-45d0-990e-d133a6d40074', 'f646c0e4-5cb4-4008-8e3b-118af6c22ede', 'manage', 'roadmap_sarpras', '{create,read,update}', '{"scope": "own_satker"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('b83704ff-9b89-4308-9575-a4d9d9c9d3d1', 'f646c0e4-5cb4-4008-8e3b-118af6c22ede', 'view', 'laporan', '{read,export}', '{"scope": "own_satker"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('f8894843-5822-48f0-b9eb-cd526d3aa072', '54b5d1ae-8aaa-4e31-b4b2-a77a49cfd06b', 'validate', 'kebutuhan_bmn', '{read,validate,return,forward}', '{"scope": "wilayah"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('89dd203f-e4a4-45a4-af02-b924b6954326', '54b5d1ae-8aaa-4e31-b4b2-a77a49cfd06b', 'validate', 'pemakaian_bmn', '{read,approve,reject,return}', '{"scope": "wilayah"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('0131a1b5-e9e8-4bb3-9130-b8a26f39453b', '54b5d1ae-8aaa-4e31-b4b2-a77a49cfd06b', 'validate', 'penghapusan_bmn', '{read,validate,return,forward}', '{"scope": "wilayah"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('44c79d5f-9347-49f6-af1e-93420b5ea24e', '54b5d1ae-8aaa-4e31-b4b2-a77a49cfd06b', 'view', 'pemeliharaan', '{read}', '{"scope": "wilayah"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('2ea0613e-f3f9-47ae-a264-37b9d3c3a659', '54b5d1ae-8aaa-4e31-b4b2-a77a49cfd06b', 'view', 'pakaian_dinas', '{read,validate}', '{"scope": "wilayah"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('facc0eb4-056d-4022-a9dc-b6a9fad07a2f', '54b5d1ae-8aaa-4e31-b4b2-a77a49cfd06b', 'view', 'bank_aset', '{read}', '{"scope": "wilayah"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('e5507ae1-9da8-4003-b68c-97b3b66b64a4', '54b5d1ae-8aaa-4e31-b4b2-a77a49cfd06b', 'view', 'dashboard', '{read}', '{"scope": "wilayah"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('05662540-f62f-47e1-9edd-9340fce53e46', '54b5d1ae-8aaa-4e31-b4b2-a77a49cfd06b', 'view', 'roadmap_sarpras', '{read}', '{"scope": "wilayah"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('c65dc764-7374-4bea-afbe-82567d7c70d7', '54b5d1ae-8aaa-4e31-b4b2-a77a49cfd06b', 'view', 'laporan', '{read,export}', '{"scope": "wilayah"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('9a3749b3-626b-4b3b-921f-6273d412dccb', '9377d14c-22b6-4674-b080-a6ba43969eb1', 'approve', 'kebutuhan_bmn', '{read,approve,reject,return}', '{"scope": "pusat"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('1aa40c66-257d-48dc-907a-6481658b0c28', '9377d14c-22b6-4674-b080-a6ba43969eb1', 'approve', 'pemakaian_bmn', '{read,approve,reject}', '{"scope": "pusat"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('d41a5d44-0216-480f-80c6-20d2de9f1878', '9377d14c-22b6-4674-b080-a6ba43969eb1', 'approve', 'penghapusan_bmn', '{read,approve,reject,generate_sk,sign_sk}', '{"scope": "pusat"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('9fac3e64-ca74-4dfa-ac59-2a8094ea5ef4', '9377d14c-22b6-4674-b080-a6ba43969eb1', 'view', 'pemeliharaan', '{read}', '{"scope": "all"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('83548f26-e611-433d-b0b3-a00d81841658', '9377d14c-22b6-4674-b080-a6ba43969eb1', 'approve', 'pakaian_dinas', '{read,approve,reject}', '{"scope": "pusat"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('7c34e1c2-3b31-41ff-8b6b-28ca0e83803e', '9377d14c-22b6-4674-b080-a6ba43969eb1', 'view', 'bank_aset', '{read}', '{"scope": "all"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('9a2aa1c4-520f-4238-a573-b75c68628a96', '9377d14c-22b6-4674-b080-a6ba43969eb1', 'view', 'dashboard', '{read}', '{"scope": "all"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('d2911766-a9c3-4259-80a6-b30a3ddf0e3a', '9377d14c-22b6-4674-b080-a6ba43969eb1', 'manage', 'roadmap_sarpras', '{read,approve}', '{"scope": "pusat"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('c81a011d-1945-4d3d-a539-c9f9272d62f6', '9377d14c-22b6-4674-b080-a6ba43969eb1', 'manage', 'mapping_kodefikasi', '{read,approve,reject}', '{"scope": "pusat"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('f4273a34-67d7-4b63-ab81-18920e464bb3', '9377d14c-22b6-4674-b080-a6ba43969eb1', 'view', 'laporan', '{read,export}', '{"scope": "all"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('574ff594-4d39-47b7-827f-8b5a69290c84', '00000000-0000-0000-0000-000000000002', 'admin', 'users', '{create,read,update,delete,assign_role}', '{"scope": "all"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('c9c5f45c-fe2a-4ec0-895e-59a67ed21857', '00000000-0000-0000-0000-000000000002', 'admin', 'roles', '{create,read,update,delete}', '{"scope": "all"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('5f7f44da-f676-4363-81c1-10c66a5bb82d', '00000000-0000-0000-0000-000000000002', 'admin', 'kebutuhan_bmn', '{create,read,update,delete,approve,reject}', '{"scope": "all"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('015e0db6-8392-44f2-87ac-71b3306a9cd3', '00000000-0000-0000-0000-000000000002', 'admin', 'pemakaian_bmn', '{create,read,update,delete,approve,reject}', '{"scope": "all"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('627f281f-688f-4933-9f8a-81d7141f95c2', '00000000-0000-0000-0000-000000000002', 'admin', 'penghapusan_bmn', '{create,read,update,delete,approve,reject,generate_sk,sign_sk}', '{"scope": "all"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('6c971994-c5b5-441d-825c-475eb1a37890', '00000000-0000-0000-0000-000000000002', 'admin', 'pemeliharaan', '{create,read,update,delete}', '{"scope": "all"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('24573bbc-c88b-451d-8626-d55861237176', '00000000-0000-0000-0000-000000000002', 'admin', 'pakaian_dinas', '{create,read,update,delete,approve}', '{"scope": "all"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('a8312b9b-d61f-4859-9c40-6f6db4f18561', '00000000-0000-0000-0000-000000000002', 'admin', 'bank_aset', '{create,read,update,delete}', '{"scope": "all"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('3180b2fa-86f0-41fa-a8c2-3c62392767ad', '00000000-0000-0000-0000-000000000002', 'admin', 'dashboard', '{read,configure}', '{"scope": "all"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('112e250e-0227-4269-ba81-516717cce884', '00000000-0000-0000-0000-000000000002', 'admin', 'roadmap_sarpras', '{create,read,update,delete,approve}', '{"scope": "all"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('1ed3c8af-c1f4-460d-9f52-54fc69e30914', '00000000-0000-0000-0000-000000000002', 'admin', 'mapping_kodefikasi', '{create,read,update,delete,approve}', '{"scope": "all"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('1ad8f4a4-ee2e-4257-bffe-dc97babdfd4d', '00000000-0000-0000-0000-000000000002', 'admin', 'laporan', '{read,export,generate}', '{"scope": "all"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('f963e411-a887-4d9c-80cf-211cf924855c', '00000000-0000-0000-0000-000000000002', 'admin', 'konfigurasi', '{read,update}', '{"scope": "all"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('12d5143f-5f61-4392-abd5-8cafabdc8def', '00000000-0000-0000-0000-000000000002', 'admin', 'audit_log', '{read}', '{"scope": "all"}', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.role_permissions (id, role_id, permission, resource, actions, conditions, created_at) VALUES ('ba5dc297-adde-4e10-9854-0ecfe6d227aa', '00000000-0000-0000-0000-000000000002', 'admin', 'master_data', '{create,read,update,delete}', '{"scope": "all"}', '2026-06-09 09:23:18.568707+00');


--
-- Data for Name: role_policies; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: saml_assertion_cache; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: saml_identity_providers; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: saml_messages; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: saml_service_providers; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: saml_sessions; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: satkers; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.satkers (id, code, name, description, parent_code, level, satker_type, active, attributes, created_at, updated_at) VALUES ('8491c150-7f80-4d71-8063-db0449447ca9', 'PUSAT', 'Kejaksaan Agung Republik Indonesia', 'Kantor pusat Kejaksaan Agung RI', NULL, 0, '"Pusat"', true, NULL, '2026-06-09 09:23:18.138412+00', '2026-06-09 09:23:18.138412+00');
INSERT INTO public.satkers (id, code, name, description, parent_code, level, satker_type, active, attributes, created_at, updated_at) VALUES ('16bd7c04-6f18-4cc4-9576-475df661750d', 'KT-DKI', 'Kejaksaan Tinggi DKI Jakarta', 'Kejaksaan Tinggi wilayah DKI Jakarta', 'PUSAT', 1, '"KejaksaanTinggi"', true, NULL, '2026-06-09 09:23:18.138412+00', '2026-06-09 09:23:18.138412+00');
INSERT INTO public.satkers (id, code, name, description, parent_code, level, satker_type, active, attributes, created_at, updated_at) VALUES ('c6ef7f34-fbd5-41a4-a375-42fd47423d95', 'KT-JABAR', 'Kejaksaan Tinggi Jawa Barat', 'Kejaksaan Tinggi wilayah Jawa Barat', 'PUSAT', 1, '"KejaksaanTinggi"', true, NULL, '2026-06-09 09:23:18.138412+00', '2026-06-09 09:23:18.138412+00');
INSERT INTO public.satkers (id, code, name, description, parent_code, level, satker_type, active, attributes, created_at, updated_at) VALUES ('482e5d8b-8dab-4e9e-9296-97f684859e35', 'KT-JATENG', 'Kejaksaan Tinggi Jawa Tengah', 'Kejaksaan Tinggi wilayah Jawa Tengah', 'PUSAT', 1, '"KejaksaanTinggi"', true, NULL, '2026-06-09 09:23:18.138412+00', '2026-06-09 09:23:18.138412+00');
INSERT INTO public.satkers (id, code, name, description, parent_code, level, satker_type, active, attributes, created_at, updated_at) VALUES ('a1047f74-8d6d-4b5a-9925-1d2e7e785caa', 'KT-JATIM', 'Kejaksaan Tinggi Jawa Timur', 'Kejaksaan Tinggi wilayah Jawa Timur', 'PUSAT', 1, '"KejaksaanTinggi"', true, NULL, '2026-06-09 09:23:18.138412+00', '2026-06-09 09:23:18.138412+00');
INSERT INTO public.satkers (id, code, name, description, parent_code, level, satker_type, active, attributes, created_at, updated_at) VALUES ('009f920c-9e0e-4aad-adcc-b136e575b9ff', 'KN-JAKPUS', 'Kejaksaan Negeri Jakarta Pusat', 'Kejaksaan Negeri Jakarta Pusat', 'KT-DKI', 2, '"KejaksaanNegeri"', true, NULL, '2026-06-09 09:23:18.138412+00', '2026-06-09 09:23:18.138412+00');
INSERT INTO public.satkers (id, code, name, description, parent_code, level, satker_type, active, attributes, created_at, updated_at) VALUES ('9307e287-667a-44df-a18f-915843b4daed', 'KN-JAKSEL', 'Kejaksaan Negeri Jakarta Selatan', 'Kejaksaan Negeri Jakarta Selatan', 'KT-DKI', 2, '"KejaksaanNegeri"', true, NULL, '2026-06-09 09:23:18.138412+00', '2026-06-09 09:23:18.138412+00');
INSERT INTO public.satkers (id, code, name, description, parent_code, level, satker_type, active, attributes, created_at, updated_at) VALUES ('645f226d-39c6-4ec2-8b71-08564ce38a15', 'KN-BANDUNG', 'Kejaksaan Negeri Bandung', 'Kejaksaan Negeri Bandung', 'KT-JABAR', 2, '"KejaksaanNegeri"', true, NULL, '2026-06-09 09:23:18.138412+00', '2026-06-09 09:23:18.138412+00');
INSERT INTO public.satkers (id, code, name, description, parent_code, level, satker_type, active, attributes, created_at, updated_at) VALUES ('bb8c5fd6-0356-4d2d-8bb7-c41032023cf2', 'KN-SEMARANG', 'Kejaksaan Negeri Semarang', 'Kejaksaan Negeri Semarang', 'KT-JATENG', 2, '"KejaksaanNegeri"', true, NULL, '2026-06-09 09:23:18.138412+00', '2026-06-09 09:23:18.138412+00');
INSERT INTO public.satkers (id, code, name, description, parent_code, level, satker_type, active, attributes, created_at, updated_at) VALUES ('dbebc7b1-1ffd-437a-9e58-67eec6b42476', 'KN-SURABAYA', 'Kejaksaan Negeri Surabaya', 'Kejaksaan Negeri Surabaya', 'KT-JATIM', 2, '"KejaksaanNegeri"', true, NULL, '2026-06-09 09:23:18.138412+00', '2026-06-09 09:23:18.138412+00');


--
-- Data for Name: satker_admin_roles; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: satker_audit_logs; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: satker_permissions; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: satker_types; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.satker_types (id, code, name, description, hierarchy_level, parent_type_id, code_pattern, is_system, metadata, created_at, updated_at) VALUES ('978d3024-5a22-4829-8b7e-56a85e2a9967', 'pusat', 'Pusat', 'Central office', 100, NULL, '^KP_.*', true, '{}', '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00');
INSERT INTO public.satker_types (id, code, name, description, hierarchy_level, parent_type_id, code_pattern, is_system, metadata, created_at, updated_at) VALUES ('a44bfac2-944d-491b-9c71-ac8606cab372', 'kejati', 'Kejaksaan Tinggi', 'High Prosecutor Office', 80, NULL, '^KT_.*', true, '{}', '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00');
INSERT INTO public.satker_types (id, code, name, description, hierarchy_level, parent_type_id, code_pattern, is_system, metadata, created_at, updated_at) VALUES ('4c8a7120-dd26-401f-a18a-423c8c3d5756', 'kejari', 'Kejaksaan Negeri', 'District Prosecutor Office', 60, NULL, '^KN_.*', true, '{}', '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00');
INSERT INTO public.satker_types (id, code, name, description, hierarchy_level, parent_type_id, code_pattern, is_system, metadata, created_at, updated_at) VALUES ('175047ec-0fed-40c6-b74d-74e1fad09fa2', 'cabang', 'Cabang', 'Branch office', 40, NULL, '^CB_.*', true, '{}', '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00');
INSERT INTO public.satker_types (id, code, name, description, hierarchy_level, parent_type_id, code_pattern, is_system, metadata, created_at, updated_at) VALUES ('b0faa1c7-2a56-40a3-9bae-b29f1092e356', 'unit_khusus', 'Unit Khusus', 'Special unit', 20, NULL, '^UK_.*', true, '{}', '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00');


--
-- Data for Name: scope_types; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.scope_types (id, code, name, description, hierarchy_level, parent_scope_type_id, scope_pattern, is_system, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('afcd88d1-7470-4bec-ae17-4408b3b8b235', 'global', 'Global', 'Global scope - applies everywhere', 100, NULL, NULL, true, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.scope_types (id, code, name, description, hierarchy_level, parent_scope_type_id, scope_pattern, is_system, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('697043a3-aeba-49f4-a8c4-02dd4794f8a8', 'realm', 'Realm', 'Realm-level scope', 90, NULL, NULL, true, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.scope_types (id, code, name, description, hierarchy_level, parent_scope_type_id, scope_pattern, is_system, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('d533e21f-d2ef-4cdc-a59f-04dd6d78be02', 'organization', 'Organization', 'Organization-level scope', 80, NULL, NULL, true, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.scope_types (id, code, name, description, hierarchy_level, parent_scope_type_id, scope_pattern, is_system, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('269440d0-01b4-4660-86cd-50d63bb0db14', 'group', 'Group', 'Group-level scope', 70, NULL, NULL, true, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);
INSERT INTO public.scope_types (id, code, name, description, hierarchy_level, parent_scope_type_id, scope_pattern, is_system, metadata, realm_id, created_at, updated_at, deleted_at) VALUES ('2ab55605-a3fa-43a8-b2f0-b16a1031a8ec', 'user', 'User', 'User-level scope', 10, NULL, NULL, true, '{}', NULL, '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00', NULL);


--
-- Data for Name: service_accounts; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: service_account_audit_log; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: service_account_roles; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: sessions; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: social_accounts; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: social_login_configs; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: software_statement_issuers; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: theme_inheritance; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: theme_resources; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: theme_templates; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: theme_types; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.theme_types (id, code, name, description, is_system, default_template, metadata, created_at, updated_at) VALUES ('4cad85fc-8a60-44ec-98fb-9dc206040282', 'login', 'Login', 'Login page theme', true, NULL, '{}', '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00');
INSERT INTO public.theme_types (id, code, name, description, is_system, default_template, metadata, created_at, updated_at) VALUES ('9e782e68-94c6-4b39-bafc-a136d53a2af7', 'account', 'Account', 'Account management theme', true, NULL, '{}', '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00');
INSERT INTO public.theme_types (id, code, name, description, is_system, default_template, metadata, created_at, updated_at) VALUES ('2ffacafa-631e-4a93-8c74-7b8db26c959a', 'admin', 'Admin', 'Admin console theme', true, NULL, '{}', '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00');
INSERT INTO public.theme_types (id, code, name, description, is_system, default_template, metadata, created_at, updated_at) VALUES ('711b241a-3cff-4c85-9c3f-d2ff05e9d6c5', 'email', 'Email', 'Email template theme', true, NULL, '{}', '2026-06-09 09:23:18.464589+00', '2026-06-09 09:23:18.464589+00');


--
-- Data for Name: token_exchange_audit; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: uma_permission_requests; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: uma_policies; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: user_attributes; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.user_attributes (id, user_id, name, value, created_at) VALUES ('d67c73c2-63dd-4ced-8933-30f4d5ccb175', '274c04e6-a811-4fe8-a940-13f0f1098a97', 'nip', '199203142014031001', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.user_attributes (id, user_id, name, value, created_at) VALUES ('3f62c475-afc4-4810-8419-eff2031d6885', '274c04e6-a811-4fe8-a940-13f0f1098a97', 'satker_code', '0100000', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.user_attributes (id, user_id, name, value, created_at) VALUES ('be9daf4c-a4db-471d-b000-bacbcfa4ad42', '274c04e6-a811-4fe8-a940-13f0f1098a97', 'satker_name', 'Kejaksaan Agung Republik Indonesia', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.user_attributes (id, user_id, name, value, created_at) VALUES ('ff65566a-6b55-426d-b83a-2d5828bba6a3', '274c04e6-a811-4fe8-a940-13f0f1098a97', 'jabatan', 'Kasubag Perlengkapan', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.user_attributes (id, user_id, name, value, created_at) VALUES ('2bbf5803-ae52-48f6-a0eb-ac423b69e046', '274c04e6-a811-4fe8-a940-13f0f1098a97', 'golongan', 'III/c', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.user_attributes (id, user_id, name, value, created_at) VALUES ('5147630f-94d2-483b-b823-7b0442f5a654', '274c04e6-a811-4fe8-a940-13f0f1098a97', 'eselon', '4', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.user_attributes (id, user_id, name, value, created_at) VALUES ('d9721f65-bede-44bf-a17c-938fa5e495e2', '274c04e6-a811-4fe8-a940-13f0f1098a97', 'pangkat', 'Penata', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.user_attributes (id, user_id, name, value, created_at) VALUES ('2baf6e26-0fc4-4b8e-9454-480ee6314492', '274c04e6-a811-4fe8-a940-13f0f1098a97', 'unit_kerja', 'Biro Umum - Bagian Perlengkapan', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.user_attributes (id, user_id, name, value, created_at) VALUES ('a090613e-a42a-4d3d-9c0a-ebf26b6e5c1d', '274c04e6-a811-4fe8-a940-13f0f1098a97', 'active_role', 'admin', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.user_attributes (id, user_id, name, value, created_at) VALUES ('c8520838-37da-4601-99f9-19fecbb86aa1', '274c04e6-a811-4fe8-a940-13f0f1098a97', 'available_roles', 'operator_satker,validator_wilayah,validator_pusat,admin', '2026-06-09 09:23:18.568707+00');


--
-- Data for Name: user_consent_scopes; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: user_consents; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: user_groups; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: user_policies; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.user_policies (id, user_id, policy_id, granted_by, reason, created_at, expires_at) VALUES ('639dd437-8dd9-4d29-b468-bf308b1bb6b9', '00000000-0000-0000-0000-000000000001', 'e7d90798-49ff-42dc-ba6f-3536968dd627', NULL, 'Bootstrap administrator - required for initial system configuration', '2026-06-09 09:23:18.464589+00', NULL);


--
-- Data for Name: user_roles; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.user_roles (id, user_id, role_id, created_at) VALUES ('d48ad7c8-e5e2-420b-aefd-b2010817ca15', '00000000-0000-0000-0000-000000000001', '00000000-0000-0000-0000-000000000002', '2026-06-09 09:23:17.185556+00');
INSERT INTO public.user_roles (id, user_id, role_id, created_at) VALUES ('d3f551e0-9ed1-417b-bc2d-f4a397a5bf4f', '274c04e6-a811-4fe8-a940-13f0f1098a97', 'f646c0e4-5cb4-4008-8e3b-118af6c22ede', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.user_roles (id, user_id, role_id, created_at) VALUES ('a982d892-1907-4079-ac86-f9990d94db6a', '274c04e6-a811-4fe8-a940-13f0f1098a97', '54b5d1ae-8aaa-4e31-b4b2-a77a49cfd06b', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.user_roles (id, user_id, role_id, created_at) VALUES ('a278c68e-641b-4612-b260-d3206eef45e9', '274c04e6-a811-4fe8-a940-13f0f1098a97', '9377d14c-22b6-4674-b080-a6ba43969eb1', '2026-06-09 09:23:18.568707+00');
INSERT INTO public.user_roles (id, user_id, role_id, created_at) VALUES ('0d5360cb-ac84-43e7-90a8-b289eba8521a', '274c04e6-a811-4fe8-a940-13f0f1098a97', '00000000-0000-0000-0000-000000000002', '2026-06-09 09:23:18.568707+00');


--
-- Data for Name: webauthn_audit_log; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: webauthn_challenges; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: webauthn_credentials; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- Data for Name: webauthn_credentials_old; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- PostgreSQL database dump complete
--


