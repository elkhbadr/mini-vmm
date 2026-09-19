use std::ffi::CString;

const KVMIO: u32 = 0xAE;
const KVM_GET_API_VERSION: u64 = libc::_IO(KVMIO, 0x00);

fn open_kvm() -> i32 {
    let path_kvm = CString::new("/dev/kvm").unwrap();
    let kvm = unsafe { libc::open(path_kvm.as_ptr(), libc::O_RDWR | libc::O_CLOEXEC) };
    let ret = unsafe { libc::ioctl(kvm, KVM_GET_API_VERSION, 0) };
    if ret < 0 {
        println!("error getting kvm version !");
        return 0 ;
    }
    if ret != 12 {
        println!("kvm version not supported !");
        return 0;
    }
    println!("kvm version is {}", ret);
    return kvm;
}   

fn main() {
    open_kvm();
}
