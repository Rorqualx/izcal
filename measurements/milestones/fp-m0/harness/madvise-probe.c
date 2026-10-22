/* FP-M0 probe: macOS accounting after MADV_FREE_REUSABLE (open question 17).
 * Allocates a 256 MB region, reads it (charging it), applies the advice
 * values one by one, and reports phys_footprint after each — using the
 * task_info API, no third-party source. Build: clang -O2 -o madvise-probe madvise-probe.c
 */
#include <stdio.h>
#include <stdlib.h>
static volatile unsigned char sink;   /* defeats dead-store elimination */
#include <string.h>
#include <sys/mman.h>
#include <mach/mach.h>

static size_t phys(void) {
    task_vm_info_data_t info;
    mach_msg_type_number_t cnt = TASK_VM_INFO_COUNT;
    if (task_info(mach_task_self(), TASK_VM_INFO, (task_info_t)&info, &cnt) != KERN_SUCCESS)
        return 0;
    return (size_t)info.phys_footprint;
}

int main(void) {
    size_t N = 256UL * 1024 * 1024;
    char *p = mmap(NULL, N, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANON, -1, 0);
    if (p == MAP_FAILED) { perror("mmap"); return 1; }
    memset(p, 1, N); sink = (unsigned char)p[N/2];   /* fault the region in */
    printf("after touch:      %6zu KB\n", phys() / 1024);
    madvise(p, N, MADV_FREE_REUSABLE);     /* the advice whose semantics Apple does not document */
    printf("FREE_REUSABLE:    %6zu KB\n", phys() / 1024);
    memset(p, 2, N); sink = (unsigned char)p[0];     /* write again: re-charge */
    printf("after re-touch:   %6zu KB\n", phys() / 1024);
    madvise(p, N, MADV_FREE);              /* macOS MADV_FREE: the comparison the spec names */
    printf("MADV_FREE:        %6zu KB\n", phys() / 1024);
    munmap(p, N);
    return 0;
}
