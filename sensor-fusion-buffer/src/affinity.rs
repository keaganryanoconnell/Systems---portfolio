use crate::error::{FusionBufferError, Result};

pub struct CpuAffinity;

impl CpuAffinity {
    #[cfg(target_os = "linux")]
    pub fn pin_current(core_id: u32) -> Result<()> {
        unsafe {
            let mut cpu_set: libc::cpu_set_t = std::mem::zeroed();
            let core_count = libc::sysconf(libc::_SC_NPROCESSORS_ONLN) as usize;

            if core_id as usize >= core_count {
                return Err(FusionBufferError::AffinityFailed(format!(
                    "core {} exceeds available {} cores",
                    core_id, core_count
                )));
            }

            libc::CPU_SET(core_id as usize, &mut cpu_set);

            let ret = libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &cpu_set);

            if ret != 0 {
                let err = std::io::Error::last_os_error();
                return Err(FusionBufferError::AffinityFailed(format!(
                    "sched_setaffinity failed on core {}: {}",
                    core_id, err
                )));
            }
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn pin_current(core_id: u32) -> Result<()> {
        unsafe {
            use windows_sys::Win32::System::Threading::{GetCurrentThread, SetThreadAffinityMask};

            let mask: usize = 1usize << core_id;
            let prev = SetThreadAffinityMask(GetCurrentThread(), mask);

            if prev == 0 {
                let err = std::io::Error::last_os_error();
                return Err(FusionBufferError::AffinityFailed(format!(
                    "SetThreadAffinityMask failed on core {}: {}",
                    core_id, err
                )));
            }
        }
        Ok(())
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    pub fn pin_current(core_id: u32) -> Result<()> {
        if core_id > 0 {
            return Err(FusionBufferError::AffinityFailed(
                "CPU affinity not supported on this platform".into(),
            ));
        }
        Ok(())
    }

    pub fn available_cores() -> u32 {
        std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(1)
    }
}
