// JS-D1's decompression cost: what a dependent load through a compressed
// slot costs against one through a full pointer, on this ISA.
//
// Section 6.1 claims an instruction count per ISA. This measures the
// three encodings of the JS-D1 row, each as the spec writes it, with the
// load sequence pinned in assembly so the compiler cannot turn one
// variant into another:
//
//   scaled32  (a)  slot u32, v = (offset << 1) | 1, offset in 8-byte
//                  units, address = base + 4·v − 4; a 16 GB cage
//   raw64     (b)  slot u64, the address itself; no cage
//   byte32    (c)  slot u32, a byte offset, address = base + v; 4 GB
//
// A node is two slots, the size of the smallest object that holds a
// reference, so a node is 8 bytes under (a) and (c) and 16 under (b):
// the footprint difference compression buys. Slot 0 is `next`.
//
// Two questions need two shapes of run, because they answer differently:
//   --nodes N   the same graph in each encoding. (b) walks twice the
//               bytes, so its cache misses are part of its cost.
//   --bytes B   the same footprint in each encoding, which isolates the
//               decompression instructions from the footprint.
//
// And two shapes of load, because a dependent chain is the worst case:
//   latency     one dependent load per step around a random cycle
//               (Sattolo), so no prefetcher can run ahead and the
//               decompression sits on the critical path. The ceiling.
//   throughput  --chains K walks K independent cycles at once, so the
//               decompression of one load overlaps another's latency.
//               Interpreter and compiled code look more like this.
//
// Output is one TSV row per (mode, encoding, working set): ns and
// estimated cycles per load.
//
// `--check` walks exactly one lap of each encoding and verifies it
// returns to its starting slot, which is what makes the timings mean
// anything: a walk that left the cycle would be fast and wrong.
//
// Build: cc -O2 -o d1-chase d1-chase.c   (no dependencies)
// Run:   ./d1-chase --nodes 1024,16384,1048576 --reps 64
#define _POSIX_C_SOURCE 200809L
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#if defined(__x86_64__)
#define ISA "x86-64"
#elif defined(__aarch64__)
#define ISA "aarch64"
#elif defined(__riscv) && __riscv_xlen == 64
#define ISA "riscv64"
#else
#error "JS-D1's spike covers x86-64, AArch64 and riscv64 only"
#endif

// ---------------------------------------------------------------- timing

static double now_ns(void) {
    struct timespec t;
    clock_gettime(CLOCK_MONOTONIC, &t);
    return (double)t.tv_sec * 1e9 + (double)t.tv_nsec;
}

// --------------------------------------------------------- the clock
//
// A dependent chain of single-cycle adds: one iteration is one cycle on
// every core that issues an add in a cycle, so this measures the clock
// the run actually saw, instead of trusting a nominal figure.

__attribute__((noinline)) static double ns_per_cycle(void) {
    uint64_t iters = 1u << 26, c = iters, x = 1;
#if defined(__x86_64__)
    double t0 = now_ns();
    __asm__ volatile("1:\n\t"
                     "addq $1, %[x]\n\t"
                     "decq %[c]\n\t"
                     "jnz 1b\n\t"
                     : [x] "+r"(x), [c] "+r"(c)
                     :
                     : "cc");
    double t1 = now_ns();
#elif defined(__aarch64__)
    double t0 = now_ns();
    __asm__ volatile("1:\n\t"
                     "add %[x], %[x], #1\n\t"
                     "subs %[c], %[c], #1\n\t"
                     "b.ne 1b\n\t"
                     : [x] "+r"(x), [c] "+r"(c)
                     :
                     : "cc");
    double t1 = now_ns();
#else
    double t0 = now_ns();
    __asm__ volatile("1:\n\t"
                     "addi %[x], %[x], 1\n\t"
                     "addi %[c], %[c], -1\n\t"
                     "bnez %[c], 1b\n\t"
                     : [x] "+r"(x), [c] "+r"(c)
                     :
                     :);
    double t1 = now_ns();
#endif
    if (x == 0) {
        fprintf(stderr, "# impossible\n");
    }
    return (t1 - t0) / (double)iters;
}

// ------------------------------------------------------- the walk loops
//
// Each loop is the encoding's own instruction sequence, nothing else.
// `steps` is a multiple of 1; the chain carries the slot value.

__attribute__((noinline)) static uint64_t chase_scaled32(const unsigned char *base,
                                                         uint32_t v0, uint64_t steps) {
    uint64_t v = v0, c = steps;
#if defined(__x86_64__)
    // one instruction: the scale and the −4 are the addressing mode
    __asm__ volatile("1:\n\t"
                     "movl -4(%[b],%[v],4), %k[v]\n\t"
                     "decq %[c]\n\t"
                     "jnz 1b\n\t"
                     : [v] "+r"(v), [c] "+r"(c)
                     : [b] "r"(base)
                     : "cc");
#elif defined(__aarch64__)
    // two instructions: the extended-register add, then the load at −4
    uint64_t t;
    __asm__ volatile("1:\n\t"
                     "add %[t], %[b], %w[v], uxtw #2\n\t"
                     "ldr %w[v], [%[t], #-4]\n\t"
                     "subs %[c], %[c], #1\n\t"
                     "b.ne 1b\n\t"
                     : [v] "+r"(v), [c] "+r"(c), [t] "=&r"(t)
                     : [b] "r"(base)
                     : "cc");
#else
    // two instructions with Zba: sh2add.uw zero-extends, scales, adds
    uint64_t t;
    __asm__ volatile("1:\n\t"
                     "sh2add.uw %[t], %[v], %[b]\n\t"
                     "lw %[v], -4(%[t])\n\t"
                     "addi %[c], %[c], -1\n\t"
                     "bnez %[c], 1b\n\t"
                     : [v] "+r"(v), [c] "+r"(c), [t] "=&r"(t)
                     : [b] "r"(base)
                     :);
#endif
    return v;
}

__attribute__((noinline)) static uint64_t chase_raw64(const unsigned char *base, uint64_t p0,
                                                      uint64_t steps) {
    (void)base;
    uint64_t v = p0, c = steps;
#if defined(__x86_64__)
    __asm__ volatile("1:\n\t"
                     "movq (%[v]), %[v]\n\t"
                     "decq %[c]\n\t"
                     "jnz 1b\n\t"
                     : [v] "+r"(v), [c] "+r"(c)
                     :
                     : "cc");
#elif defined(__aarch64__)
    __asm__ volatile("1:\n\t"
                     "ldr %[v], [%[v]]\n\t"
                     "subs %[c], %[c], #1\n\t"
                     "b.ne 1b\n\t"
                     : [v] "+r"(v), [c] "+r"(c)
                     :
                     : "cc");
#else
    __asm__ volatile("1:\n\t"
                     "ld %[v], 0(%[v])\n\t"
                     "addi %[c], %[c], -1\n\t"
                     "bnez %[c], 1b\n\t"
                     : [v] "+r"(v), [c] "+r"(c)
                     :
                     :);
#endif
    return v;
}

__attribute__((noinline)) static uint64_t chase_byte32(const unsigned char *base, uint32_t v0,
                                                       uint64_t steps) {
    uint64_t v = v0, c = steps;
#if defined(__x86_64__)
    __asm__ volatile("1:\n\t"
                     "movl (%[b],%[v]), %k[v]\n\t"
                     "decq %[c]\n\t"
                     "jnz 1b\n\t"
                     : [v] "+r"(v), [c] "+r"(c)
                     : [b] "r"(base)
                     : "cc");
#elif defined(__aarch64__)
    // one instruction: an unscaled register-offset load
    __asm__ volatile("1:\n\t"
                     "ldr %w[v], [%[b], %w[v], uxtw]\n\t"
                     "subs %[c], %[c], #1\n\t"
                     "b.ne 1b\n\t"
                     : [v] "+r"(v), [c] "+r"(c)
                     : [b] "r"(base)
                     : "cc");
#else
    // riscv64 has no register-offset load, so (c) is also two
    uint64_t t;
    __asm__ volatile("1:\n\t"
                     "add %[t], %[b], %[v]\n\t"
                     "lw %[v], 0(%[t])\n\t"
                     "addi %[c], %[c], -1\n\t"
                     "bnez %[c], 1b\n\t"
                     : [v] "+r"(v), [c] "+r"(c), [t] "=&r"(t)
                     : [b] "r"(base)
                     :);
#endif
    return v;
}

// --------------------------------------------- four chains at one time
//
// The same cycle, entered at four points a quarter-lap apart, so the
// working set is the latency run's and the four loads of an iteration
// are independent of each other. Returns the loads' xor.

__attribute__((noinline)) static uint64_t tput_scaled32(const unsigned char *base,
                                                        const uint32_t *start, uint64_t iters) {
    uint64_t a = start[0], b2 = start[1], c2 = start[2], d = start[3], c = iters;
#if defined(__x86_64__)
    __asm__ volatile("1:\n\t"
                     "movl -4(%[b],%[a],4), %k[a]\n\t"
                     "movl -4(%[b],%[b2],4), %k[b2]\n\t"
                     "movl -4(%[b],%[c2],4), %k[c2]\n\t"
                     "movl -4(%[b],%[d],4), %k[d]\n\t"
                     "decq %[c]\n\t"
                     "jnz 1b\n\t"
                     : [a] "+r"(a), [b2] "+r"(b2), [c2] "+r"(c2), [d] "+r"(d), [c] "+r"(c)
                     : [b] "r"(base)
                     : "cc");
#elif defined(__aarch64__)
    uint64_t t0, t1, t2, t3;
    __asm__ volatile("1:\n\t"
                     "add %[t0], %[b], %w[a], uxtw #2\n\t"
                     "add %[t1], %[b], %w[b2], uxtw #2\n\t"
                     "add %[t2], %[b], %w[c2], uxtw #2\n\t"
                     "add %[t3], %[b], %w[d], uxtw #2\n\t"
                     "ldr %w[a], [%[t0], #-4]\n\t"
                     "ldr %w[b2], [%[t1], #-4]\n\t"
                     "ldr %w[c2], [%[t2], #-4]\n\t"
                     "ldr %w[d], [%[t3], #-4]\n\t"
                     "subs %[c], %[c], #1\n\t"
                     "b.ne 1b\n\t"
                     : [a] "+r"(a), [b2] "+r"(b2), [c2] "+r"(c2), [d] "+r"(d), [c] "+r"(c),
                       [t0] "=&r"(t0), [t1] "=&r"(t1), [t2] "=&r"(t2), [t3] "=&r"(t3)
                     : [b] "r"(base)
                     : "cc");
#else
    uint64_t t0, t1, t2, t3;
    __asm__ volatile("1:\n\t"
                     "sh2add.uw %[t0], %[a], %[b]\n\t"
                     "sh2add.uw %[t1], %[b2], %[b]\n\t"
                     "sh2add.uw %[t2], %[c2], %[b]\n\t"
                     "sh2add.uw %[t3], %[d], %[b]\n\t"
                     "lw %[a], -4(%[t0])\n\t"
                     "lw %[b2], -4(%[t1])\n\t"
                     "lw %[c2], -4(%[t2])\n\t"
                     "lw %[d], -4(%[t3])\n\t"
                     "addi %[c], %[c], -1\n\t"
                     "bnez %[c], 1b\n\t"
                     : [a] "+r"(a), [b2] "+r"(b2), [c2] "+r"(c2), [d] "+r"(d), [c] "+r"(c),
                       [t0] "=&r"(t0), [t1] "=&r"(t1), [t2] "=&r"(t2), [t3] "=&r"(t3)
                     : [b] "r"(base)
                     :);
#endif
    return a ^ b2 ^ c2 ^ d;
}

__attribute__((noinline)) static uint64_t tput_raw64(const uint64_t *start, uint64_t iters) {
    uint64_t a = start[0], b2 = start[1], c2 = start[2], d = start[3], c = iters;
#if defined(__x86_64__)
    __asm__ volatile("1:\n\t"
                     "movq (%[a]), %[a]\n\t"
                     "movq (%[b2]), %[b2]\n\t"
                     "movq (%[c2]), %[c2]\n\t"
                     "movq (%[d]), %[d]\n\t"
                     "decq %[c]\n\t"
                     "jnz 1b\n\t"
                     : [a] "+r"(a), [b2] "+r"(b2), [c2] "+r"(c2), [d] "+r"(d), [c] "+r"(c)
                     :
                     : "cc");
#elif defined(__aarch64__)
    __asm__ volatile("1:\n\t"
                     "ldr %[a], [%[a]]\n\t"
                     "ldr %[b2], [%[b2]]\n\t"
                     "ldr %[c2], [%[c2]]\n\t"
                     "ldr %[d], [%[d]]\n\t"
                     "subs %[c], %[c], #1\n\t"
                     "b.ne 1b\n\t"
                     : [a] "+r"(a), [b2] "+r"(b2), [c2] "+r"(c2), [d] "+r"(d), [c] "+r"(c)
                     :
                     : "cc");
#else
    __asm__ volatile("1:\n\t"
                     "ld %[a], 0(%[a])\n\t"
                     "ld %[b2], 0(%[b2])\n\t"
                     "ld %[c2], 0(%[c2])\n\t"
                     "ld %[d], 0(%[d])\n\t"
                     "addi %[c], %[c], -1\n\t"
                     "bnez %[c], 1b\n\t"
                     : [a] "+r"(a), [b2] "+r"(b2), [c2] "+r"(c2), [d] "+r"(d), [c] "+r"(c)
                     :
                     :);
#endif
    return a ^ b2 ^ c2 ^ d;
}

__attribute__((noinline)) static uint64_t tput_byte32(const unsigned char *base,
                                                      const uint32_t *start, uint64_t iters) {
    uint64_t a = start[0], b2 = start[1], c2 = start[2], d = start[3], c = iters;
#if defined(__x86_64__)
    __asm__ volatile("1:\n\t"
                     "movl (%[b],%[a]), %k[a]\n\t"
                     "movl (%[b],%[b2]), %k[b2]\n\t"
                     "movl (%[b],%[c2]), %k[c2]\n\t"
                     "movl (%[b],%[d]), %k[d]\n\t"
                     "decq %[c]\n\t"
                     "jnz 1b\n\t"
                     : [a] "+r"(a), [b2] "+r"(b2), [c2] "+r"(c2), [d] "+r"(d), [c] "+r"(c)
                     : [b] "r"(base)
                     : "cc");
#elif defined(__aarch64__)
    __asm__ volatile("1:\n\t"
                     "ldr %w[a], [%[b], %w[a], uxtw]\n\t"
                     "ldr %w[b2], [%[b], %w[b2], uxtw]\n\t"
                     "ldr %w[c2], [%[b], %w[c2], uxtw]\n\t"
                     "ldr %w[d], [%[b], %w[d], uxtw]\n\t"
                     "subs %[c], %[c], #1\n\t"
                     "b.ne 1b\n\t"
                     : [a] "+r"(a), [b2] "+r"(b2), [c2] "+r"(c2), [d] "+r"(d), [c] "+r"(c)
                     : [b] "r"(base)
                     : "cc");
#else
    uint64_t t0, t1, t2, t3;
    __asm__ volatile("1:\n\t"
                     "add %[t0], %[b], %[a]\n\t"
                     "add %[t1], %[b], %[b2]\n\t"
                     "add %[t2], %[b], %[c2]\n\t"
                     "add %[t3], %[b], %[d]\n\t"
                     "lw %[a], 0(%[t0])\n\t"
                     "lw %[b2], 0(%[t1])\n\t"
                     "lw %[c2], 0(%[t2])\n\t"
                     "lw %[d], 0(%[t3])\n\t"
                     "addi %[c], %[c], -1\n\t"
                     "bnez %[c], 1b\n\t"
                     : [a] "+r"(a), [b2] "+r"(b2), [c2] "+r"(c2), [d] "+r"(d), [c] "+r"(c),
                       [t0] "=&r"(t0), [t1] "=&r"(t1), [t2] "=&r"(t2), [t3] "=&r"(t3)
                     : [b] "r"(base)
                     :);
#endif
    return a ^ b2 ^ c2 ^ d;
}

// ------------------------------------------------------------ the graph

// A random cycle over `n` indices: perm[i] is the successor of i.
static uint32_t *cycle(uint64_t n, uint64_t seed) {
    uint32_t *order = malloc(n * sizeof *order);
    uint32_t *next = malloc(n * sizeof *next);
    if (!order || !next) {
        return NULL;
    }
    for (uint64_t i = 0; i < n; i++) {
        order[i] = (uint32_t)i;
    }
    // Sattolo: one cycle through every index
    uint64_t s = seed | 1;
    for (uint64_t i = n - 1; i > 0; i--) {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        uint64_t j = s % i;
        uint32_t t = order[i];
        order[i] = order[j];
        order[j] = t;
    }
    for (uint64_t i = 0; i < n; i++) {
        next[order[i]] = order[(i + 1) % n];
    }
    free(order);
    return next;
}

enum enc { SCALED32, RAW64, BYTE32 };
static const char *enc_name[] = {"scaled32", "raw64", "byte32"};
static const unsigned enc_node[] = {8, 16, 8};  // bytes per node
static const unsigned enc_instr_x86[] = {1, 1, 1};
static const unsigned enc_instr_arm[] = {2, 1, 1};
static const unsigned enc_instr_rv[] = {2, 1, 2};

// The slot value that reaches node `idx` under `e`.
static uint64_t slot_for(enum enc e, const unsigned char *mem, uint64_t idx) {
    if (e == SCALED32) {
        return (idx << 1) | 1;
    }
    if (e == BYTE32) {
        return 8 * idx;
    }
    return (uint64_t)(uintptr_t)(mem + 16 * idx);
}

// One run: build the graph in `enc`, walk it, return ns per load.
// `chains` 1 is the dependent chain, 4 the independent one. With
// `check`, one lap is walked first and must return to its start.
static double run(enum enc e, uint64_t n, uint64_t loads, int chains, int check, double *checksum,
                  const char **err) {
    size_t bytes = (size_t)n * enc_node[e];
    unsigned char *mem = NULL;
    if (posix_memalign((void **)&mem, 4096, bytes) != 0 || !mem) {
        *err = "allocation";
        return -1;
    }
    memset(mem, 0, bytes);
    uint32_t *next = cycle(n, 0x9e3779b97f4a7c15ull ^ n);
    if (!next) {
        free(mem);
        *err = "cycle";
        return -1;
    }
    for (uint64_t i = 0; i < n; i++) {
        uint64_t j = next[i];
        if (e == RAW64) {
            *(uint64_t *)(mem + 16 * i) = (uint64_t)(uintptr_t)(mem + 16 * j);
        } else {
            *(uint32_t *)(mem + 8 * i) = (uint32_t)slot_for(e, mem, j);
        }
    }

    // four entry points a quarter lap apart, by walking the cycle
    uint64_t idx = 0;
    uint32_t start32[4];
    uint64_t start64[4];
    for (int k = 0; k < 4; k++) {
        start32[k] = (uint32_t)slot_for(e, mem, idx);
        start64[k] = slot_for(e, mem, idx);
        for (uint64_t s = 0; s < n / 4; s++) {
            idx = next[idx];
        }
    }
    free(next);

    if (check) {
        // one lap must come back to the slot it started from
        uint64_t got = e == SCALED32  ? chase_scaled32(mem, start32[0], n)
                       : e == BYTE32  ? chase_byte32(mem, start32[0], n)
                                      : chase_raw64(mem, start64[0], n);
        if (got != start64[0]) {
            free(mem);
            *err = "the walk left its cycle";
            return -1;
        }
    }

    uint64_t sink = 0;
    uint64_t iters = chains == 1 ? loads : loads / 4;
    for (int w = 0; w < 2; w++) {  // warm
        uint64_t few = iters > 4096 ? 4096 : iters;
        if (chains == 1) {
            sink ^= e == SCALED32  ? chase_scaled32(mem, start32[0], few)
                    : e == BYTE32  ? chase_byte32(mem, start32[0], few)
                                   : chase_raw64(mem, start64[0], few);
        } else {
            sink ^= e == SCALED32  ? tput_scaled32(mem, start32, few)
                    : e == BYTE32  ? tput_byte32(mem, start32, few)
                                   : tput_raw64(start64, few);
        }
    }
    double t0 = now_ns();
    if (chains == 1) {
        sink ^= e == SCALED32  ? chase_scaled32(mem, start32[0], iters)
                : e == BYTE32  ? chase_byte32(mem, start32[0], iters)
                               : chase_raw64(mem, start64[0], iters);
    } else {
        sink ^= e == SCALED32  ? tput_scaled32(mem, start32, iters)
                : e == BYTE32  ? tput_byte32(mem, start32, iters)
                               : tput_raw64(start64, iters);
    }
    double t1 = now_ns();
    *checksum += (double)(sink & 0xffff);
    free(mem);
    return (t1 - t0) / (double)(chains == 1 ? iters : iters * 4);
}

static uint64_t *parse_list(const char *s, int *count) {
    uint64_t *v = malloc(64 * sizeof *v);
    *count = 0;
    while (*s && *count < 64) {
        v[(*count)++] = strtoull(s, NULL, 10);
        const char *c = strchr(s, ',');
        if (!c) {
            break;
        }
        s = c + 1;
    }
    return v;
}

int main(int argc, char **argv) {
    const char *nodes = "512,2048,8192,65536,524288,4194304";
    const char *bytes = NULL;
    uint64_t reps = 16, min_steps = 1u << 22;
    double mhz = 0;
    for (int i = 1; i < argc; i++) {
        if (!strcmp(argv[i], "--nodes") && i + 1 < argc) {
            nodes = argv[++i];
        } else if (!strcmp(argv[i], "--bytes") && i + 1 < argc) {
            bytes = argv[++i];
        } else if (!strcmp(argv[i], "--reps") && i + 1 < argc) {
            reps = strtoull(argv[++i], NULL, 10);
        } else if (!strcmp(argv[i], "--mhz") && i + 1 < argc) {
            mhz = strtod(argv[++i], NULL);
        } else {
            fprintf(stderr, "usage: %s [--nodes N,N] [--bytes B,B] [--reps R] [--mhz F]\n",
                    argv[0]);
            return 2;
        }
    }
    const unsigned *instr = !strcmp(ISA, "x86-64")
                                ? enc_instr_x86
                                : (!strcmp(ISA, "aarch64") ? enc_instr_arm : enc_instr_rv);

    if (mhz <= 0) {
        // the clock this run saw, from a dependent add chain
        double best = 1e30;
        for (int i = 0; i < 5; i++) {
            double n = ns_per_cycle();
            if (n < best) {
                best = n;
            }
        }
        mhz = 1000.0 / best;
    }
    printf("# isa=%s reps=%" PRIu64 " clock_mhz=%.0f\n", ISA, reps, mhz);
    printf("footprint\tchains\tencoding\tnodes\tbytes\tinstr_per_load\tns_per_load"
           "\tcycles_per_load\tmin_ns\tcell_mhz\n");
    double checksum = 0;
    int failures = 0;

    int nc = 0;
    uint64_t *list = parse_list(bytes ? bytes : nodes, &nc);
    const int chain_counts[2] = {1, 4};
    for (int ci = 0; ci < 2; ci++) {
        int chains = chain_counts[ci];
        for (int k = 0; k < nc; k++) {
            for (int e = 0; e < 3; e++) {
                // --nodes: the same graph. --bytes: the same footprint.
                uint64_t n = bytes ? list[k] / enc_node[e] : list[k];
                if (n < 64) {
                    continue;
                }
                uint64_t loads = min_steps;
                if (loads < n * 4) {
                    loads = n * 4;
                }
                double best = 1e30, sum = 0;
                for (uint64_t r = 0; r < reps; r++) {
                    const char *err = NULL;
                    // check the cycle once per cell, on the first rep
                    double ns = run((enum enc)e, n, loads, chains, r == 0, &checksum, &err);
                    if (ns < 0) {
                        fprintf(stderr, "FAIL %s n=%" PRIu64 " chains=%d: %s\n", enc_name[e], n,
                                chains, err ? err : "?");
                        failures++;
                        break;
                    }
                    if (ns < best) {
                        best = ns;
                    }
                    sum += ns;
                }
                if (best > 1e29) {
                    continue;
                }
                double mean = sum / (double)reps;
                // the clock this cell saw: a governor may move it between
                // cells, and cycles computed from a stale clock would be
                // a comparison between frequencies, not encodings
                double cell_ns_cyc = 1e30;
                for (int i = 0; i < 3; i++) {
                    double v = ns_per_cycle();
                    if (v < cell_ns_cyc) {
                        cell_ns_cyc = v;
                    }
                }
                printf("%s\t%d\t%s\t%" PRIu64 "\t%" PRIu64 "\t%u\t%.4f\t%.3f\t%.4f\t%.0f\n",
                       bytes ? "equal-bytes" : "equal-nodes", chains, enc_name[e], n,
                       n * enc_node[e], instr[e], mean, mean / cell_ns_cyc, best,
                       1000.0 / cell_ns_cyc);
                fflush(stdout);
            }
        }
    }
    fprintf(stderr, "# checksum %.0f failures %d\n", checksum, failures);
    return failures ? 1 : 0;
}
