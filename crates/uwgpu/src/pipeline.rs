//! Creation of compute pipelines for running microbenchmarks, see
//! [BenchmarkComputePipeline]

use std::collections::HashMap;

use thiserror::Error;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource,
    CompilationInfo, CompilationMessage, ComputePipelineDescriptor,
    ShaderModule, ShaderModuleDescriptor, ShaderSource,
};

use crate::gpu::GPUContext;

/// Represents a compute pipeline that can be used to execute one benchmark by
/// passing it to [Benchmark::run](crate::Benchmark::run).
pub struct BenchmarkComputePipeline<'a> {
    pub(crate) gpu: &'a GPUContext,
    pub(crate) shader_module: ShaderModule,
    pub(crate) bind_group_0: BindGroup,
    pub(crate) pipeline: wgpu::ComputePipeline,
}

/// This type can be used to create a [BenchmarkComputePipeline] by calling
/// [BenchmarkComputePipeline::new()].
#[derive(Clone)]
pub struct PipelineParameters<'a, 'b> {
    /// Compute shader to execute
    pub shader: ShaderModuleDescriptor<'b>,

    /// Entry point of the compute shader.
    /// Must be the name of a shader function annotated with `@compute` and no
    /// return value.
    pub entry_point: &'b str,

    /// This bind group must specify all the bindings used in the shader.
    /// The key used in the HashMap is the `n` index value of the corresponding
    /// `@binding(n)` attribute in the shader.
    ///
    /// This BindGroup will be assigned to `@group(0)` in the shader,
    /// the shader should only use that group.
    ///
    /// Note: All the executions of the benchmark will reuse this same bind
    /// group, so for example if the shader uses the same buffer for input
    /// and output (by overriding it), it will keep overriding the same
    /// buffer over and over, effectively using last iteration's output as its
    /// next iteration's input.
    pub bind_group_0: HashMap<u32, BindingResource<'b>>,

    /// GPU context that is to be used for creating this pipeline.
    pub gpu: &'a GPUContext,

    /// The size of workgroups to dispatch.
    ///
    /// If [Some], the pipeline will look for and replace every instance of
    /// `$workgroup$` placeholder with the size given here, it is expected this
    /// will be used to programatically set the `@workgroup_size` of the
    /// shader.
    ///
    /// The shader entrypoint would look like:
    ///
    /// ```wgsl
    /// @compute @workgroup_size($workgroup$)fn computeSomething( /* ... */ )
    /// ```
    ///
    /// Additionally, it will also replace the following placeholders.
    /// - `$workgroup_x$`: workgroup_size.0
    /// - `$workgroup_y$`: workgroup_size.1
    /// - `$workgroup_z$`: workgroup_size.2
    pub workgroup_size: Option<(u32, u32, u32)>,
}

impl<'a> BenchmarkComputePipeline<'a> {
    /// If the shader compilation fails this function will error. If it doesn't
    /// fail we still recommend checking
    /// [get_shader_compilation_info](Self::get_shader_compilation_info) for any
    /// warnings.
    pub async fn new(
        params: PipelineParameters<'a, '_>,
    ) -> Result<Self, CreatePipelineError> {
        let shader = {
            if let Some(workgroup_size) = params.workgroup_size {
                replace_shader_workgroup_variable(
                    &params.shader,
                    &workgroup_size,
                )
            } else {
                params.shader
            }
        };

        let shader_module = params.gpu.device.create_shader_module(shader);
        check_shader_compilation_errors(&shader_module).await?;

        let pipeline = params.gpu.device.create_compute_pipeline(
            &ComputePipelineDescriptor {
                label: None,
                layout: None,
                module: &shader_module,
                entry_point: Some(params.entry_point),
                compilation_options: Default::default(),
                cache: None,
            },
        );

        let bind_group =
            params.gpu.device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &pipeline.get_bind_group_layout(0),
                entries: &params
                    .bind_group_0
                    .into_iter()
                    .map(|(id, resource)| BindGroupEntry {
                        binding: id,
                        resource,
                    })
                    .collect::<Vec<BindGroupEntry>>(),
            });

        Ok(Self {
            gpu: params.gpu,
            shader_module,
            bind_group_0: bind_group,
            pipeline,
        })
    }

    /// Create a bind group in this pipeline
    pub fn create_bind_group(&self, params: BindGroupParams) -> BindGroup {
        self.gpu.device.create_bind_group(&BindGroupDescriptor {
            label: params.label,
            layout: &self.pipeline.get_bind_group_layout(params.group),
            entries: &params
                .entries
                .into_iter()
                .map(|(id, resource)| BindGroupEntry {
                    binding: id,
                    resource,
                })
                .collect::<Vec<BindGroupEntry>>(),
        })
    }

    /// Get the compilation messages from compiling the shader modules
    pub async fn get_shader_compilation_info(&self) -> CompilationInfo {
        self.shader_module.get_compilation_info().await
    }
}

/// Describes a group of bindings and the resources to be bound.
///
/// For use with [`BenchmarkComputePipeline::create_bind_group`].
///
/// Corresponds to [wgpu `BindGroupDescriptor`](wgpu::BindGroupDescriptor).
#[derive(Clone, Debug)]
pub struct BindGroupParams<'a, 'b> {
    /// Debug label of the bind group.
    pub label: Option<&'a str>,
    /// The group number of this bind group
    pub group: u32,
    /// The resources to bind to this bind group.
    pub entries: HashMap<u32, BindingResource<'b>>,
}

async fn check_shader_compilation_errors(
    shader_module: &ShaderModule,
) -> Result<(), CreatePipelineError> {
    let compilation_info = shader_module.get_compilation_info().await;
    if compilation_info
        .messages
        .iter()
        .any(|msg| msg.message_type == wgpu::CompilationMessageType::Error)
    {
        return Err(CreatePipelineError::ShaderCompilationError(
            compilation_info
                .messages
                .into_iter()
                .map(Into::into)
                .collect(),
        ));
    } else {
        Ok(())
    }
}

/// Inspects the shader source to replace the expected $workgroup$ variable with
/// the workgroup size given
fn replace_shader_workgroup_variable<'a>(
    shader: &'a ShaderModuleDescriptor,
    wg_size: &(u32, u32, u32),
) -> ShaderModuleDescriptor<'a> {
    let source = match &shader.source {
        ShaderSource::Wgsl(source) => {
            let mut source: String = source.replace("$workgroup$",
                &format!("{}, {}, {}", wg_size.0, wg_size.1, wg_size.2)
            ).into();

            if let Some(_) = source.find("$workgroup_x$") {
                source = source.replace("$workgroup_x$", &wg_size.0.to_string());
            }

            if let Some(_) = source.find("$workgroup_y$") {
                source = source.replace("$workgroup_y$", &wg_size.1.to_string());
            }

            if let Some(_) = source.find("$workgroup_z$") {
                source = source.replace("$workgroup_z$", &wg_size.2.to_string());
            }

            ShaderSource::Wgsl(source.into())
        },
        source @ _ => unimplemented!("No support for workgroup size variation in the given type (\"{:?}\") of shader source yet", source),
    };

    ShaderModuleDescriptor {
        label: shader.label,
        source,
    }
}

/// Error creating a [ComputePipeline]
#[derive(Debug, Clone, Error)]
pub enum CreatePipelineError {
    /// Error compiling the shader
    #[error("error compiling shader")]
    ShaderCompilationError(Vec<CompilationMessage>),
}
