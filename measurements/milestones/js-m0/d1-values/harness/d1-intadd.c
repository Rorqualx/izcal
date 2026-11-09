// JS-D1's other per-ISA cost: what the small-integer overflow check
// costs where there is no flags register.
//
// With tag 0 and 31 payload bits, two tagged small integers add with one
// machine add, and the 32-bit overflow flag decides whether the result
// is still a small integer (Section 6.1's derivation table). riscv64 has
// no flags register, so there the add is followed by an explicit check:
// a 32-bit add and a 64-bit add compared, about three instructions. The
// spec calls that a per-ISA cost, to be measured at this spike.
//
// Two sequences, the same arithmetic:
//
//   flags     one add, then a branch on the overflow flag.
//             x86-64: `add` + `jo`                        (2 instr)
//             AArch64: `adds` + `b.vs`                    (2 instr)
//             riscv64: cannot be written; the row is absent there
//
//   explicit  a 32-bit add and a 64-bit add, compared and branched.
//             riscv64: `addw` + `add` + `bne`             (3 instr)
//               `addw` sign-extends its result and `bne` compares and
//               branches in one instruction, so riscv64 needs three.
//             AArch64: `add w` + `sxtw` + `add x` + `cmp` + `b.ne` (5)
//             x86-64: `mov` + `addl` + `movslq` + `lea` + `cmp` + `jne` (6)
//               Neither gets the sign extension free and neither has a
//               compare-and-branch, so BOTH OVERSTATE what riscv64
//               pays. What the rows on these two machines measure is the
//               SHAPE of the cost — a dependent ALU op and a compare in
//               place of a flag branch — and an upper bound on it. The
//               riscv64 figure itself has to be taken on a riscv64 core
//               (the K3 board, when it arrives).
//
// Both shapes of load are measured, because a tagged add in an
// interpreter sits on a dependency chain while a compiled loop's adds
// pipeline:
//   latency     each add consumes the previous result
//   throughput  four independent accumulators
//
// No add overflows during a run: the taken branch belongs to the slow
// path, not to the common one measured here. Each row is checked
// afterwards against the value the arithmetic must have produced, so a
// loop that exited early (which would look fast) is reported as FAIL
// rather than printed as a measurement.
//
// Build: cc -O2 -o d1-intadd d1-intadd.c
#define _POSIX_C_SOURCE 200809L
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#if defined(__x86_64__)
#define ISA "x86-64"
#define HAVE_FLAGS 1
#define N_FLAGS 2
#define N_EXPLICIT 6
#elif defined(__aarch64__)
#define ISA "aarch64"
#define HAVE_FLAGS 1
#define N_FLAGS 2
#define N_EXPLICIT 5
#elif defined(__riscv) && __riscv_xlen == 64
#define ISA "riscv64"
#define HAVE_FLAGS 0
#define N_FLAGS 0
#define N_EXPLICIT 3
#else
#error "JS-D1's spike covers x86-64, AArch64 and riscv64 only"
#endif

// The addend, read through volatile so the compiler cannot fold it into
// the accumulator's register: with both known to be 2 it allocated one
// register, turning the chain into a doubling that overflowed in 31
// steps and left the loop early.
static volatile uint32_t addend = 2;

static double now_ns(void) {
    struct timespec t;
    clock_gettime(CLOCK_MONOTONIC, &t);
    return (double)t.tv_sec * 1e9 + (double)t.tv_nsec;
}

__attribute__((noinline)) static double ns_per_cycle(void) {
    uint64_t iters = 1u << 26, c = iters, x = 1;
    double t0 = now_ns();
#if defined(__x86_64__)
    __asm__ volatile("1:\n\taddq $1, %[x]\n\tdecq %[c]\n\tjnz 1b\n\t"
                     : [x] "+r"(x), [c] "+r"(c)
                     :
                     : "cc");
#elif defined(__aarch64__)
    __asm__ volatile("1:\n\tadd %[x], %[x], #1\n\tsubs %[c], %[c], #1\n\tb.ne 1b\n\t"
                     : [x] "+r"(x), [c] "+r"(c)
                     :
                     : "cc");
#else
    __asm__ volatile("1:\n\taddi %[x], %[x], 1\n\taddi %[c], %[c], -1\n\tbnez %[c], 1b\n\t"
                     : [x] "+r"(x), [c] "+r"(c)
                     :
                     :);
#endif
    double t1 = now_ns();
    return (t1 - t0) / (double)iters + (x == 0 ? 1.0 : 0.0);
}

// ----------------------------------------------------------- sequences
// Each returns the accumulated value, which main checks.

#if HAVE_FLAGS
__attribute__((noinline)) static uint64_t lat_flags(uint64_t iters, uint32_t b) {
    uint64_t c = iters;
    uint32_t v = 0;
#if defined(__x86_64__)
    __asm__ volatile("1:\n\t"
                     "addl %k[b], %k[v]\n\t"
                     "jo 2f\n\t"
                     "decq %[c]\n\t"
                     "jnz 1b\n\t"
                     "2:\n\t"
                     : [v] "+r"(v), [c] "+r"(c)
                     : [b] "r"(b)
                     : "cc");
#else
    __asm__ volatile("1:\n\t"
                     "adds %w[v], %w[v], %w[b]\n\t"
                     "b.vs 2f\n\t"
                     "subs %[c], %[c], #1\n\t"
                     "b.ne 1b\n\t"
                     "2:\n\t"
                     : [v] "+r"(v), [c] "+r"(c)
                     : [b] "r"(b)
                     : "cc");
#endif
    return v;
}

__attribute__((noinline)) static uint64_t tput_flags(uint64_t iters, uint32_t b) {
    uint64_t c = iters;
    uint32_t p = 0, q = 0, r = 0, s = 0;
#if defined(__x86_64__)
    __asm__ volatile("1:\n\t"
                     "addl %k[b], %k[p]\n\tjo 2f\n\t"
                     "addl %k[b], %k[q]\n\tjo 2f\n\t"
                     "addl %k[b], %k[r]\n\tjo 2f\n\t"
                     "addl %k[b], %k[s]\n\tjo 2f\n\t"
                     "decq %[c]\n\tjnz 1b\n\t"
                     "2:\n\t"
                     : [p] "+r"(p), [q] "+r"(q), [r] "+r"(r), [s] "+r"(s), [c] "+r"(c)
                     : [b] "r"(b)
                     : "cc");
#else
    __asm__ volatile("1:\n\t"
                     "adds %w[p], %w[p], %w[b]\n\tb.vs 2f\n\t"
                     "adds %w[q], %w[q], %w[b]\n\tb.vs 2f\n\t"
                     "adds %w[r], %w[r], %w[b]\n\tb.vs 2f\n\t"
                     "adds %w[s], %w[s], %w[b]\n\tb.vs 2f\n\t"
                     "subs %[c], %[c], #1\n\tb.ne 1b\n\t"
                     "2:\n\t"
                     : [p] "+r"(p), [q] "+r"(q), [r] "+r"(r), [s] "+r"(s), [c] "+r"(c)
                     : [b] "r"(b)
                     : "cc");
#endif
    // the four accumulators must agree; main checks one of them
    return (p == q && q == r && r == s) ? p : 0;
}
#endif  // HAVE_FLAGS

__attribute__((noinline)) static uint64_t lat_explicit(uint64_t iters, uint32_t b) {
    uint64_t c = iters, w = 0, bb = (uint64_t)(int32_t)b, t;
#if defined(__x86_64__)
    __asm__ volatile("1:\n\t"
                     "movl %k[w], %k[t]\n\t"
                     "addl %k[bb], %k[t]\n\t"
                     "movslq %k[t], %[t]\n\t"
                     "leaq (%[w],%[bb]), %[w]\n\t"
                     "cmpq %[t], %[w]\n\t"
                     "jne 2f\n\t"
                     "decq %[c]\n\t"
                     "jnz 1b\n\t"
                     "2:\n\t"
                     : [w] "+r"(w), [c] "+r"(c), [t] "=&r"(t)
                     : [bb] "r"(bb)
                     : "cc");
#elif defined(__aarch64__)
    __asm__ volatile("1:\n\t"
                     "add %w[t], %w[w], %w[bb]\n\t"
                     "sxtw %[t], %w[t]\n\t"
                     "add %[w], %[w], %[bb]\n\t"
                     "cmp %[t], %[w]\n\t"
                     "b.ne 2f\n\t"
                     "subs %[c], %[c], #1\n\t"
                     "b.ne 1b\n\t"
                     "2:\n\t"
                     : [w] "+r"(w), [c] "+r"(c), [t] "=&r"(t)
                     : [bb] "r"(bb)
                     : "cc");
#else
    __asm__ volatile("1:\n\t"
                     "addw %[t], %[w], %[bb]\n\t"
                     "add %[w], %[w], %[bb]\n\t"
                     "bne %[t], %[w], 2f\n\t"
                     "addi %[c], %[c], -1\n\t"
                     "bnez %[c], 1b\n\t"
                     "2:\n\t"
                     : [w] "+r"(w), [c] "+r"(c), [t] "=&r"(t)
                     : [bb] "r"(bb)
                     :);
#endif
    return w;
}

__attribute__((noinline)) static uint64_t tput_explicit(uint64_t iters, uint32_t b) {
    uint64_t c = iters, bb = (uint64_t)(int32_t)b;
    uint64_t p = 0, q = 0, r = 0, s = 0, t0, t1, t2, t3;
#if defined(__x86_64__)
    __asm__ volatile("1:\n\t"
                     "movl %k[p], %k[t0]\n\taddl %k[bb], %k[t0]\n\tmovslq %k[t0], %[t0]\n\t"
                     "leaq (%[p],%[bb]), %[p]\n\tcmpq %[t0], %[p]\n\tjne 2f\n\t"
                     "movl %k[q], %k[t1]\n\taddl %k[bb], %k[t1]\n\tmovslq %k[t1], %[t1]\n\t"
                     "leaq (%[q],%[bb]), %[q]\n\tcmpq %[t1], %[q]\n\tjne 2f\n\t"
                     "movl %k[r], %k[t2]\n\taddl %k[bb], %k[t2]\n\tmovslq %k[t2], %[t2]\n\t"
                     "leaq (%[r],%[bb]), %[r]\n\tcmpq %[t2], %[r]\n\tjne 2f\n\t"
                     "movl %k[s], %k[t3]\n\taddl %k[bb], %k[t3]\n\tmovslq %k[t3], %[t3]\n\t"
                     "leaq (%[s],%[bb]), %[s]\n\tcmpq %[t3], %[s]\n\tjne 2f\n\t"
                     "decq %[c]\n\tjnz 1b\n\t"
                     "2:\n\t"
                     : [p] "+r"(p), [q] "+r"(q), [r] "+r"(r), [s] "+r"(s), [c] "+r"(c),
                       [t0] "=&r"(t0), [t1] "=&r"(t1), [t2] "=&r"(t2), [t3] "=&r"(t3)
                     : [bb] "r"(bb)
                     : "cc");
#elif defined(__aarch64__)
    __asm__ volatile("1:\n\t"
                     "add %w[t0], %w[p], %w[bb]\n\tsxtw %[t0], %w[t0]\n\t"
                     "add %w[t1], %w[q], %w[bb]\n\tsxtw %[t1], %w[t1]\n\t"
                     "add %w[t2], %w[r], %w[bb]\n\tsxtw %[t2], %w[t2]\n\t"
                     "add %w[t3], %w[s], %w[bb]\n\tsxtw %[t3], %w[t3]\n\t"
                     "add %[p], %[p], %[bb]\n\tcmp %[t0], %[p]\n\tb.ne 2f\n\t"
                     "add %[q], %[q], %[bb]\n\tcmp %[t1], %[q]\n\tb.ne 2f\n\t"
                     "add %[r], %[r], %[bb]\n\tcmp %[t2], %[r]\n\tb.ne 2f\n\t"
                     "add %[s], %[s], %[bb]\n\tcmp %[t3], %[s]\n\tb.ne 2f\n\t"
                     "subs %[c], %[c], #1\n\tb.ne 1b\n\t"
                     "2:\n\t"
                     : [p] "+r"(p), [q] "+r"(q), [r] "+r"(r), [s] "+r"(s), [c] "+r"(c),
                       [t0] "=&r"(t0), [t1] "=&r"(t1), [t2] "=&r"(t2), [t3] "=&r"(t3)
                     : [bb] "r"(bb)
                     : "cc");
#else
    __asm__ volatile("1:\n\t"
                     "addw %[t0], %[p], %[bb]\n\tadd %[p], %[p], %[bb]\n\t"
                     "bne %[t0], %[p], 2f\n\t"
                     "addw %[t1], %[q], %[bb]\n\tadd %[q], %[q], %[bb]\n\t"
                     "bne %[t1], %[q], 2f\n\t"
                     "addw %[t2], %[r], %[bb]\n\tadd %[r], %[r], %[bb]\n\t"
                     "bne %[t2], %[r], 2f\n\t"
                     "addw %[t3], %[s], %[bb]\n\tadd %[s], %[s], %[bb]\n\t"
                     "bne %[t3], %[s], 2f\n\t"
                     "addi %[c], %[c], -1\n\tbnez %[c], 1b\n\t"
                     "2:\n\t"
                     : [p] "+r"(p), [q] "+r"(q), [r] "+r"(r), [s] "+r"(s), [c] "+r"(c),
                       [t0] "=&r"(t0), [t1] "=&r"(t1), [t2] "=&r"(t2), [t3] "=&r"(t3)
                     : [bb] "r"(bb)
                     :);
#endif
    return (p == q && q == r && r == s) ? p : 0;
}

int main(int argc, char **argv) {
    uint64_t iters = 1u << 24, reps = 16;
    for (int i = 1; i < argc; i++) {
        if (!strcmp(argv[i], "--iters") && i + 1 < argc) {
            iters = strtoull(argv[++i], NULL, 10);
        } else if (!strcmp(argv[i], "--reps") && i + 1 < argc) {
            reps = strtoull(argv[++i], NULL, 10);
        } else {
            fprintf(stderr, "usage: %s [--iters N] [--reps R]\n", argv[0]);
            return 2;
        }
    }
    const uint32_t b = addend;  // 2: the tagged form of 1
    // The accumulator must stay inside 31 bits for the whole run, or an
    // add really would overflow and the slow-path branch would be taken.
    if ((double)iters * b >= 2147483648.0) {
        fprintf(stderr, "--iters %" PRIu64 " would overflow 31 bits; use fewer\n", iters);
        return 2;
    }
    printf("# isa=%s iters=%" PRIu64 " reps=%" PRIu64 " addend=%u\n", ISA, iters, reps, b);
    printf("shape\tsequence\tinstr\tns_per_add\tcycles_per_add\tmin_ns\tcell_mhz\tcheck\n");

    enum { LAT_FLAGS, TPUT_FLAGS, LAT_EXPL, TPUT_EXPL, NROWS };
    static const char *shape[] = {"latency", "throughput", "latency", "throughput"};
    static const char *seq[] = {"flags", "flags", "explicit", "explicit"};
    int failures = 0;
    for (int k = 0; k < NROWS; k++) {
#if !HAVE_FLAGS
        if (k == LAT_FLAGS || k == TPUT_FLAGS) {
            continue;  // no flags register on this ISA
        }
#endif
        int tput = (k == TPUT_FLAGS || k == TPUT_EXPL);
        unsigned instr = (k == LAT_FLAGS || k == TPUT_FLAGS) ? N_FLAGS : N_EXPLICIT;
        uint64_t it = tput ? iters / 4 : iters;
        uint64_t want = (uint64_t)it * b;  // every accumulator's end value
        double sum = 0, best = 1e30;
        int ok = 1;
        for (uint64_t r = 0; r < reps; r++) {
            double t0 = now_ns();
            uint64_t got;
            switch (k) {
#if HAVE_FLAGS
                case LAT_FLAGS:
                    got = lat_flags(it, b);
                    break;
                case TPUT_FLAGS:
                    got = tput_flags(it, b);
                    break;
#endif
                case LAT_EXPL:
                    got = lat_explicit(it, b);
                    break;
                default:
                    got = tput_explicit(it, b);
                    break;
            }
            double t1 = now_ns();
            if (got != want) {
                fprintf(stderr, "FAIL %s %s: got %" PRIu64 ", want %" PRIu64 "\n", shape[k], seq[k],
                        got, want);
                ok = 0;
                break;
            }
            double ns = (t1 - t0) / (double)(tput ? it * 4 : it);
            sum += ns;
            if (ns < best) {
                best = ns;
            }
        }
        if (!ok) {
            failures++;
            continue;
        }
        double nsc = 1e30;
        for (int i = 0; i < 3; i++) {
            double v = ns_per_cycle();
            if (v < nsc) {
                nsc = v;
            }
        }
        double mean = sum / (double)reps;
        printf("%s\t%s\t%u\t%.4f\t%.3f\t%.4f\t%.0f\tok\n", shape[k], seq[k], instr, mean,
               mean / nsc, best, 1000.0 / nsc);
        fflush(stdout);
    }
    return failures ? 1 : 0;
}
