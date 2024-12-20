--- UP ---

CREATE TYPE wgpu_backend AS ENUM ('vulkan', 'metal', 'dx12', 'gl', 'browser_web_gpu');

CREATE TYPE wgpu_device_type AS ENUM ('unknown', 'integrated_gpu', 'discrete_gpu', 'virtual_gpu', 'cpu');

CREATE TABLE wgpu_adapter_info (
  wgpu_adapter_info_id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,

  name VARCHAR(256) CHECK( length(name) > 0 ) NULL,
  vendor INT NOT NULL,
  device INT NOT NULL,
  device_type wgpu_device_type NOT NULL,

  driver VARCHAR(256) CHECK( length(driver) > 0 ) NULL,
  driver_info VARCHAR(256) CHECK( length(driver_info) > 0 ) NULL,

  backend wgpu_backend NOT NULL,

  created_at TIMESTAMPTZ NOT NULL DEFAULT current_timestamp,

	UNIQUE NULLS NOT DISTINCT(name, vendor, device, device_type, driver, driver_info, backend)
);
