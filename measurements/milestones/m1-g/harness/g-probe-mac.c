// M1 phase G: one reading of a macOS process for the renderer-skeleton row.
// Prints: phys_footprint_bytes interrupt_wakeups pkg_idle_wakeups regions
// - phys_footprint: FP 6.2's charged memory on macOS (no graphics memory
//   is owned by these processes, so nothing is subtracted).
// - wakeups: proc_pid_rusage RUSAGE_INFO_V2, as FP-M0's wakeups-mac.c.
// - regions: the VM regions proc_pidinfo(PROC_PIDREGIONINFO) walks, from
//   address 0 up; FP 6.20.6's mapping count on macOS.
// Documented APIs only (libproc.h, sys/resource.h).
// Build: clang -O2 -o g-probe-mac g-probe-mac.c
#include <libproc.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/resource.h>

int main(int argc, char **argv) {
    if (argc != 2) { fprintf(stderr, "usage: g-probe-mac PID\n"); return 2; }
    int pid = atoi(argv[1]);
    struct rusage_info_v2 ri;
    if (proc_pid_rusage(pid, RUSAGE_INFO_V2, (rusage_info_t *)&ri) != 0) {
        perror("proc_pid_rusage"); return 1;
    }
    unsigned long long regions = 0;
    uint64_t addr = 0;
    for (;;) {
        struct proc_regioninfo r;
        int n = proc_pidinfo(pid, PROC_PIDREGIONINFO, addr, &r, sizeof r);
        if (n != (int)sizeof r) break;
        regions++;
        uint64_t next = r.pri_address + r.pri_size;
        if (next <= addr) break;
        addr = next;
    }
    printf("%llu %llu %llu %llu\n", (unsigned long long)ri.ri_phys_footprint,
           (unsigned long long)ri.ri_interrupt_wkups,
           (unsigned long long)ri.ri_pkg_idle_wkups, regions);
    return 0;
}
