use std::ffi::CString;
use std::io;
use std::ptr;
use kvm_bindings::{kvm_regs, kvm_sregs, kvm_userspace_memory_region, kvm_run};

const KVMIO: u32 = 0xAE;
const KVM_GET_API_VERSION: u64 = libc::_IO(KVMIO, 0x00);
const KVM_CREATE_VM: u64 = libc::_IO(KVMIO, 0x01);
const KVM_GET_VCPU_MMAP_SIZE: u64 = libc::_IO(KVMIO, 0x04);
const KVM_CREATE_VCPU: u64 = libc::_IO(KVMIO, 0x41);
const KVM_SET_USER_MEMORY_REGION: u64 = libc::_IOW::<kvm_userspace_memory_region>(KVMIO, 0x46);
const KVM_RUN: u64 = libc::_IO(KVMIO, 0x80);
const KVM_GET_REGS: u64 = libc::_IOR::<kvm_regs>(KVMIO, 0x81);
const KVM_SET_REGS: u64 = libc::_IOW::<kvm_regs>(KVMIO, 0x82);
const KVM_GET_SREGS: u64 = libc::_IOR::<kvm_sregs>(KVMIO, 0x83);
const KVM_SET_SREGS: u64 = libc::_IOW::<kvm_sregs>(KVMIO, 0x84);
const KVM_EXIT_IO: u32 = 2;
const KVM_EXIT_HLT: u32 = 5;
const KVM_EXIT_IO_OUT: u8 = 1;


const MEM_SIZE: usize = 0x100000;
// We chose the serial port COM1
const PORT_EXIT: u16 = 0x3f8;
// mov dx, 0x3f8 
// out dx, al
// mov al, '\n'
// out dx, al
// hlt
const GUEST_CODE: [u8; 8] = [
    0xBA, 0xF8, 0x03, // mov dx, 0x3f8
    0xEE,             // out dx, al
    0xB0, b'\n',      // mov al, '\n'
    0xEE,             // out dx, al
    0xF4,             // hlt
];

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
    let region = kvm_userspace_memory_region {
        slot: 0,
        flags: 0,
        guest_phys_addr: 0x1000,
        memory_size: MEM_SIZE as u64,
        userspace_addr: mem as u64,
    };
    let ret = unsafe { libc::ioctl(vm_fd, KVM_SET_USER_MEMORY_REGION, &region as *const kvm_userspace_memory_region) };
    if ret < 0 {
        let err = Err(io::Error::last_os_error());
        println!("error setting up guest memory !");
        return err;
    }
    Ok(())
}

fn load_guest_code(mem: *mut libc::c_void, code: &[u8]) {
    unsafe {
        ptr::copy_nonoverlapping(code.as_ptr(), mem as *mut u8, code.len())
    }
}

fn init_vcpu_regs(vcpu_fd: i32) -> io::Result<()> {
    let mut sregs = kvm_sregs::default();
    let ret = unsafe { libc::ioctl(vcpu_fd, KVM_GET_SREGS, &mut sregs as *mut kvm_sregs) };
    if ret < 0 {
        let err = Err(io::Error::last_os_error());
        println!("error getting sregs !");
        return err;
    }
    sregs.cs.base = 0;
    sregs.cs.selector = 0;
    let ret = unsafe { libc::ioctl(vcpu_fd, KVM_SET_SREGS, &sregs as *const kvm_sregs) };
    if ret < 0 {
        let err = Err(io::Error::last_os_error());
        println!("error setting up sregs !");
        return err;
    }

    let mut regs = kvm_regs::default();
    regs.rip = 0x1000 as u64;
    // We put A in the register rax
    regs.rax = b'A' as u64;
    regs.rflags = 0x2 as u64;
    let ret = unsafe { libc::ioctl(vcpu_fd, KVM_SET_REGS, &regs as *const kvm_regs) };
    if ret < 0 {
        let err = Err(io::Error::last_os_error());
        println!("error setting up regs !");
        return err;
    }

    Ok(())
}

fn init_kvm_run(kvm: i32, vcpu_fd: i32) -> io::Result<*mut kvm_run> {
    let mmap_size = unsafe { libc::ioctl(kvm, KVM_GET_VCPU_MMAP_SIZE, 0) };
    if mmap_size < 0 {
        let err = Err(io::Error::last_os_error());
        println!("error getting vcpu mmap size !");
        return err;
    }
    let run = unsafe {
        libc::mmap(
            ptr::null_mut(),
            mmap_size as usize,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            vcpu_fd,
            0
        )
    };
    if run ==  libc::MAP_FAILED {
        return Err(io::Error::last_os_error());
    }
    Ok(run as *mut kvm_run)
}

fn run_instructions(vcpu_fd: i32, run: *mut kvm_run) -> io::Result<()> {
    loop {
        let ret = unsafe { libc::ioctl(vcpu_fd, KVM_RUN, 0) };
        if ret < 0 {
            let err = Err(io::Error::last_os_error());
            println!("error calling KVM_RUN on cpu_fd = {vcpu_fd} !");
            return err;
        }
        let exit_reason = unsafe { ptr::read_volatile(&(*run).exit_reason) };
        match exit_reason {
            KVM_EXIT_HLT => {
                println!("EXIT: HLT");
                break;
            }
            KVM_EXIT_IO => {
                let io = unsafe { &(*run).__bindgen_anon_1.io };
                if io.port == PORT_EXIT 
                    && io.direction == KVM_EXIT_IO_OUT 
                    && io.count == 1 && io.size == 1
                {
                    let byte = unsafe { *(run as *const u8).add(io.data_offset as usize) };
                    print!("{}", byte as char);
                }
                else {
                    let err = Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        format!("unhandled KVM_EXIT_IO"),
                    ));
                    println!("unhandled KVM_EXIT_IO !");
                    return err;
                }
            }
            _ => {
                let err = Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    format!("unhandled exit"),
                ));
                println!("unhandled exit !");
                return err;
            }
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let kvm_fd = open_kvm()?;
    let vm_fd = create_vm(kvm_fd)?;
    let vcpu_fd = create_vcpu(vm_fd)?;
    let mem = allocate_guest_mem()?;
    set_memory(vm_fd, mem)?;
    load_guest_code(mem, &GUEST_CODE);
    init_vcpu_regs(vcpu_fd)?;
    let run = init_kvm_run(kvm_fd, vcpu_fd)?;
    run_instructions(vcpu_fd, run)?;

    println!("Instructions run");
    Ok(())
}
