// Counts this process's own VM map entries two ways:
// (1) proc_pidinfo(PROC_PIDREGIONINFO) walk (G's method);
// (2) mach_vm_region_recurse at depth 0 (top-level entries of the task's
//     map; a submap such as the dyld shared region is one entry there).
// Documented: libproc.h, mach/mach_vm.h.
#include <libproc.h>
#include <mach/mach.h>
#include <mach/mach_vm.h>
#include <stdio.h>
#include <unistd.h>
int main(void) {
    int n1 = 0; uint64_t a = 0;
    for (;;) { struct proc_regioninfo r;
        if (proc_pidinfo(getpid(), PROC_PIDREGIONINFO, a, &r, sizeof r) != (int)sizeof r) break;
        n1++; uint64_t nx = r.pri_address + r.pri_size; if (nx <= a) break; a = nx; }
    int n2 = 0, subs = 0; mach_vm_address_t addr = 0;
    for (;;) { mach_vm_size_t size = 0; natural_t depth = 0;
        vm_region_submap_info_data_64_t info; mach_msg_type_number_t cnt = VM_REGION_SUBMAP_INFO_COUNT_64;
        if (mach_vm_region_recurse(mach_task_self(), &addr, &size, &depth, (vm_region_recurse_info_t)&info, &cnt) != KERN_SUCCESS) break;
        n2++; if (info.is_submap) { subs++; printf("submap entry %llx-%llx (%llu MiB)\n", addr, addr+size, size>>20); }
        addr += size; }
    printf("PROC_PIDREGIONINFO walk: %d entries; top-level map entries: %d (%d submaps)\n", n1, n2, subs);
    return 0;
}
