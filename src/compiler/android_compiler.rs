// // src/compiler/android_compiler.rs

// use crate::compiler::ast::*;
// use crate::compiler::lexer::Lexer;
// use crate::compiler::parser::Parser;
// use crate::compiler::codegen_inkwell::InkwellAndroidCodegen;
// use inkwell::context::Context;
// use std::fs;
// use std::process::Command;

// pub struct AndroidCompiler {
//     ndk_path: String,
//     api_level: u32,
// }

// impl AndroidCompiler {
//     pub fn new(ndk_path: String, api_level: u32) -> Self {
//         Self {
//             ndk_path,
//             api_level,
//         }
//     }

//     pub fn compile_source_to_apk(
//         &self,
//         source_code: &str,
//         output_dir: &str,
//     ) -> Result<String, Box<dyn std::error::Error>> {
//         // Parse source code
//         let mut lexer = Lexer::new(source_code);
//         let tokens = lexer.tokenize()?;
        
//         let mut parser = Parser::new(tokens);
//         let ast = parser.parse()?;

//         // Generate LLVM IR using Inkwell
//         let context = Context::create();
//         let mut codegen = InkwellAndroidCodegen::new(&context, "android_program");
        
//         let _llvm_ir = codegen.generate(&ast)?;
        
//         // Compile to object file
//         let obj_path = format!("{}/program.o", output_dir);
//         codegen.compile_to_object(&obj_path)?;
        
//         // Link executable
//         let executable_path = self.link_android_executable(&obj_path, output_dir)?;
        
//         // Create simple APK structure (simplified version)
//         let apk_path = self.create_simple_apk(&executable_path, output_dir)?;
        
//         Ok(apk_path)
//     }

//     fn link_android_executable(&self, obj_path: &str, output_dir: &str) -> Result<String, Box<dyn std::error::Error>> {
//         let executable_path = format!("{}/program", output_dir);
//         let linker = format!(
//             "{}/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android{}-clang",
//             self.ndk_path, self.api_level
//         );

//         let output = Command::new(&linker)
//             .arg("-pie")
//             .arg("-o")
//             .arg(&executable_path)
//             .arg(obj_path)
//             .arg("-lc")
//             .arg("-lm")
//             .output()?;

//         if !output.status.success() {
//             let error = String::from_utf8_lossy(&output.stderr);
//             return Err(format!("Linking failed: {}", error).into());
//         }

//         Ok(executable_path)
//     }

//     fn create_simple_apk(&self, executable_path: &str, output_dir: &str) -> Result<String, Box<dyn std::error::Error>> {
//         // For now, just return the executable path
//         // In a real implementation, you'd create a proper APK structure
//         Ok(executable_path.to_string())
//     }

//     pub fn compile_and_run_on_device(
//         &self,
//         source_code: &str,
//         device_id: Option<&str>,
//     ) -> Result<String, Box<dyn std::error::Error>> {
//         // Create temporary directory
//         let temp_dir = tempfile::tempdir()?;
//         let output_dir = temp_dir.path().to_str().unwrap();

//         // Parse and compile
//         let mut lexer = Lexer::new(source_code);
//         let tokens = lexer.tokenize()?;
        
//         let mut parser = Parser::new(tokens);
//         let ast = parser.parse()?;

//         // Generate object file
//         let context = Context::create();
//         let mut codegen = InkwellAndroidCodegen::new(&context, "android_program");
//         codegen.generate(&ast)?;
        
//         let obj_path = format!("{}/program.o", output_dir);
//         codegen.compile_to_object(&obj_path)?;

//         // Link executable
//         let executable_path = self.link_android_executable(&obj_path, output_dir)?;

//         // Push to device and run
//         self.run_on_device(&executable_path, device_id)
//     }

//     fn run_on_device(&self, executable_path: &str, device_id: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
//         let mut adb_cmd = Command::new("adb");
        
//         if let Some(device) = device_id {
//             adb_cmd.arg("-s").arg(device);
//         }

//         // Push executable to device
//         let device_path = "/data/local/tmp/program";
//         let push_output = adb_cmd
//             .arg("push")
//             .arg(executable_path)
//             .arg(device_path)
//             .output()?;

//         if !push_output.status.success() {
//             let error = String::from_utf8_lossy(&push_output.stderr);
//             return Err(format!("Failed to push to device: {}", error).into());
//         }

//         // Make executable
//         let mut chmod_cmd = Command::new("adb");
//         if let Some(device) = device_id {
//             chmod_cmd.arg("-s").arg(device);
//         }
        
//         chmod_cmd
//             .arg("shell")
//             .arg("chmod")
//             .arg("755")
//             .arg(device_path)
//             .output()?;

//         // Run on device
//         let mut run_cmd = Command::new("adb");
//         if let Some(device) = device_id {
//             run_cmd.arg("-s").arg(device);
//         }

//         let run_output = run_cmd
//             .arg("shell")
//             .arg(device_path)
//             .output()?;

//         let stdout = String::from_utf8_lossy(&run_output.stdout);
//         let stderr = String::from_utf8_lossy(&run_output.stderr);

//         if !run_output.status.success() {
//             return Err(format!("Execution failed: {}", stderr).into());
//         }

//         Ok(format!("Program output:\n{}", stdout))
//     }
// }