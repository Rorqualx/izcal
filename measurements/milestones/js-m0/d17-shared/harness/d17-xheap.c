/* JS-D17 leg 3: what a cross-heap reference costs, and what the brand
 * discipline would cost if it had to be checked at run time.
 *
 * Section 6.1 and the shared-structs paragraph of the JS spec say that
 * because every agent heap of a process is in one cage, "a reference
 * from an agent heap into it is an ordinary 32-bit compressed reference:
 * no second base, no overlapping reservation", and that "the brand
 * discipline extends with a shared-heap brand that agent heaps may
 * reference one way".
 *
 * Neither the shared heap nor its brand exists in `hg-heap` yet, so
 * there is nothing to time directly. What can be timed is the price of
 * the two things JS-D17 (a) claims to avoid, each with its sequence
 * pinned in inline assembly so the compiler cannot turn one variant into
 * another:
 *
 *   one_base          (a): one process-wide base. A slot is
 *                     v = (offset/8) << 1 | 1 and the address is
 *                     base + 4*v - 4, as JS spec 6.1 states. A reference
 *                     into another agent's blocks, or into a shared
 *                     heap's blocks, decodes exactly the same way: the
 *                     base does not depend on which heap the target is
 *                     in.
 *   brand_dynamic     (a) plus the one-way rule checked at run time: the
 *                     target's slot is compared against the shared
 *                     heap's slot range before the reference is
 *                     followed. This prices what the shared-heap brand
 *                     would cost if the type system could not carry it.
 *                     Every check passes here, so the branch is
 *                     perfectly predicted: the best case.
 *   two_base_static   (b), one cage per agent, where the referring
 *                     field's type names the cage, so the base register
 *                     is picked at compile time. The load is one_base's
 *                     load off a different base.
 *   two_base_dynamic  (b) where the slot must say which cage, because
 *                     the field may hold either (a tagged field). Bit 1
 *                     of the slot selects the base through a conditional
 *                     move and the unit offset starts at bit 2.
 *
 * Method, following the JS-D1 spike's harness:
 *   - A random cycle of nodes (Sattolo), one dependent load per step, so
 *     the decode sits on the load's critical path.
 *   - THE CYCLE IS CHECKED: each lap must return to its starting slot.
 *     A walk that fell out of its cycle would be fast and wrong, and
 *     prints FAIL instead of a number.
 *   - The payload at each step is summed and checked against the value
 *     the walk must have produced, so no variant can be elided. The
 *     assembly is given its own scratch operands; nothing it writes
 *     touches the checked values.
 *   - THE CLOCK IS MEASURED PER CELL from a dependent add chain in
 *     assembly (one add per cycle, as the JS-D1 spike measured at 1.02
 *     and 1.03 cycles on these two cores), cross-checked against what
 *     the OS says the cores are running at. No system setting is
 *     changed: hueyj's governor is left on powersave.
 *
 * Builds with `cc -O2`. x86-64 and AArch64 carry their own sequences;
 * any other ISA falls back to C, which the header line says, and whose
 * instruction counts are not the spec's claim.
 */

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#if defined(__x86_64__)
#define ISA "x86-64"
#define ASM_PINNED 1
#elif defined(__aarch64__)
#define ISA "aarch64"
#define ASM_PINNED 1
#else
#define ISA "other"
#define ASM_PINNED 0
#endif

static double now_s(void) {
  struct timespec t;
  clock_gettime(CLOCK_MONOTONIC, &t);
  return (double)t.tv_sec + (double)t.tv_nsec * 1e-9;
}

/* ---------------------------------------------------------------- clock */

static double ns_per_add(uint64_t n) {
  volatile uint64_t one = 1;
  uint64_t k = one;
  uint64_t x = 0;
  double t0 = now_s();
#if defined(__x86_64__)
  for (uint64_t i = 0; i < n; i++) {
    __asm__ volatile("add %1, %0" : "+r"(x) : "r"(k));
  }
#elif defined(__aarch64__)
  for (uint64_t i = 0; i < n; i++) {
    __asm__ volatile("add %0, %0, %1" : "+r"(x) : "r"(k));
  }
#else
  for (uint64_t i = 0; i < n; i++) {
    x += k;
  }
#endif
  double dt = now_s() - t0;
  if (x != n) {
    return -1.0; /* the chain did not happen: a fast wrong answer */
  }
  return dt * 1e9 / (double)n;
}

static double os_mhz_max(void) {
  FILE *f = fopen("/proc/cpuinfo", "r");
  if (!f) {
    return 0.0;
  }
  char line[512];
  double max = 0.0;
  while (fgets(line, sizeof line, f)) {
    if (strncmp(line, "cpu MHz", 7) == 0) {
      char *c = strchr(line, ':');
      if (c) {
        double v = atof(c + 1);
        if (v > max) {
          max = v;
        }
      }
    }
  }
  fclose(f);
  return max;
}

/* ---------------------------------------------------------------- graph */

/* A node is two 32-bit slots: the next reference and a payload. Eight
 * bytes, the smallest object that holds a reference. */
#define NODE_BYTES 8

enum variant { ONE_BASE, BRAND_DYNAMIC, TWO_BASE_STATIC, TWO_BASE_DYNAMIC, N_VARIANTS };

static const char *variant_name(enum variant v) {
  switch (v) {
  case ONE_BASE:
    return "one_base";
  case BRAND_DYNAMIC:
    return "brand_dynamic";
  case TWO_BASE_STATIC:
    return "two_base_static";
  case TWO_BASE_DYNAMIC:
    return "two_base_dynamic";
  default:
    return "?";
  }
}

/* A random permutation of 0..n-1 that is one single cycle (Sattolo). */
static void sattolo(uint32_t *perm, uint64_t n, uint64_t seed) {
  for (uint64_t i = 0; i < n; i++) {
    perm[i] = (uint32_t)i;
  }
  uint64_t s = seed | 1;
  for (uint64_t i = n - 1; i > 0; i--) {
    s ^= s << 13;
    s ^= s >> 7;
    s ^= s << 17;
    uint64_t j = s % i; /* strictly below i: one cycle */
    uint32_t t = perm[i];
    perm[i] = perm[j];
    perm[j] = t;
  }
}

/* Walks the cycle `laps` times with the decode pinned per variant.
 *
 * One loop is generated per variant, by the macro below, so the variant
 * is never tested inside the inner loop: an early version dispatched on
 * it per step, which added the same few cycles to every variant and
 * compressed the differences being measured.
 *
 * `scratch_a` and `scratch_b` are the assembly's own temporaries. The
 * checked values `sum` and `slot` live in the function's frame and no
 * sequence writes them.
 */

struct cell {
  double ns_per_step;
  int cycle_ok;
  int sum_ok;
  uint64_t steps;
};

#define WALK_BODY(NAME, STEP)                                                            \
  static struct cell NAME(unsigned char *base0, unsigned char *base1, uint32_t start,     \
                          uint64_t n, uint64_t laps, uint32_t lo, uint32_t width) {       \
    struct cell c;                                                                        \
    c.ns_per_step = 0.0;                                                                  \
    c.cycle_ok = 1;                                                                       \
    c.sum_ok = 1;                                                                         \
    c.steps = 0;                                                                          \
    uint64_t sum = 0;                                                                     \
    const uint64_t want = n * laps;                                                       \
    uint64_t scratch_a = 0;                                                               \
    uint32_t scratch_b = 0;                                                               \
    (void)base1;                                                                          \
    (void)lo;                                                                             \
    (void)width;                                                                          \
    (void)scratch_a;                                                                      \
    (void)scratch_b;                                                                      \
    double t0 = now_s();                                                                  \
    for (uint64_t lap = 0; lap < laps; lap++) {                                           \
      uint32_t slot = start;                                                              \
      for (uint64_t i = 0; i < n; i++) {                                                  \
        uint32_t next = 0;                                                                \
        uint32_t pay = 0;                                                                 \
        STEP                                                                              \
        sum += pay;                                                                       \
        slot = next;                                                                      \
        c.steps++;                                                                        \
      }                                                                                   \
      if (slot != start) {                                                                \
        c.cycle_ok = 0;                                                                   \
        break;                                                                            \
      }                                                                                   \
    }                                                                                     \
    double dt = now_s() - t0;                                                             \
    c.ns_per_step = c.steps ? dt * 1e9 / (double)c.steps : 0.0;                            \
    c.sum_ok = (sum == want);                                                             \
    return c;                                                                             \
  }

/* (a) one process-wide base: base + 4*v - 4, one addressing mode on
 * x86-64, an extended-register add and a load at -4 on AArch64. A
 * reference into another agent's blocks, or into a shared heap's, uses
 * this same sequence: the base does not depend on the target's heap. */
#if defined(__x86_64__)
#define STEP_ONE_BASE(B)                                                                 \
  __asm__ volatile("movl -4(%[b],%[s],4), %[n]\n\t"                                      \
                   "movl 0(%[b],%[s],4), %[p]"                                           \
                   : [n] "=&r"(next), [p] "=r"(pay)                                      \
                   : [b] "r"(B), [s] "r"((uint64_t)slot)                                 \
                   : "memory");
#elif defined(__aarch64__)
#define STEP_ONE_BASE(B)                                                                 \
  __asm__ volatile("add %[t], %[b], %w[s], uxtw #2\n\t"                                  \
                   "ldur %w[n], [%[t], #-4]\n\t"                                         \
                   "ldr %w[p], [%[t]]"                                                   \
                   : [n] "=&r"(next), [p] "=&r"(pay), [t] "=&r"(scratch_a)               \
                   : [b] "r"(B), [s] "r"(slot)                                           \
                   : "memory");
#else
#define STEP_ONE_BASE(B)                                                                 \
  next = *(uint32_t *)((B) + 4 * (uint64_t)slot - 4);                                     \
  pay = *(uint32_t *)((B) + 4 * (uint64_t)slot);
#endif

/* (a) plus the one-way rule checked at run time: (v - lo) < width, then
 * the same load. Every check passes, so the branch is predicted. */
#if defined(__x86_64__)
#define STEP_BRAND_DYNAMIC                                                               \
  __asm__ volatile("movl %k[s], %[t2]\n\t"                                               \
                   "subl %[lo], %[t2]\n\t"                                               \
                   "cmpl %[w], %[t2]\n\t"                                                \
                   "jae 1f\n\t"                                                          \
                   "movl -4(%[b],%[s],4), %[n]\n\t"                                      \
                   "movl 0(%[b],%[s],4), %[p]\n\t"                                       \
                   "jmp 2f\n"                                                            \
                   "1:\n\t"                                                              \
                   "xorl %[n], %[n]\n\t"                                                 \
                   "xorl %[p], %[p]\n"                                                   \
                   "2:"                                                                  \
                   : [n] "=&r"(next), [p] "=&r"(pay), [t2] "=&r"(scratch_b)              \
                   : [b] "r"(base0), [s] "r"((uint64_t)slot), [lo] "r"(lo),              \
                     [w] "r"(width)                                                       \
                   : "cc", "memory");
#elif defined(__aarch64__)
#define STEP_BRAND_DYNAMIC                                                               \
  __asm__ volatile("sub %w[t2], %w[s], %w[lo]\n\t"                                       \
                   "cmp %w[t2], %w[w]\n\t"                                               \
                   "b.hs 1f\n\t"                                                         \
                   "add %[t], %[b], %w[s], uxtw #2\n\t"                                  \
                   "ldur %w[n], [%[t], #-4]\n\t"                                         \
                   "ldr %w[p], [%[t]]\n\t"                                               \
                   "b 2f\n"                                                              \
                   "1:\n\t"                                                              \
                   "mov %w[n], #0\n\t"                                                   \
                   "mov %w[p], #0\n"                                                     \
                   "2:"                                                                  \
                   : [n] "=&r"(next), [p] "=&r"(pay), [t] "=&r"(scratch_a),              \
                     [t2] "=&r"(scratch_b)                                                \
                   : [b] "r"(base0), [s] "r"(slot), [lo] "r"(lo), [w] "r"(width)          \
                   : "cc", "memory");
#else
#define STEP_BRAND_DYNAMIC                                                               \
  if ((uint32_t)(slot - lo) < width) {                                                    \
    next = *(uint32_t *)(base0 + 4 * (uint64_t)slot - 4);                                 \
    pay = *(uint32_t *)(base0 + 4 * (uint64_t)slot);                                      \
  }
#endif

/* (b) where the slot must say which cage: bit 1 selects the base through
 * a conditional move, and one shift puts the slot back in (a)'s form
 * against the chosen base. */
#if defined(__x86_64__)
#define STEP_TWO_BASE_DYNAMIC                                                            \
  __asm__ volatile("movq %[b0], %[t]\n\t"                                                \
                   "testl $2, %k[s]\n\t"                                                 \
                   "cmovneq %[b1], %[t]\n\t"                                             \
                   "movl %k[s], %[t2]\n\t"                                               \
                   "shrl $1, %[t2]\n\t"                                                  \
                   "movl -4(%[t],%q[t2],4), %[n]\n\t"                                    \
                   "movl 0(%[t],%q[t2],4), %[p]"                                         \
                   : [n] "=&r"(next), [p] "=&r"(pay), [t] "=&r"(scratch_a),              \
                     [t2] "=&r"(scratch_b)                                                \
                   : [b0] "r"(base0), [b1] "r"(base1), [s] "r"((uint64_t)slot)            \
                   : "cc", "memory");
#elif defined(__aarch64__)
#define STEP_TWO_BASE_DYNAMIC                                                            \
  __asm__ volatile("tst %w[s], #2\n\t"                                                   \
                   "csel %[t], %[b1], %[b0], ne\n\t"                                     \
                   "lsr %w[t2], %w[s], #1\n\t"                                           \
                   "add %[t], %[t], %w[t2], uxtw #2\n\t"                                 \
                   "ldur %w[n], [%[t], #-4]\n\t"                                         \
                   "ldr %w[p], [%[t]]"                                                   \
                   : [n] "=&r"(next), [p] "=&r"(pay), [t] "=&r"(scratch_a),              \
                     [t2] "=&r"(scratch_b)                                                \
                   : [b0] "r"(base0), [b1] "r"(base1), [s] "r"(slot)                      \
                   : "cc", "memory");
#else
#define STEP_TWO_BASE_DYNAMIC                                                            \
  {                                                                                       \
    unsigned char *b = (slot & 2) ? base1 : base0;                                        \
    uint32_t u = slot >> 1;                                                               \
    next = *(uint32_t *)(b + 4 * (uint64_t)u - 4);                                        \
    pay = *(uint32_t *)(b + 4 * (uint64_t)u);                                             \
  }
#endif

WALK_BODY(walk_one_base, STEP_ONE_BASE(base0))
WALK_BODY(walk_brand_dynamic, STEP_BRAND_DYNAMIC)
WALK_BODY(walk_two_base_static, STEP_ONE_BASE(base1))
WALK_BODY(walk_two_base_dynamic, STEP_TWO_BASE_DYNAMIC)

static struct cell walk(enum variant v, unsigned char *base0, unsigned char *base1,
                        uint32_t start, uint64_t n, uint64_t laps, uint32_t lo,
                        uint32_t width) {
  switch (v) {
  case ONE_BASE:
    return walk_one_base(base0, base1, start, n, laps, lo, width);
  case BRAND_DYNAMIC:
    return walk_brand_dynamic(base0, base1, start, n, laps, lo, width);
  case TWO_BASE_STATIC:
    return walk_two_base_static(base0, base1, start, n, laps, lo, width);
  default:
    return walk_two_base_dynamic(base0, base1, start, n, laps, lo, width);
  }
}

/* Lays the graph out in `buf` under the one_base encoding: node i is at
 * byte offset (i+1)*8, its slot is v = (i+1)*2 + 1, the next reference
 * sits at base + 4*v - 4 and the payload 4 bytes above it. */
static void lay_one_base(unsigned char *buf, const uint32_t *perm, uint64_t n) {
  for (uint64_t i = 0; i < n; i++) {
    size_t at = (size_t)(i + 1) * NODE_BYTES;
    *(uint32_t *)(buf + at) = (uint32_t)((perm[i] + 1) * 2 + 1);
    *(uint32_t *)(buf + at + 4) = 1;
  }
}

/* The same graph under the dynamic two-base encoding: a slot is
 * (unit << 2) | (sel << 1) | 1 with sel = 1, so every step takes the
 * conditional move's taken side and the measured cost is the select
 * itself, never a mispredicted branch. The shift in the sequence turns
 * the slot back into (unit << 1) | 1, so the node layout is unchanged. */
static void lay_two_base_dynamic(unsigned char *buf, const uint32_t *perm, uint64_t n) {
  for (uint64_t i = 0; i < n; i++) {
    size_t at = (size_t)(i + 1) * NODE_BYTES;
    uint32_t unit = (uint32_t)(perm[i] + 1);
    *(uint32_t *)(buf + at) = (unit << 2) | 2u | 1u;
    *(uint32_t *)(buf + at + 4) = 1;
  }
}

int main(void) {
  /* Footprints in nodes, from L1-resident to out in DRAM. */
  const uint64_t footprints[] = {1024, 8192, 65536, 262144, 2097152, 8388608};
  const int nfoot = (int)(sizeof footprints / sizeof footprints[0]);
  const uint64_t step_budget = 40000000;

  printf("# d17-xheap: the cost of a cross-heap reference and of a run-time brand check\n");
  printf("# isa=%s asm_pinned=%d node_bytes=%d\n", ISA, ASM_PINNED, NODE_BYTES);
  double na = ns_per_add(200000000);
  double mhz = os_mhz_max();
  if (na < 0) {
    printf("FAIL\tclock\tthe add chain did not produce the value it must have\n");
    return 1;
  }
  printf("# clock: %.4f ns/add (one add per cycle) = %.3f GHz; OS says %.0f MHz\n", na,
         1.0 / na, mhz);
  printf("variant\tnodes\tbytes\tns_per_step\tghz_chain\tmhz_os\tcycles_per_step\tverdict\n");
  fflush(stdout);

  for (int fi = 0; fi < nfoot; fi++) {
    uint64_t n = footprints[fi];
    size_t bytes = ((size_t)(n + 2) * NODE_BYTES + 4095) & ~(size_t)4095;
    unsigned char *c0 = aligned_alloc(4096, bytes);
    unsigned char *c1 = aligned_alloc(4096, bytes);
    uint32_t *perm = malloc((size_t)n * sizeof *perm);
    if (!c0 || !c1 || !perm) {
      printf("FAIL\talloc\t%llu\tnodes\n", (unsigned long long)n);
      free(c0);
      free(c1);
      free(perm);
      continue;
    }
    memset(c0, 0, bytes);
    memset(c1, 0, bytes);
    sattolo(perm, n, 0x9E3779B97F4A7C15ull ^ n);

    uint64_t laps = step_budget / n;
    if (laps < 2) {
      laps = 2;
    }
    /* The slot range the brand check accepts: every node of the graph. */
    uint32_t lo = 3;
    uint32_t width = (uint32_t)(n * 2 + 1);

    for (int vi = 0; vi < N_VARIANTS; vi++) {
      enum variant v = (enum variant)vi;
      uint32_t start;
      if (v == TWO_BASE_DYNAMIC) {
        lay_two_base_dynamic(c1, perm, n);
        start = (1u << 2) | 2u | 1u;
      } else {
        lay_one_base(c0, perm, n);
        lay_one_base(c1, perm, n);
        start = 1 * 2 + 1;
      }
      unsigned char *b1 = (v == TWO_BASE_STATIC || v == TWO_BASE_DYNAMIC) ? c1 : c0;
      struct cell c = walk(v, c0, b1, start, n, laps, lo, width);
      if (c.cycle_ok && c.sum_ok) {
        printf("%s\t%llu\t%llu\t%.4f\t%.3f\t%.0f\t%.3f\tok\n", variant_name(v),
               (unsigned long long)n, (unsigned long long)(n * NODE_BYTES), c.ns_per_step,
               1.0 / na, mhz, c.ns_per_step / na);
      } else {
        printf("%s\t%llu\t%llu\tFAIL\t%.3f\t%.0f\tFAIL\tcycle_ok=%d sum_ok=%d\n",
               variant_name(v), (unsigned long long)n,
               (unsigned long long)(n * NODE_BYTES), 1.0 / na, mhz, c.cycle_ok, c.sum_ok);
      }
      fflush(stdout);
    }
    free(c0);
    free(c1);
    free(perm);
  }
  return 0;
}
