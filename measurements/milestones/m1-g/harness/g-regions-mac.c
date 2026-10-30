// Lists the map entries PROC_PIDREGIONINFO walks (G's mapping count),
// with VM user tag, size, share mode and protection. libproc.h only.
#include <libproc.h>
#include <stdio.h>
#include <stdlib.h>
int main(int argc, char **argv) {
    int pid = atoi(argv[1]);
    uint64_t addr = 0; int n = 0;
    for (;;) {
        struct proc_regioninfo r;
        if (proc_pidinfo(pid, PROC_PIDREGIONINFO, addr, &r, sizeof r) != (int)sizeof r) break;
        printf("%3d %016llx %10llu KiB tag=%3u share=%u prot=%d/%d depth=%u\n", ++n,
               (unsigned long long)r.pri_address, (unsigned long long)r.pri_size / 1024,
               r.pri_user_tag, r.pri_share_mode, r.pri_protection, r.pri_max_protection, r.pri_depth);
        uint64_t next = r.pri_address + r.pri_size;
        if (next <= addr) break;
        addr = next;
    }
    return 0;
}
