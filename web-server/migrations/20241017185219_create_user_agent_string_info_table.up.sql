--- UP ---

CREATE TABLE user_agent_string_info (
  user_agent_string_info_id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,

	user_agent_id UUID REFERENCES user_agent,
	user_agent_device_id UUID REFERENCES user_agent_device,
	user_agent_os_id UUID REFERENCES user_agent_os,

  created_at TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,

	UNIQUE(user_agent_id, user_agent_device_id, user_agent_os_id)
);
