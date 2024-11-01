use ovmf_prebuilt::{Arch, FileType, Source};
use std::path::PathBuf;

fn main() {
    let uefi_path = env!("UEFI_PATH");
    let bios_path = env!("BIOS_PATH");

    eprintln!("UEFI path: {}", uefi_path);
    eprintln!("BIOS path: {}", bios_path);

    let uefi = true;

    let mut cmd = std::process::Command::new("qemu-system-x86_64");

    if uefi {
        let prebuilt_dir = PathBuf::from("target").join("ovmf");

        let prebuilt = ovmf_prebuilt::Prebuilt::fetch(Source::LATEST, prebuilt_dir).unwrap();

        let code_path = prebuilt.get_file(Arch::X64, FileType::Code);
        let vars_path = prebuilt.get_file(Arch::X64, FileType::Vars);

        eprintln!("Code path: {}", code_path.display());
        eprintln!("Vars path: {}", vars_path.display());

        cmd.arg("-drive").arg(format!(
            "if=pflash,format=raw,readonly=on,file={}",
            code_path.display()
        ));

        cmd.arg("-drive")
            .arg(format!("if=pflash,format=raw,file={}", vars_path.display()));

        cmd.arg("-drive")
            .arg(format!("format=raw,file={uefi_path}"));
    } else {
        cmd.arg("-drive")
            .arg(format!("format=raw,file={bios_path}"));
    }

    cmd.arg("-serial").arg("stdio");

    let mut child = cmd.spawn().unwrap();

    child.wait().unwrap();
}
