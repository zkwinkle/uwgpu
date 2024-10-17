--- UP ---

CREATE TYPE wgpu_backend AS ENUM ('vulkan', 'metal', 'dx12', 'gl', 'browser_webgpu');

CREATE TYPE wgpu_device_type AS ENUM ('unknown', 'integrated_gpu', 'discrete_gpu', 'virtual_gpu', 'browser_webgpu');

CREATE TABLE wgpu_adapter_info (
  wgpu_adapter_info_id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,

  name TEXT NOT NULL,
  vendor INT NOT NULL,
  device INT NOT NULL,
  device_type wgpu_device_type NOT NULL,

  driver TEXT NOT NULL,
  driver_info TEXT NOT NULL,

  backend wgpu_backend NOT NULL,

  created_at TIMESTAMPTZ NOT NULL DEFAULT current_timestamp
);
