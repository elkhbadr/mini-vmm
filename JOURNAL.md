# Journal

## 2026-09-17 — Day 1

**Done:**
- Set up the GitHub repository (`mini-vmm`), initialized with `cargo init`
- Added README and LICENSE
- Reading LWN "Using the KVM API" article
- Adding libc dependency

**Key takeaway:** VM and vCPU are separate fds — /dev/kvm just creates them, doesn't manage them directly.

**Next session:**
- opening /dev/kvm and creating the VM and vCPU

## 2026-09-18 — Day 2

**Done:**
- Learning the basics of Rust

## 2026-09-19 — Day 3

**Done:**
- Opening /dev/kvm

## 2026-09-24 — Day 4

**Done:**
- Creating VM and vCPU

## 2026-09-25 — Day 5

**Done:**
- Allocating memory

## 2026-09-28 — Day 6

**Done:**
- Setting up guest memory region

## 2026-09-29 — Day 7

**Done:**
- Adding kvm-bindings dependency

## 2026-09-30 — Day 8

**Done:**
- Mmap the structure kvm_run
- Test of the guest code that displays value of a register

## 2026-09-30 — Day 9

**Done:**
- Handle string I/O (count > 1) in KVM_EXIT_IO