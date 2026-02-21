pub mod llvm;

#[derive(Debug)]
pub enum OS {
    Windows, Linux
}
#[derive(Debug)]
pub enum Arch {
    X86_64, X86, Arm, Aarch64
}

#[derive(Debug)]
pub struct Target {
    pub os: OS,
    pub cpu: Arch,
    pub ptr_bits: u8
}

impl Default for Target {
    fn default () -> Self {
        let os = if cfg!(target_os = "windows") { OS::Windows } else if cfg!(target_os = "linux") { OS::Linux } else { panic!("Unsupported platform") };
        let cpu = if cfg!(target_arch = "x86") { Arch::X86 }
        else if cfg!(target_arch = "x86_64") { Arch::X86_64 }
        else if cfg!(target_arch = "arm") { Arch::Arm }
        else if cfg!(target_arch = "aarch64") { Arch::Aarch64 }
        else { panic!("Unsupported platform") };
        let ptr_bits = if cfg!(target_pointer_width = "64") { 64 } else if cfg!(target_pointer_width = "32") { 32 } else { panic!("Unsupported platform") };
        
        Self { os, cpu, ptr_bits }
    }
}