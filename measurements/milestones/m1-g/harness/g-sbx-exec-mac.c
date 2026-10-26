// Black-box: does an SBPL sandbox applied before exec carry into the new
// image, and what does the new image's phys_footprint look like?
// argv[1]=="child": report footprint and sandbox_check verdicts, then hold.
#include <libproc.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <unistd.h>
int sandbox_init_with_parameters(const char *, uint64_t, const char *const[], char **);
int sandbox_check(pid_t, const char *, int, ...);
static unsigned long long fp(void) {
    struct rusage_info_v2 ri; proc_pid_rusage(getpid(), RUSAGE_INFO_V2, (rusage_info_t *)&ri);
    return ri.ri_phys_footprint / 1024;
}
int main(int argc, char **argv) {
    if (argc > 1 && strcmp(argv[1], "child") == 0) {
        printf("child: footprint=%llu KiB sandboxed=%d file-read(/etc/hosts)=%s\n", fp(),
               sandbox_check(getpid(), NULL, 0),
               sandbox_check(getpid(), "file-read-data", 1, "/etc/hosts") ? "D" : "A");
        fflush(stdout);
        char *e2 = NULL;
        int rc2 = sandbox_init_with_parameters("(version 1)\n(deny default)\n(allow sysctl-read)\n", 0, NULL, &e2);
        printf("child: second sandbox_init rc=%d (%s) footprint=%llu KiB file-read(/usr/lib/dyld)=%s\n", rc2,
               e2 ? e2 : "", fp(), sandbox_check(getpid(), "file-read-data", 1, "/usr/lib/dyld") ? "D" : "A");
        fflush(stdout);
        if (argc > 2) sleep(atoi(argv[2]));
        return 0;
    }
    char prof[4096];
    snprintf(prof, sizeof prof,
             "(version 1)\n(deny default)\n(allow sysctl-read)\n"
             "(allow process-exec (literal \"%s\"))\n(allow file-read* (literal \"%s\") (subpath \"/System\") (subpath \"/usr\") (subpath \"/private/preboot\") (literal \"/\") (subpath \"/dev\"))\n(allow file-read-metadata)\n",
             argv[0], argv[0]);
    char *err = NULL;
    if (sandbox_init_with_parameters(prof, 0, NULL, &err) != 0) { printf("init failed: %s\n", err); return 1; }
    printf("parent: after sandbox_init footprint=%llu KiB\n", fp()); fflush(stdout);
    char *const av[] = {argv[0], "child", argc > 1 ? argv[1] : NULL, NULL};
    execv(argv[0], av);
    perror("execv"); return 1;
}
