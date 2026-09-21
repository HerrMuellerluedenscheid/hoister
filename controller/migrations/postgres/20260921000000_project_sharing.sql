-- Project sharing: a project stays owned by `project.user_id` (the account
-- whose agent reports it); co-maintainers get access through
-- `project_member`. People without an account yet are tracked as pending
-- `project_invitation`s keyed by email and turned into memberships once they
-- sign up with that (verified) address.

CREATE TABLE project_member (
    project_id UUID NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    user_id VARCHAR(128) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    invited_by VARCHAR(128) REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (project_id, user_id)
);

CREATE INDEX project_member_user_id_idx ON project_member(user_id);

-- `email` is stored lower-cased; matching on claim is exact.
CREATE TABLE project_invitation (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    email VARCHAR(320) NOT NULL,
    invited_by VARCHAR(128) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(project_id, email)
);

CREATE INDEX project_invitation_email_idx ON project_invitation(email);

-- Project-scoped notifiers. NULL keeps the existing account-wide behaviour
-- (every event of every project the user owns); a project id limits the
-- notifier to that project's events and shares it with the project members.
-- `user_id` of a project notifier is always the project owner, so plan
-- limits and account deletion apply exactly as for account-wide notifiers.
ALTER TABLE notifier ADD COLUMN project_id UUID REFERENCES project(id) ON DELETE CASCADE;

CREATE INDEX notifier_project_id_idx ON notifier(project_id);
