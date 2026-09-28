use std::ffi::CString;
use std::io;
use std::ptr;

const KVMIO: u32 = 0xAE;
const KVM_GET_API_VERSION: u64 = libc::_IO(KVMIO, 0x00);
const KVM_CREATE_VM: u64 = libc::_IO(KVMIO, 0x01);
const KVM_CREATE_VCPU: u64 = libc::_IO(KVMIO, 0x41);
const MEM_SIZE: usize = 0x100000;
const KVM_SET_USER_MEMORY_REGION: u64 = libc::_IOW::<KvmUserspaceMemoryRegion>(KVMIO, 0x46);

#[repr(C)]
struct KvmUserspaceMemoryRegion {
    slot: u32,
    flags: u32,
    guest_phys_addr: u64,
    memory_size: u64,
    userspace_addr: u64,
}

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

fn allocate_guest_mem() -> io::Result<*mut libc::c_void> {
    let addr = unsafe {
        libc::mmap(
            ptr::null_mut(),
            MEM_SIZE,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED | libc::MAP_ANONYMOUS,
            -1,
            0
        )
    };
    if addr ==  libc::MAP_FAILED {
        return Err(io::Error::last_os_error());
    }
    return Ok(addr);
}

fn set_memory(vm_fd: i32, mem: *mut libc::c_void) -> io::Result<()> {
    let region = KvmUserspaceMemoryRegion {
        slot: 0,
        flags: 0,
        guest_phys_addr: 0x1000,
        memory_size: MEM_SIZE as u64,
        userspace_addr: mem as u64,
    };
    let ret = unsafe { libc::ioctl(vm_fd, KVM_SET_USER_MEMORY_REGION, &region as *const KvmUserspaceMemoryRegion) };
    if ret < 0 {
        let err = Err(io::Error::last_os_error());
        println!("error setting up guest memory !");
        return err;
    }
    Ok(())
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

    let mem = match allocate_guest_mem() {
        Ok(p) => p,
        Err(e) => {
            eprintln! ("error mmap ! {e}");
            return;
        }
    };

    if let Err(e) = set_memory(vm_fd, mem) {
        eprintln!("error setting up guest memory : {e}");
        return;
    }

    println!("Guest memory installed : guest_phys 0x1000, size {MEM_SIZE}");
}
