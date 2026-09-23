use std::ffi::CString;
use std::io;

const KVMIO: u32 = 0xAE;
const KVM_GET_API_VERSION: u64 = libc::_IO(KVMIO, 0x00);

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

fn main() {
    let _ = open_kvm();
    match open_kvm() {
        Ok(kvm_fd) => println!("KVM open with success, fd = {kvm_fd}"),
        Err(e) => eprintln!("error opening KVM : {e}"),
    }
}
