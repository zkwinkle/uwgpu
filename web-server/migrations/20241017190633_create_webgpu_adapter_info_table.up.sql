--- UP ---

CREATE TABLE webgpu_adapter_info (
  webgpu_adapter_info_id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,

	architecture VARCHAR(256) CHECK( length(architecture) > 0 ),
	description VARCHAR(256) CHECK( length(description) > 0 ),
	device VARCHAR(256) CHECK( length(device) > 0 ),
	vendor VARCHAR(256) CHECK( length(vendor) > 0 ),

  created_at TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,

	UNIQUE(architecture, description, device, vendor)
);
