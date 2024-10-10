//! Creation of compute pipelines for running microbenchmarks, see
//! [BenchmarkComputePipeline]

use std::collections::HashMap;

use thiserror::Error;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource,
    CompilationInfo, ComputePipelineDescriptor, ShaderModule,
    ShaderModuleDescriptor, ShaderSource,
};

use crate::gpu::GPUContext;

/// Represents a compute pipeline that can be used to execute one benchmark by
/// passing it to [Benchmark::run](crate::Benchmark::run).
pub struct BenchmarkComputePipeline<'a> {
    pub(crate) gpu: &'a GPUContext,
    pub(crate) shader_module: ShaderModule,
    pub(crate) bind_group: BindGroup,
    pub(crate) pipeline: wgpu::ComputePipeline,
    pub(crate) workgroups_dispatch: (u32, u32, u32),
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
                entry_point: params.entry_point,
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
            bind_group,
            pipeline,
            workgroups_dispatch: params.workgroups_dispatch,
        })
    }

    /// Get the compilation messages from compiling the shader modules
    pub async fn get_shader_compilation_info(&self) -> CompilationInfo {
        self.shader_module.get_compilation_info().await
    }
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

    /// The amount of workgroups to dispatch, the tuple represents the `(x, y,
    /// z)` dimensions of the grid of workgroups.
    pub workgroups_dispatch: (u32, u32, u32),

    /// The size of workgroups to dispatch.
    ///
    /// If [Some], the pipeline will look for and replace a `$workgroup$`
    /// placeholder with the size given here, it is expected this will be used
    /// to programatically set the `@workgroup_size` of the shader.
    ///
    /// The shader entrypoint would look like:
    ///
    /// ```wgsl
    /// @compute @workgroup_size($workgroup$)fn computeSomething( /* ... */ )
    /// ```
    pub workgroup_size: Option<(u32, u32, u32)>,
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
            ShaderSource::Wgsl(source.replace("$workgroup$", &format!("{}, {}, {}", wg_size.0, wg_size.1, wg_size.2)).into())
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

//fn print_compilation_messages(messages: &[CompilationMessage]) {
//    for message in messages {
//        print!(match message.message_type {
//    CompilationMessageType::Error => "Error: ",
//    CompilationMessageType::Warning => "Warning: ",
//    CompilationMessageType::Info => "Info: ",
//})
//    }
//}

/// A single message from the shader compilation process.
///
/// Roughly corresponds to [`GPUCompilationMessage`](https://www.w3.org/TR/webgpu/#gpucompilationmessage),
/// except that the location uses UTF-8 for all positions.
#[derive(Debug, Clone)]
pub struct CompilationMessage {
    /// The text of the message.
    pub message: String,
    /// The type of the message.
    pub message_type: CompilationMessageType,
    /// Where in the source code the message points at.
    pub location: Option<SourceLocation>,
}

/// The type of a compilation message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilationMessageType {
    /// An error message.
    Error,
    /// A warning message.
    Warning,
    /// An informational message.
    Info,
}

/// A clone of [wgpu::SourceLocation] to implement serialize/deserialize on.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SourceLocation {
    /// 1-based line number.
    pub line_number: u32,
    /// 1-based column in code units (in bytes) of the start of the span.
    /// Remember to convert accordingly when displaying to the user.
    pub line_position: u32,
    /// 0-based Offset in code units (in bytes) of the start of the span.
    pub offset: u32,
    /// Length in code units (in bytes) of the span.
    pub length: u32,
}

impl From<wgpu::CompilationMessage> for CompilationMessage {
    fn from(other: wgpu::CompilationMessage) -> Self {
        Self {
            message: other.message,
            message_type: match other.message_type {
                wgpu::CompilationMessageType::Error => {
                    CompilationMessageType::Error
                }
                wgpu::CompilationMessageType::Warning => {
                    CompilationMessageType::Warning
                }
                wgpu::CompilationMessageType::Info => {
                    CompilationMessageType::Info
                }
            },
            location: other.location.map(|other| SourceLocation {
                line_number: other.line_number,
                line_position: other.line_position,
                offset: other.offset,
                length: other.length,
            }),
        }
    }
}
