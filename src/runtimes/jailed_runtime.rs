// TODO: Remove after implementing this runtime
#![allow(clippy::derivable_impls, dead_code, unused_variables)]

use crate::{
    common::compiler::CompilationResult,
    compilers::{CompiledCode, Compiler},
};

use super::{native_runtime::NativeRuntime, CodeRuntime};

/// Jailed runtime.
/// This uses chroot jail to run the code.
/// This is only available on Linux and requires root privileges.
/// It is automatically implemented for every native compiler.
#[derive(Debug, Clone)]
pub struct JailedRuntime;

/// Jail configuration.
#[derive(Debug, Clone)]
pub struct JailedConfig {
    native_runtime_config: super::native_runtime::NativeConfig,
}

impl Default for JailedConfig {
    fn default() -> Self {
        Self {
            native_runtime_config: super::native_runtime::NativeConfig::default(),
        }
    }
}

/// Runtime for jailed code.
impl CodeRuntime for JailedRuntime {
    /// Configuration for the runtime.
    type Config = JailedConfig;
    /// Additional compilation data.
    type AdditionalData = super::native_runtime::NativeAdditionalData;
    /// Error type for the runtime.
    type Error = std::io::Error;

    /// Runs the code in a chroot jail.
    fn run(
        &self,
        code: &crate::compilers::CompiledCode<Self>,
        config: Self::Config,
    ) -> Result<super::ExecutionResult, Self::Error> {
        todo!("JailedRuntime is not implemented yet")
    }
}

/// Implementation of JailedRuntime compiler for every native compiler.
impl<C> Compiler<JailedRuntime> for C
where
    C: Compiler<NativeRuntime>,
{
    /// Configuration for the compiler.
    type Config = C::Config;

    /// Compiles the code using the native compiler.
    fn compile(
        &self,
        code: &mut impl std::io::Read,
        config: Self::Config,
    ) -> CompilationResult<crate::compilers::CompiledCode<JailedRuntime>> {
        let native_code: CompiledCode<NativeRuntime> = C::compile(self, code, config)?;
        Ok(CompiledCode {
            executable: native_code.executable.clone(),
            temp_dir_handle: native_code.temp_dir_handle.clone(),
            runtime_marker: std::marker::PhantomData,
            additional_data: native_code.additional_data.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compilers::{rust_compiler::RustCompiler, Compiler};

    // #[test]
    // fn test_compile() {
    //     let code = r#"
    //     fn main() {
    //         println!("Hello, world!");
    //     }
    //     "#;

    //     let compiled_code = RustCompiler.compile(&mut code.as_bytes(), Default::default()).unwrap();
    //     let result = JailedRuntime.run(&compiled_code, Default::default()).unwrap();

    //     assert_eq!(result.stdout, Some("Hello, world!\n".to_string()));
    // }
}
