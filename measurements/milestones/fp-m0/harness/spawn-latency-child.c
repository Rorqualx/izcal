/* FP-M0 spawn-latency child: write one byte to fd 3 (the inherited
 * ready pipe) as the first act after exec, then wait forever until
 * the parent kills it. Build: clang -O2 -o spawn-latency-child
 * spawn-latency-child.c
 */
#include <unistd.h>

int main(void) {
  char r = 'R';
  write(3, &r, 1);
  for (;;) pause();
  return 0;
}
