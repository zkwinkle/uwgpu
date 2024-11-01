--- UP ---

CREATE TABLE platform (
  platform_id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,

	user_agent_string_info_id UUID NULL REFERENCES user_agent_string_info,
	wgpu_adapter_info_id UUID NOT NULL REFERENCES wgpu_adapter_info,
	webgpu_adapter_info_id UUID NULL REFERENCES webgpu_adapter_info,

  created_at TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,

	UNIQUE(user_agent_string_info_id, wgpu_adapter_info_id, webgpu_adapter_info_id)
);
