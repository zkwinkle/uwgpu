--- UP ---

CREATE TABLE user_agent_os (
	user_agent_os_id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,

	operating_system VARCHAR(256) NOT NULL CHECK( length(operating_system) > 0 ),
	major VARCHAR(256) CHECK( length(major) > 0 ) NULL,
	minor VARCHAR(256) CHECK( length(minor) > 0 ) NULL,
	patch VARCHAR(256) CHECK( length(patch) > 0 ) NULL,
	patch_minor VARCHAR(256) CHECK( length(patch_minor) > 0 ) NULL,

	created_at TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,

	UNIQUE NULLS NOT DISTINCT(operating_system, major, minor, patch, patch_minor)
);
