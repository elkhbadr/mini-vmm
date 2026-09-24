use std::ffi::CString;
use std::io;

const KVMIO: u32 = 0xAE;
const KVM_GET_API_VERSION: u64 = libc::_IO(KVMIO, 0x00);
const KVM_CREATE_VM: u64 = libc::_IO(KVMIO, 0x01);
const KVM_CREATE_VCPU: u64 = libc::_IO(KVMIO, 0x41);

fn open_kvm() -> io::Result<i32> {
    let path_kvm = CString::new("/dev/kvm").unwrap();
    let kvm = unsafe { libc::open(path_kvm.as_ptr(), libc::O_RDWR | libc::O_CLOEXEC) };

    if kvm < 0 {
        let err = Err(io::Error::last_os_error());
        return err;
    }

    let ret = unsafe { libc::ioctl(kvm, KVM_GET_API_VERSION, 0) };
    if ret < 0 {
        let err = Err(io::Error::last_os_error());
        unsafe { libc::close(kvm) };
        println!("error getting kvm version !");
        return err;
    }
    if ret != 12 {
        let err = Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("kvm version not supported : {ret}"),
        ));
        unsafe { libc::close(kvm) };
        println!("kvm version not supported !");
        return err;
    }
    println!("kvm version is {}", ret);
    return Ok(kvm);
}

fn create_vm(kvm: i32) -> io::Result<i32> {
    let vm_fd = unsafe { libc::ioctl(kvm, KVM_CREATE_VM, 0) };

    if vm_fd < 0 {
        let err = Err(io::Error::last_os_error());
        return err;
    }
    return Ok(vm_fd);
}

fn create_vcpu(vm_fd: i32) -> io::Result<i32> {
    let vcpu_fd = unsafe { libc::ioctl(vm_fd, KVM_CREATE_VCPU, 0) };

    if vcpu_fd < 0 {
        let err = Err(io::Error::last_os_error());
        return err;
    }
    return Ok(vcpu_fd);
}

fn main() {
    let kvm_fd = match open_kvm() {
        Ok(fd) => {
            println!("KVM open with success, fd = {fd}");
            fd
        }
        Err(e) => {
            eprintln!("error opening KVM : {e}");
            return;
        }
    };

    let vm_fd = match create_vm(kvm_fd) {
        Ok(fd) => {
            println!("VM created with success, fd = {fd}");
            fd
        }
        Err(e) => {
            eprintln!("error creating VM : {e}");
            return;
        }
    };

    let vcpu_fd = match create_vcpu(vm_fd) {
        Ok(fd) => {
            println!("vCPU created with success, fd = {fd}");
            fd
        }
        Err(e) => {
            eprintln!("error creating vCPU : {e}");
            return;
        }
    };

    println!("vCPU fd = {vcpu_fd}");
}
