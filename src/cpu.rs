use windows;
use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, GetLastError};
use windows::Win32::System::Threading::{SetThreadGroupAffinity, GetCurrentProcess, GetCurrentThread, GetProcessGroupAffinity};
use windows::Win32::System::SystemInformation::GROUP_AFFINITY;

pub fn set_thread_affinity(id: usize) {
    let mut cpu_group = 0;
    let mut cpu_id = id;
    let total_cores = get_num_logical_cpus_ex_windows().unwrap();
    if id >= 64 {
        cpu_group = total_cores / 64;
        cpu_id = id - (cpu_group * 64);
    }
    println!("Setting affinity to group {} and id {}", cpu_group, cpu_id);
    // Convert id to mask
    let mask: usize = 1 << cpu_id;

    // Set core affinity for current thread
    unsafe {
        let ga = GROUP_AFFINITY {
            Mask: mask,
            Group: cpu_group as u16,
            Reserved: [0; 3]
        };

        let mut outga = GROUP_AFFINITY::default();

        SetThreadGroupAffinity(GetCurrentThread(),
                               &ga,
                               &mut outga);
    }
}


pub fn get_num_logical_cpus_ex_windows() -> Option<usize> {
    use std::mem;
    use std::ptr;
    use std::slice;

    #[allow(non_upper_case_globals)]
    const RelationProcessorCore: u32 = 0;

    #[repr(C)]
    #[allow(non_camel_case_types)]
    #[allow(dead_code)]
    struct GROUP_AFFINITY {
        mask: usize,
        group: u16,
        reserved: [u16; 3],
    }

    #[repr(C)]
    #[allow(non_camel_case_types)]
    #[allow(dead_code)]
    struct PROCESSOR_RELATIONSHIP {
        flags: u8,
        efficiencyClass: u8,
        reserved: [u8; 20],
        groupCount: u16,
        groupMaskTenative: [GROUP_AFFINITY; 1],
    }

    #[repr(C)]
    #[allow(non_camel_case_types)]
    #[allow(dead_code)]
    struct SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX {
        relationship: u32,
        size: u32,
        processor: PROCESSOR_RELATIONSHIP,
    }

    extern "system" {
        fn GetLogicalProcessorInformationEx(
            relationship: u32,
            data: *mut u8,
            length: &mut u32,
        ) -> bool;
    }

    // First we need to determine how much space to reserve.

    // The required size of the buffer, in bytes.
    let mut needed_size = 0;

    unsafe {
        GetLogicalProcessorInformationEx(RelationProcessorCore, ptr::null_mut(), &mut needed_size);
    }

    // Could be 0, or some other bogus size.
    if needed_size == 0 {
        return None;
    }

    // Allocate memory where we will store the processor info.
    let mut buffer: Vec<u8> = vec![0 as u8; needed_size as usize];

    unsafe {
        let result: bool = GetLogicalProcessorInformationEx(
            RelationProcessorCore,
            buffer.as_mut_ptr(),
            &mut needed_size,
        );

        if result == false {
            return None;
        }
    }

    let mut n_logical_procs: usize = 0;

    let mut byte_offset: usize = 0;
    while byte_offset < needed_size as usize {
        unsafe {
            // interpret this byte-array as SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX struct
            let part_ptr_raw: *const u8 = buffer.as_ptr().offset(byte_offset as isize);
            let part_ptr: *const SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX =
                mem::transmute::<*const u8, *const SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX>(
                    part_ptr_raw,
                );
            let part: &SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX = &*part_ptr;

            // we are only interested in RelationProcessorCore information and hence
            // we have requested only for this kind of data (so we should not see other types of data)
            if part.relationship == RelationProcessorCore {
                // the number of GROUP_AFFINITY structs in the array will be specified in the 'groupCount'
                // we tenatively use the first element to get the pointer to it and reinterpret the
                // entire slice with the groupCount
                let groupmasks_slice: &[GROUP_AFFINITY] =
                    slice::from_raw_parts(
                        part.processor.groupMaskTenative.as_ptr(),
                        part.processor.groupCount as usize);

                // count the local logical processors of the group and accumulate
                let n_local_procs: usize = groupmasks_slice
                    .iter()
                    .map(|g| g.mask.count_ones() as usize)
                    .sum::<usize>();
                n_logical_procs += n_local_procs;
            }

            // set the pointer to the next part as indicated by the size of this part
            byte_offset += part.size as usize;
        }
    }

    Some(n_logical_procs)
}
