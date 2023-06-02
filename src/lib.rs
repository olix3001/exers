//! # Exers
//! Exers is a tool for compiling and running code in various languages and runtimes.
//!
//! ## Support
//! Currently supported languages can be found in [compilers](crate::compilers) module.
//! And supported runtimes can be found in [runtimes](crate::runtimes) module.
//!
//! ## Usage
//! This crate provides two main elements:
//! - [Compilers](crate::compilers) - for compiling code
//! - [Runtimes](crate::runtimes) - for running code
//!
//! Each compiler implements some kind of [Compiler](crate::compilers::Compiler) trait. <br/>
//! Some of them may not support all runtimes, so I recommend checking the documentation of each compiler.
//! Compilers also have some kind of config object, which is used to configure the compiler. <br/>
//! All compiler configs implement [Default](std::default::Default) trait, so you can use `Default::default()` to get default config.
//!
//! Each runtime implements some kind of [CodeRuntime](crate::runtimes::CodeRuntime) trait.
//! Runtimes also have some kind of config object, which is used to configure the runtime. <br/>
//! All runtime configs implement [Default](std::default::Default) trait, so you can use `Default::default()` to get default config.
//!
//! Compilers return [`CompiledCode<R: CodeRuntime>`](crate::compilers::CompiledCode) object,
//! which contains executable file (in temporary directory) and additional data for the runtime.
//!
//! Runtimes take [`CompiledCode<R: CodeRuntime>`](crate::compilers::CompiledCode) object and run it.
//!
//! ## Example
//! ```rust, no_run
//! // Create compiler.
//! let compiler = RustCompiler;
//!
//! // Create runtime.
//! let runtime = NativeRuntime;
//!
//! // Our code
//! let code = r#"
//!     fn main() {
//!        println!("Hello, world!");
//!     }
//! "#;
//!
//! // Compile the code. Code can be any kind of object that implements (Read)[std::io::Read] trait.
//! let compiled = compiler.compile(&mut code.as_bytes(), Default::default()).unwrap();
//!     
//! // Run the code. Native runtime just runs the executable file.
//! let result = runtime.run(&compiled, Default::default()).unwrap();
//!
//! // Print the result.
//! println!("stdout: {}", result.stdout.unwrap());
//! ```

#![allow(clippy::clone_double_ref, clippy::uninlined_format_args)]

pub mod common;
pub mod compilers;
pub mod runtimes;
