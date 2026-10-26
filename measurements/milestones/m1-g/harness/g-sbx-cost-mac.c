// Black-box: what sandbox_init_with_parameters costs in phys_footprint and
// in the default malloc zone, with Izcal's empty-process profile.
// Documented APIs: libproc (proc_pid_rusage), malloc_zone_statistics
// (malloc/malloc.h), sandbox_init_with_parameters (as Izcal declares it).
#include <libproc.h>
#include <malloc/malloc.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <unistd.h>

int sandbox_init_with_parameters(const char *profile, uint64_t flags,
                                 const char *const parameters[], char **errorbuf);

static void show(const char *what) {
    struct rusage_info_v2 ri;
    proc_pid_rusage(getpid(), RUSAGE_INFO_V2, (rusage_info_t *)&ri);
    malloc_statistics_t st;
    malloc_zone_statistics(NULL, &st);
    printf("%-28s footprint=%6llu KiB  malloc in_use=%6zu KiB touched=%6zu KiB\n", what,
           (unsigned long long)ri.ri_phys_footprint / 1024, st.size_in_use / 1024,
           st.max_size_in_use / 1024);
}

int main(int argc, char **argv) {
    show("start");
    char *err = NULL;
    int rc = sandbox_init_with_parameters("(version 1)\n(deny default)\n(allow sysctl-read)\n",
                                          0, NULL, &err);
    show(rc == 0 ? "after sandbox_init" : "sandbox_init FAILED");
    if (argc > 1 && strcmp(argv[1], "pin") == 0) {
        void *pin = calloc(1, 81920);
        show("after 80K calloc (pin)");
        size_t r = malloc_zone_pressure_relief(NULL, 0);
        printf("pressure_relief returned %zu KiB\n", r / 1024);
        show("after pressure_relief");
        sleep(2); show("2 s later");
        (void)pin;
    }
    if (argc > 1 && strcmp(argv[1], "relief") == 0) {
        size_t r = malloc_zone_pressure_relief(NULL, 0);
        printf("pressure_relief returned %zu bytes\n", r);
        show("after pressure_relief");
    }
    if (argc > 1 && strcmp(argv[1], "hold") == 0) {
        for (int i = 0; i < 40; i++) { sleep(15); show("held"); fflush(stdout); }
    }
    return 0;
}
