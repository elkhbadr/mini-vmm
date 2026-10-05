use std::io;
use std::ptr;
use kvm_bindings::kvm_userspace_memory_region;
use kvm_ioctls::{Kvm, VcpuExit, VcpuFd, VmFd};

const KVM_API_VERSION: i32 = 12;
const IN_VALUE: u8 = 0x67;
const MEM_SIZE: usize = 0x100000;
// We chose the serial port COM1
const PORT_OUT: u16 = 0x3f8;
const PORT_IN: u16 = 0x3f9;

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
// mov dx, 0x3f8 
// mov si, 0x100C
// mov cx, 13
// rep outsb
// hlt
const GUEST_CODE2: [u8; 25] = [
    0xBA, 0xF8, 0x03, // mov dx, 0x3f8
    0xBE, 0x0C, 0x10, // mov si, 0x100C
    0xB9, 0x0D, 0x00, // mov cx, 13
    0xF3, 0x6e,       // rep outsb
    0xF4,             // hlt
    b'H', b'e', b'l', b'l', b'o', b' ',
    b'W', b'o', b'r', b'l', b'd', b'!', b'\n',
];
// mov dx, 0x3f9
// in al, dx
// hlt
const GUEST_CODE_IN: [u8; 5] =  [
    0xBA, 0xF9, 0x03, // mov dx, 0x3f9
    0xEC,             // in al, dx
    0xF4,             // hlt
];

struct SerialDevice {
    in_value: u8
}

impl SerialDevice {
    fn new(in_value: u8) -> Self {
        Self { in_value }
    }

    fn handle_out(&mut self, data: &[u8]) {
        for &byte in data {
            print!("{}", byte as char);
        }
    }

    fn handle_in(&mut self) -> u8 {
        self.in_value
    }
}

fn open_kvm() -> io::Result<Kvm> {
    let kvm = Kvm::new()?;
    let version = kvm.get_api_version();

    if version != KVM_API_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("kvm version not supported : {version}"),
        ));
    }
    println!("kvm version is {}", version);
    Ok(kvm)
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
    Ok(addr)
}

fn set_memory(vm: &VmFd, mem: *mut libc::c_void) -> io::Result<()> {
    let region = kvm_userspace_memory_region {
        slot: 0,
        flags: 0,
        guest_phys_addr: 0x1000,
        memory_size: MEM_SIZE as u64,
        userspace_addr: mem as u64,
    };

    unsafe { vm.set_user_memory_region(region)? };

    Ok(())
}

fn load_guest_code(mem: *mut libc::c_void, code: &[u8]) {
    unsafe {
        ptr::copy_nonoverlapping(code.as_ptr(), mem as *mut u8, code.len())
    }
}

fn init_vcpu_regs(vcpu: &VcpuFd) -> io::Result<()> {
    let mut sregs = vcpu.get_sregs()?;
    sregs.cs.base = 0;
    sregs.cs.selector = 0;
    vcpu.set_sregs(&sregs)?;

    let mut regs = vcpu.get_regs()?;
    regs.rip = 0x1000 as u64;
    // We put 0xAABBCCDD in the register rax for testing when changing al
    regs.rax = 0xAABBCCDD as u64;
    regs.rflags = 0x2 as u64;
    vcpu.set_regs(&regs)?;

    Ok(())
}

fn run_instructions(vcpu: &mut VcpuFd, serial: &mut SerialDevice) -> io::Result<()> {
    loop {
        match vcpu.run()? {
            VcpuExit::Hlt => {
                println!("EXIT: HLT");
                let regs = vcpu.get_regs()?;
                println!("rax = {:#018x}", regs.rax);
                println!("al  = {:#04x}", regs.rax & 0xff);
                println!("rip = {:#x}", regs.rip);
                break;
            }
            VcpuExit::IoOut(port, data) if port == PORT_OUT => {
                serial.handle_out(data);
            }
            VcpuExit::IoIn(port, data) if port == PORT_IN => {
                for b in data.iter_mut() {
                    *b = serial.handle_in();
                }
            }
            
            exit => {
                let err = Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    format!("unhandled exit: {exit:?}"),
                ));
                return err;
            }
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let kvm = open_kvm()?;
    let vm = kvm.create_vm()?;
    let mut vcpu = vm.create_vcpu(0)?;
    let mem = allocate_guest_mem()?;
    set_memory(&vm, mem)?;
    load_guest_code(mem, &GUEST_CODE_IN);
    init_vcpu_regs(&vcpu)?;

    let mut serial = SerialDevice::new(IN_VALUE);
    run_instructions(&mut vcpu, &mut serial)?;

    println!("Instructions run");
    Ok(())
}
