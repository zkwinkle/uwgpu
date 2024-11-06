@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> result: array<f32>;

/// Each element in this array is an array of atomic numbers, each atomic
/// number is used as a barrier to coordinate when the chunk of results
/// produced by a "workgroup size amount" of workgroups can continue
/// being processed in the next reduction phase.
///
/// Each element of the enclosing array can be seen is a set of barriers
/// for each reduction phase.
///
/// Example: if the original input holds 198 elements with a WG of 4, this
/// array should have a length of 3, each element's length should be
/// [[13], [4], [1]].
///
/// In reality though, this is created as a single buffer of 13+4+1 size.
@group(0) @binding(2) var<storage, read_write> barriers: array<atomic<u32>>;

// How many elements are left in each additional pass of the reduction.
//
// It looks very similar to the lengths of the (conceptual) barriers arrays, but
// also includes the amount of results produced by the first iteration.
//
// Because the reduction always ends up in a single element result, the
// last element of this array would always be 1. This element is
// still included because it makes the implementation easier.
//
// Example: if the original input holds 198 elements with a WG of
// 4, this array should be [50, 13, 4, 1]
@group(0) @binding(3) var<storage, read> elements_left: array<u32>;

// Shared memory for reduction within a workgroup
var<workgroup> wg_reduce: array<f32, $workgroup_x$>;
var<workgroup> wg_broadcast: u32;

fn div_round_up(n: u32, d: u32) -> u32 {
    return (n + d - 1) / d;
}

// TODO: Rename
// - $workgroup$ -> $workgroup$
// - $workgroup_x$ -> $workgroup_x$

@compute @workgroup_size($workgroup$)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>) {

    if global_id.x < arrayLength(&input) {
        wg_reduce[local_id.x] = input[global_id.x];
    } else {
        wg_reduce[local_id.x] = 0.0; // Zero pad for out-of-bounds threads
    }

    workgroupBarrier();

    // Fist local reduction loop
    var step: u32 = $workgroup_x$ / 2;
    while step > 0 {
        if local_id.x < step {
            wg_reduce[local_id.x] += wg_reduce[local_id.x + step];
        }
        step = step / 2;
        workgroupBarrier();
    }

    if local_id.x == 0 {
        result[workgroup_id.x] = wg_reduce[0];
    }

		// Case where only 1 workgroup could do the whole reduction
    if arrayLength(&elements_left) == 1 {
        return;
    }

    var global_index = global_id.x;
    var workgroup_index = workgroup_id.x;
    let thread_index = local_id.x;

    var barrier_offset: u32 = 0;

    for (var i: u32 = 0; i < (arrayLength(&elements_left) - 1); i++) {
        let barrier_index = workgroup_index / $workgroup_x$;
        if thread_index == 0 {
            wg_broadcast = atomicAdd(&barriers[barrier_offset + barrier_index], 1u);
        }
        let wg_id: u32 = workgroupUniformLoad(&wg_broadcast) + 1;
        barrier_offset = barrier_offset + elements_left[i + 1];

        let wg_completes_chunk: bool = wg_id == $workgroup_x$;
				// Note: the following line will be false if the last chunk has the exact
				// amount of workgroups as $workgroup_x$, but it doesn't matter because
				// in that case wg_completes_chunk would be true.
        let is_last_wg_of_last_chunk: bool = ((barrier_index + 1) == elements_left[i + 1]) && (wg_id == ((workgroup_index + 1) % $workgroup_x$));

        if !wg_completes_chunk && !is_last_wg_of_last_chunk {
            return;
        }

		    // Set the indexes that the remaining workgroup will work with for the
				// next pass
        workgroup_index = barrier_index;
        global_index = workgroup_index * $workgroup_x$ + thread_index;

		    // load values into local array
        if global_index < elements_left[i] {
            wg_reduce[thread_index] = result[global_index];
        } else {
            wg_reduce[thread_index] = 0.0; // Zero pad for out-of-bounds threads
        }

        workgroupBarrier();

				// reduction pass
        step = $workgroup_x$u / 2u;
        while step > 0 {
            if thread_index < step {
                wg_reduce[thread_index] += wg_reduce[thread_index + step];
            }
            step = step / 2;
            workgroupBarrier();
        }

        if thread_index == 0 {
            result[workgroup_index] = wg_reduce[0];
        }
    }
}
