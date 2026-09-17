# Journal

## 2026-09-17 — Day 1

**Done:**
- Set up the GitHub repository (`mini-vmm`), initialized with `cargo init`
- Added README and LICENSE
- Reading LWN "Using the KVM API" article

**Key takeaway:** VM and vCPU are separate fds — /dev/kvm just creates them, doesn't manage them directly.

**Next session:**
- opening /dev/kvm and creating the VM and vCPU
