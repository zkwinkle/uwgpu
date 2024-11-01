--- UP ---

CREATE TABLE user_agent_device (
  user_agent_device_id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,

  device VARCHAR(256) NOT NULL CHECK( length(device) > 0 ),
  brand VARCHAR(256) CHECK( length(brand) > 0 ) NULL,
  model VARCHAR(256) CHECK( length(model) > 0 ) NULL,

  created_at TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,

	UNIQUE NULLS NOT DISTINCT(device, brand, model)
);
