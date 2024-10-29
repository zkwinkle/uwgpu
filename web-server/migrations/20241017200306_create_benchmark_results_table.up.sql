--- UP ---

CREATE TABLE benchmark_results (
	benchmark_results_id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,

	platform_id UUID NOT NULL REFERENCES platform,

	count INT NOT NULL CHECK( count > 0 ),
	total_time_spent DOUBLE PRECISION NOT NULL CHECK( total_time_spent > 0 ),

	workgroup_size_x INT NOT NULL CHECK( workgroup_size_x > 0 ),
	workgroup_size_y INT NOT NULL CHECK( workgroup_size_y > 0 ),
	workgroup_size_z INT NOT NULL CHECK( workgroup_size_z > 0 ),

	-- The benchmark is either a computational or memory benchmark
	-- They are XORed so that one and only one must be non-null
	computational_benchmark_id UUID REFERENCES computational_benchmark,
	memory_benchmark_id UUID REFERENCES memory_benchmark,
	CHECK (
	(computational_benchmark_id IS NOT NULL AND memory_benchmark_id IS NULL) OR
	(computational_benchmark_id IS NULL AND memory_benchmark_id IS NOT NULL)
	),

	created_at TIMESTAMPTZ NOT NULL DEFAULT current_timestamp
);
