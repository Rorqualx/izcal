/* FP-M0 spawn-latency probe, Windows flavour: time from the parent's
 * CreateProcess call to the child's "ready" byte on an inherited pipe.
 * The child (spawn-latency-child-win.exe) writes one byte to the pipe
 * handle it receives on its command line, then sleeps. The parent's
 * blocking ReadFile is the timer stop: spawn-to-ready, the floor the
 * spare-process gate (10 ms†) compares against; the sandbox path is
 * gated on the sandbox itself and can only add cost. 500 runs,
 * min/p50/p90/p99. Build (vcvars64 first):
 *   cl /O2 spawn-latency-probe-win.c
 *   cl /O2 spawn-latency-child-win.c
 * No third-party source; Win32 API only.
 */
#include <stdio.h>
#include <stdlib.h>
#include <windows.h>

static int cmp_d(const void *a, const void *b) {
  double x = *(const double *)a, y = *(const double *)b;
  return (x > y) - (x < y);
}

int main(int argc, char **argv) {
  int runs = (argc > 1) ? atoi(argv[1]) : 500;
  if (runs <= 0) runs = 500;
  double *t = (double *)malloc((size_t)runs * sizeof(double));
  if (!t) return 1;

  LARGE_INTEGER freq;
  QueryPerformanceFrequency(&freq);

  for (int i = 0; i < runs; i++) {
    SECURITY_ATTRIBUTES sa = { sizeof(sa), NULL, TRUE };
    HANDLE rd, wr;
    if (!CreatePipe(&rd, &wr, &sa, 0)) continue;

    char cmdline[512];
    _snprintf_s(cmdline, sizeof(cmdline), _TRUNCATE,
                "spawn-latency-child-win.exe %llu",
                (unsigned long long)(uintptr_t)wr);

    STARTUPINFO si = { sizeof(si) };
    PROCESS_INFORMATION pi = { 0 };
    LARGE_INTEGER a, b;
    QueryPerformanceCounter(&a);
    if (!CreateProcessA(NULL, cmdline, NULL, NULL, TRUE, 0, NULL, NULL, &si, &pi)) {
      CloseHandle(rd); CloseHandle(wr); continue;
    }
    CloseHandle(wr);  /* the child holds its inherited copy */

    char byte; DWORD got = 0;
    ReadFile(rd, &byte, 1, &got, NULL);   /* blocks until the child is up */
    QueryPerformanceCounter(&b);
    CloseHandle(rd);
    TerminateProcess(pi.hProcess, 0);
    CloseHandle(pi.hProcess); CloseHandle(pi.hThread);

    if (got != 1) continue;
    t[i] = (double)(b.QuadPart - a.QuadPart) * 1e3 / (double)freq.QuadPart;
  }

  qsort(t, (size_t)runs, sizeof(double), cmp_d);
  printf("runs=%d  min=%.3f ms  p50=%.3f ms  p90=%.3f ms  p99=%.3f ms  max=%.3f ms\n",
         runs, t[0], t[runs / 2], t[runs * 90 / 100], t[runs * 99 / 100], t[runs - 1]);
  free(t);
  return 0;
}
