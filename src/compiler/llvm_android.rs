// // src/android_compiler.rs - Integration example for Android compilation

// use crate::compiler::ast::*;
// use crate::compiler::parser::Parser;
// use crate::compiler::lexer::Lexer;
// use crate::compiler::codegen_inkwell::{InkwellAndroidCodegen, compile_to_android_object};
// use inkwell::context::Context;
// use std::fs;
// use std::path::Path;
// use std::process::Command;

// pub struct AndroidCompiler {
//     ndk_path: String,
//     target_arch: String,
//     api_level: u32,
// }

// impl AndroidCompiler {
//     pub fn new(ndk_path: String, api_level: u32) -> Self {
//         Self {
//             ndk_path,
//             target_arch: "aarch64".to_string(),
//             api_level,
//         }
//     }

//     pub fn compile_source_to_apk(
//         &self, 
//         source_code: &str, 
//         output_dir: &str
//     ) -> Result<String, Box<dyn std::error::Error>> {
//         // Step 1: Parse source code
//         let mut lexer = Lexer::new(source_code);
//         let tokens = lexer.tokenize()?;
        
//         let mut parser = Parser::new(tokens);
//         let ast = parser.parse()?;

//         // Step 2: Generate LLVM IR using Inkwell
//         let context = Context::create();
//         let mut codegen = InkwellAndroidCodegen::new(&context, "android_program");
        
//         let llvm_ir = codegen.generate(&ast)?;
        
//         // Step 3: Write LLVM IR to file
//         let ir_path = format!("{}/program.ll", output_dir);
//         fs::write(&ir_path, &llvm_ir)?;
        
//         // Step 4: Compile to object file
//         let obj_path = format!("{}/program.o", output_dir);
//         codegen.compile_to_object(&obj_path)?;
        
//         // Step 5: Link with Android runtime
//         let executable_path = self.link_android_executable(&obj_path, output_dir)?;
        
//         // Step 6: Package into APK (optional)
//         let apk_path = self.create_apk(&executable_path, output_dir)?;
        
//         Ok(apk_path)
//     }

//     fn link_android_executable(&self, obj_path: &str, output_dir: &str) -> Result<String, Box<dyn std::error::Error>> {
//         let executable_path = format!("{}/program", output_dir);
//         let sysroot = format!("{}/toolchains/llvm/prebuilt/linux-x86_64/sysroot", self.ndk_path);
//         let linker = format!("{}/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android{}-clang", 
//                            self.ndk_path, self.api_level);

//         let output = Command::new(&linker)
//             .arg("--sysroot")
//             .arg(&sysroot)
//             .arg("-pie") // Position independent executable
//             .arg("-o")
//             .arg(&executable_path)
//             .arg(obj_path)
//             .arg("-lc") // Link with Android libc
//             .arg("-lm") // Link with math library
//             .output()?;

//         if !output.status.success() {
//             let error = String::from_utf8_lossy(&output.stderr);
//             return Err(format!("Linking failed: {}", error).into());
//         }

//         Ok(executable_path)
//     }

//     fn create_apk(&self, executable_path: &str, output_dir: &str) -> Result<String, Box<dyn std::error::Error>> {
//         // Create basic APK structure
//         let apk_dir = format!("{}/apk", output_dir);
//         fs::create_dir_all(&format!("{}/lib/arm64-v8a", apk_dir))?;
//         fs::create_dir_all(&format!("{}/META-INF", apk_dir))?;

//         // Copy executable as native library
//         fs::copy(executable_path, format!("{}/lib/arm64-v8a/libprogram.so", apk_dir))?;

//         // Create AndroidManifest.xml
//         let manifest = r#"<?xml version="1.0" encoding="utf-8"?>
// <manifest xmlns:android="http://schemas.android.com/apk/res/android"
//     package="com.example.generatedapp"
//     android:versionCode="1"
//     android:versionName="1.0">
    
//     <uses-sdk android:minSdkVersion="21" android:targetSdkVersion="33" />
    
//     <application android:label="Generated App">
//         <activity android:name=".MainActivity">
//             <intent-filter>
//                 <action android:name="android.intent.action.MAIN" />
//                 <category android:name="android.intent.category.LAUNCHER" />
//             </intent-filter>
//         </activity>
//     </application>
// </manifest>"#;

//         fs::write(format!("{}/AndroidManifest.xml", apk_dir), manifest)?;

//         // Package APK using aapt (Android Asset Packaging Tool)
//         let apk_path = format!("{}/program.apk", output_dir);
//         let build_tools = format!("{}/build-tools/33.0.0", 
//                                 std::env::var("ANDROID_HOME").unwrap_or_default());
        
//         let output = Command::new(format!("{}/aapt", build_tools))
//             .arg("package")
//             .arg("-f")
//             .arg("-M")
//             .arg(format!("{}/AndroidManifest.xml", apk_dir))
//             .arg("-I")
//             .arg(format!("{}/platforms/android-33/android.jar", 
//                         std::env::var("ANDROID_HOME").unwrap_or_default()))
//             .arg("-F")
//             .arg(&apk_path)
//             .arg(&apk_dir)
//             .output()?;

//         if !output.status.success() {
//             let error = String::from_utf8_lossy(&output.stderr);
//             return Err(format!("APK packaging failed: {}", error).into());
//         }

//         Ok(apk_path)
//     }

//     pub fn compile_and_run_on_device(
//         &self,
//         source_code: &str,
//         device_id: Option<&str>
//     ) -> Result<String, Box<dyn std::error::Error>> {
//         // Create temporary directory
//         let temp_dir = tempfile::tempdir()?;
//         let output_dir = temp_dir.path().to_str().unwrap();

//         // Compile to APK
//         let apk_path = self.compile_source_to_apk(source_code, output_dir)?;

//         // Install and run on Android device using ADB
//         let mut adb_cmd = Command::new("adb");
        
//         if let Some(device) = device_id {
//             adb_cmd.arg("-s").arg(device);
//         }

//         // Install APK
//         let install_output = adb_cmd
//             .arg("install")
//             .arg("-r") // Replace existing
//             .arg(&apk_path)
//             .output()?;

//         if !install_output.status.success() {
//             let error = String::from_utf8_lossy(&install_output.stderr);
//             return Err(format!("APK installation failed: {}", error).into());
//         }

//         // Launch the app
//         let mut launch_cmd = Command::new("adb");
//         if let Some(device) = device_id {
//             launch_cmd.arg("-s").arg(device);
//         }

//         let launch_output = launch_cmd
//             .arg("shell")
//             .arg("am")
//             .arg("start")
//             .arg("-n")
//             .arg("com.example.generatedapp/.MainActivity")
//             .output()?;

//         if !launch_output.status.success() {
//             let error = String::from_utf8_lossy(&launch_output.stderr);
//             return Err(format!("App launch failed: {}", error).into());
//         }

//         Ok("App successfully compiled and launched on Android device".to_string())
//     }
// }

// // Example usage function
// pub fn example_android_compilation() -> Result<(), Box<dyn std::error::Error>> {
//     // Sample source code in your custom language
//     let source_code = r#"
//         function main() {
//             let x: i32 = 42;
//             let y: i32 = x + 8;
//             print(y);
            
//             let i: i32 = 0;
//             while i < 5 {
//                 print(i);
//                 i = i + 1;
//             }
            
//             return 0;
//         }
//     "#;

//     // Set up Android compiler
//     let ndk_path = std::env::var("ANDROID_NDK_ROOT")
//         .expect("ANDROID_NDK_ROOT environment variable not set");
    
//     let compiler = AndroidCompiler::new(ndk_path, 29); // API level 29

//     // Compile for Android
//     let output_dir = "./android_build";
//     fs::create_dir_all(output_dir)?;

//     match compiler.compile_source_to_apk(source_code, output_dir) {
//         Ok(apk_path) => {
//             println!("Successfully compiled to APK: {}", apk_path);
            
//             // Optionally run on connected device
//             if let Ok(_) = std::env::var("RUN_ON_DEVICE") {
//                 match compiler.compile_and_run_on_device(source_code, None) {
//                     Ok(result) => println!("{}", result),
//                     Err(e) => eprintln!("Failed to run on device: {}", e),
//                 }
//             }
//         }
//         Err(e) => {
//             eprintln!("Compilation failed: {}", e);
//         }
//     }

//     Ok(())
// }

// // Benchmark comparison between string-based and Inkwell codegen
// pub fn benchmark_codegen_methods(ast: &[Stmt]) -> Result<(), Box<dyn std::error::Error>> {
//     use std::time::Instant;

//     println!("Benchmarking codegen methods...");

//     // Benchmark original string-based method
//     let start = Instant::now();
//     let _string_result = crate::compiler::codegen::compile_to_llvm_android(ast)?;
//     let string_duration = start.elapsed();
    
//     // Benchmark Inkwell method
//     let start = Instant::now();
//     let context = Context::create();
//     let mut inkwell_codegen = InkwellAndroidCodegen::new(&context, "benchmark");
//     let _inkwell_result = inkwell_codegen.generate(ast)?;
//     let inkwell_duration = start.elapsed();

//     println!("String-based codegen: {:?}", string_duration);
//     println!("Inkwell codegen: {:?}", inkwell_duration);
//     println!("Inkwell is {}x faster", 
//              string_duration.as_nanos() as f64 / inkwell_duration.as_nanos() as f64);

//     Ok(())
// }

// // Error handling improvements
// #[derive(Debug)]
// pub enum AndroidCompileError {
//     ParseError(String),
//     CodegenError(String),
//     LinkError(String),
//     ApkError(String),
//     DeviceError(String),
// }

// impl std::fmt::Display for AndroidCompileError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             AndroidCompileError::ParseError(msg) => write!(f, "Parse error: {}", msg),
//             AndroidCompileError::CodegenError(msg) => write!(f, "Code generation error: {}", msg),
//             AndroidCompileError::LinkError(msg) => write!(f, "Linking error: {}", msg),
//             AndroidCompileError::ApkError(msg) => write!(f, "APK creation error: {}", msg),
//             AndroidCompileError::DeviceError(msg) => write!(f, "Device error: {}", msg),
//         }
//     }
// }

// impl std::error::Error for AndroidCompileError {}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_inkwell_basic_function() {
//         let context = Context::create();
//         let mut codegen = InkwellAndroidCodegen::new(&context, "test");
        
//         let test_func = Function {
//             name: "test".to_string(),
//             params: vec![],
//             body: vec![
//                 Stmt::Return(Some(Expr::Literal(Value::I32(42))))
//             ],
//         };

//         let stmts = vec![Stmt::Function(test_func)];
//         let result = codegen.generate(&stmts);
//         assert!(result.is_ok());
        
//         let llvm_ir = result.unwrap();
//         assert!(llvm_ir.contains("define i64 @test()"));
//         assert!(llvm_ir.contains("ret i64 42"));
//     }

//     #[test]
//     fn test_android_target_setup() {
//         let context = Context::create();
//         let codegen = InkwellAndroidCodegen::new(&context, "test");
        
//         // This test verifies that Android target setup doesn't crash
//         // In a real environment with LLVM properly installed
//         // let result = codegen.setup_android_target();
//         // assert!(result.is_ok());
//     }
// }