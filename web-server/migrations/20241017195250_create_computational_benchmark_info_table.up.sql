--- UP ---

CREATE TYPE computational_benchmark_kind AS ENUM ('matmul', 'reduction', 'convolution', 'scan');

CREATE TABLE computational_benchmark (
  computational_benchmark_id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,

  kind computational_benchmark_kind NOT NULL,
	flops DOUBLE PRECISION NOT NULL,

  created_at TIMESTAMPTZ NOT NULL DEFAULT current_timestamp
);
