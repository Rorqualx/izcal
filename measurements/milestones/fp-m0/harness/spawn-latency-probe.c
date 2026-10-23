/* FP-M0 spawn-latency probe (macOS first): time from the parent's
 * spawn call to the child's "ready" byte on an inherited socketpair.
 * The child is spawn-latency-child (same dir), which writes one byte
 * to fd 3 as its first act after exec and then waits. The parent's
 * blocking read is the timer stop: spawn-to-ready, the floor the
 * spare-process gate (10 ms† — keep a spare only while spawn-to-ready
 * exceeds it) compares against. The full sandbox path is gated on the
 * sandbox itself and can only add cost. 500 runs, min/p50/p90/p99.
 * Build: clang -O2 -o spawn-latency-probe spawn-latency-probe.c
 *        clang -O2 -o spawn-latency-child spawn-latency-child.c
 * No third-party source; posix_spawn from the OS headers.
 */
#include <errno.h>
#include <signal.h>
#include <spawn.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/socket.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

extern char **environ;

static int cmp_d(const void *a, const void *b) {
  double x = *(const double *)a, y = *(const double *)b;
  return (x > y) - (x < y);
}

int main(int argc, char **argv) {
  int runs = (argc > 1) ? atoi(argv[1]) : 500;
  if (runs <= 0) runs = 500;
  const char *child_path = (argc > 2) ? argv[2] : "./spawn-latency-child";
  double *t = malloc((size_t)runs * sizeof(double));
  if (!t) { perror("malloc"); return 1; }

  for (int i = 0; i < runs; i++) {
    int sv[2];
    if (socketpair(AF_UNIX, SOCK_STREAM, 0, sv) != 0) { perror("socketpair"); return 1; }

    posix_spawn_file_actions_t fa;
    posix_spawn_file_actions_init(&fa);
    posix_spawn_file_actions_addclose(&fa, sv[0]);
    posix_spawn_file_actions_adddup2(&fa, sv[1], 3);  /* fd 3 = ready pipe */

    char *child_argv[] = {(char *)child_path, NULL};
    pid_t pid = -1;
    int rc = -1;

    struct timespec a, b;
    clock_gettime(CLOCK_MONOTONIC_RAW, &a);
    rc = posix_spawn(&pid, child_path, &fa, NULL, child_argv, environ);
    posix_spawn_file_actions_destroy(&fa);
    close(sv[1]);
    if (rc != 0) { close(sv[0]); continue; }

    char byte;
    ssize_t n = read(sv[0], &byte, 1);   /* blocks until the child is up */
    clock_gettime(CLOCK_MONOTONIC_RAW, &b);
    close(sv[0]);
    kill(pid, SIGKILL);
    waitpid(pid, NULL, 0);

    if (n != 1) continue;                /* child died before ready */
    t[i] = (double)(b.tv_sec - a.tv_sec) * 1e3 +
           (double)(b.tv_nsec - a.tv_nsec) / 1e6;
  }

  qsort(t, (size_t)runs, sizeof(double), cmp_d);
  printf("runs=%d  min=%.3f ms  p50=%.3f ms  p90=%.3f ms  p99=%.3f ms  max=%.3f ms\n",
         runs, t[0], t[runs / 2], t[runs * 90 / 100], t[runs * 99 / 100], t[runs - 1]);
  free(t);
  return 0;
}
