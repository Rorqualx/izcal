// FP-M0 idle-wakeup counter, macOS: samples a process's wakeup counters
// (proc_pid_rusage, RUSAGE_INFO_V2: ri_pkg_idle_wkups = wakeups from the
// package idle state, ri_interrupt_wkups = all interrupt-driven wakeups)
// at the start and end of a window, and prints the per-second rates.
// Usage: wakeups-mac <pid> <seconds>
// Build: clang -O2 -o wakeups-mac wakeups-mac.c
#include <libproc.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/resource.h>
#include <unistd.h>

static int sample(pid_t pid, struct rusage_info_v2 *ri) {
  return proc_pid_rusage(pid, RUSAGE_INFO_V2, (rusage_info_t *)ri);
}

int main(int argc, char **argv) {
  if (argc != 3) { fprintf(stderr, "usage: %s <pid> <seconds>\n", argv[0]); return 2; }
  pid_t pid = (pid_t)atoi(argv[1]);
  int secs = atoi(argv[2]);
  struct rusage_info_v2 a, b;
  if (sample(pid, &a) != 0) { perror("proc_pid_rusage"); return 1; }
  sleep((unsigned)secs);
  if (sample(pid, &b) != 0) { perror("proc_pid_rusage"); return 1; }
  unsigned long long idle = b.ri_pkg_idle_wkups - a.ri_pkg_idle_wkups;
  unsigned long long intr = b.ri_interrupt_wkups - a.ri_interrupt_wkups;
  unsigned long long cpu_ns = (b.ri_user_time + b.ri_system_time) - (a.ri_user_time + a.ri_system_time);
  printf("pid=%d window=%ds idle_wakeups=%llu (%.3f/s) interrupt_wakeups=%llu (%.3f/s) cpu=%.3f ms\n",
         pid, secs, idle, (double)idle / secs, intr, (double)intr / secs, cpu_ns / 1e6);
  return 0;
}
