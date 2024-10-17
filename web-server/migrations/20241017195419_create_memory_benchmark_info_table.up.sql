--- UP ---

CREATE TYPE memory_benchmark_kind AS ENUM ('buffer_sequential', 'buffer_shuffled', 'buffer_to_texture', 'texture_to_texture');

CREATE TABLE memory_benchmark (
  memory_benchmark_id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,

  kind memory_benchmark_kind NOT NULL,
	bandwidth DOUBLE PRECISION NOT NULL,

  created_at TIMESTAMPTZ NOT NULL DEFAULT current_timestamp
);
