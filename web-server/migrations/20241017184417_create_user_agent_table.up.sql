--- UP ---

CREATE TABLE user_agent (
  user_agent_id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,

	family VARCHAR(256) NOT NULL CHECK( length(family) > 0 ),
	major VARCHAR(256) CHECK( length(major) > 0 ),
	minor VARCHAR(256) CHECK( length(minor) > 0 ),
	patch VARCHAR(256) CHECK( length(patch) > 0 ),
	patch_minor VARCHAR(256) CHECK( length(patch_minor) > 0 ),

  created_at TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,

	UNIQUE(family, major, minor, patch, patch_minor)
);
