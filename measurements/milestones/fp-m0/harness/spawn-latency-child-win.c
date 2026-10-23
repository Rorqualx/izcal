/* FP-M0 spawn-latency child, Windows flavour: write one byte to the
 * inherited pipe handle passed as argv[1], then sleep until the
 * parent terminates us. Build: cl /O2 spawn-latency-child-win.c
 */
#include <stdlib.h>
#include <windows.h>

int main(int argc, char **argv) {
  if (argc < 2) return 1;
  HANDLE h = (HANDLE)(uintptr_t)_strtoui64(argv[1], NULL, 10);
  char r = 'R';
  DWORD n = 0;
  WriteFile(h, &r, 1, &n, NULL);
  Sleep(INFINITE);
  return 0;
}
