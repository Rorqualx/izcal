/*
 * SEC-D6 probe: do Seatbelt (SBPL) profiles applied with sandbox_init /
 * sandbox_init_with_parameters still apply on macOS 27, and does the
 * Mach-O SDK version (LC_BUILD_VERSION sdk field) change the outcome?
 *
 * Written from Apple's sandbox_init(3), sandbox-exec(1) and sandbox.h only,
 * plus black-box tests. No other browser's profile files were consulted.
 *
 * Usage: probe <mode> <allowed_file> <denied_file>
 *   baseline       no sandbox; run all checks (expected: everything allowed)
 *   custom-init    sandbox_init(custom deny-default profile, 0, &err)
 *   custom-params  sandbox_init_with_parameters(profile using (param ...), 0, params, &err)
 *   named-pure     sandbox_init(kSBXProfilePureComputation, SANDBOX_NAMED, &err)
 *   named-nonet    sandbox_init(kSBXProfileNoNetwork, SANDBOX_NAMED, &err)
 *   mach           custom deny-default profile that allows mach-lookup of one service
 *   checks         no sandbox_init; run checks only (used under sandbox-exec)
 *
 * Every line is prefixed "RESULT" so the driver can grep. Exit 0 means the
 * probe ran to the end; a signal exit means the process was killed.
 */
#include <errno.h>
#include <fcntl.h>
#include <mach/mach.h>
#include <servers/bootstrap.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <unistd.h>
#include <mach-o/dyld.h>

#pragma clang diagnostic ignored "-Wdeprecated-declarations"
#include <sandbox.h>

/* Not in the public header; exported by libsystem_sandbox. Signature from
 * black-box use: parameters is a NULL-terminated key, value, key, value list. */
extern int sandbox_init_with_parameters(const char *profile, uint64_t flags,
                                        const char *const parameters[],
                                        char **errorbuf);
/* Private; sandbox_check(pid, NULL, 0) returns 1 if pid is sandboxed. */
extern int sandbox_check(pid_t pid, const char *operation, int type, ...);
/* dyld SPI: the SDK version dyld read from LC_BUILD_VERSION. */
extern uint32_t dyld_get_program_sdk_version(void);
extern uint32_t dyld_get_program_min_os_version(void);

#define ALLOWED_SERVICE "com.apple.system.notification_center"
#define DENIED_SERVICE "com.apple.cfprefsd.daemon"

static void vers(const char *what, uint32_t v) {
  printf("RESULT %s=%u.%u.%u\n", what, v >> 16, (v >> 8) & 0xff, v & 0xff);
}

static void check_read(const char *label, const char *path) {
  int fd = open(path, O_RDONLY);
  if (fd < 0) {
    printf("RESULT read_%s=FAIL errno=%d (%s)\n", label, errno, strerror(errno));
    return;
  }
  char buf[64];
  ssize_t n = read(fd, buf, sizeof buf);
  int e = errno;
  close(fd);
  if (n < 0)
    printf("RESULT read_%s=FAIL(read) errno=%d (%s)\n", label, e, strerror(e));
  else
    printf("RESULT read_%s=OK bytes=%zd\n", label, n);
}

static void check_write(const char *dir) {
  char p[1024];
  snprintf(p, sizeof p, "%s/probe-write-%d.tmp", dir, getpid());
  int fd = open(p, O_WRONLY | O_CREAT | O_TRUNC, 0600);
  if (fd < 0) {
    printf("RESULT write_tmp=FAIL errno=%d (%s)\n", errno, strerror(errno));
    return;
  }
  close(fd);
  unlink(p);
  printf("RESULT write_tmp=OK\n");
}

static void check_connect(int port) {
  int s = socket(AF_INET, SOCK_STREAM, 0);
  if (s < 0) {
    printf("RESULT net_socket=FAIL errno=%d (%s)\n", errno, strerror(errno));
    return;
  }
  struct sockaddr_in a = {0};
  a.sin_len = sizeof a;
  a.sin_family = AF_INET;
  a.sin_port = htons((uint16_t)port);
  a.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
  if (connect(s, (struct sockaddr *)&a, sizeof a) < 0)
    printf("RESULT net_connect=FAIL errno=%d (%s)\n", errno, strerror(errno));
  else
    printf("RESULT net_connect=OK\n");
  close(s);
}

static void check_mach(const char *label, const char *svc) {
  mach_port_t p = MACH_PORT_NULL;
  kern_return_t kr = bootstrap_look_up(bootstrap_port, svc, &p);
  if (kr == KERN_SUCCESS && MACH_PORT_VALID(p))
    printf("RESULT mach_%s=OK (%s)\n", label, svc);
  else
    printf("RESULT mach_%s=FAIL kr=0x%x (%s) (%s)\n", label, kr,
           bootstrap_strerror(kr), svc);
  if (MACH_PORT_VALID(p)) mach_port_deallocate(mach_task_self(), p);
}

static void compute(void) {
  volatile uint64_t x = 1;
  for (int i = 0; i < 1000000; i++) x = x * 6364136223846793005ULL + 1442695040888963407ULL;
  printf("RESULT compute=OK x=%llu\n", (unsigned long long)x);
}

/* Listener created before the sandbox so a sandboxed connect has a peer:
 * EPERM then means policy, ECONNREFUSED would mean no peer. */
static int make_listener(int *port) {
  int s = socket(AF_INET, SOCK_STREAM, 0);
  struct sockaddr_in a = {0};
  a.sin_len = sizeof a;
  a.sin_family = AF_INET;
  a.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
  if (s < 0 || bind(s, (struct sockaddr *)&a, sizeof a) < 0 || listen(s, 8) < 0) {
    perror("listener");
    exit(3);
  }
  socklen_t l = sizeof a;
  getsockname(s, (struct sockaddr *)&a, &l);
  *port = ntohs(a.sin_port);
  return s;
}

static void run_checks(const char *allowed, const char *denied, const char *tmpdir, int port) {
  printf("RESULT sandboxed=%d\n", sandbox_check(getpid(), NULL, 0));
  check_read("allowed", allowed);
  check_read("denied", denied);
  check_write(tmpdir);
  check_connect(port);
  check_mach("allowed_svc", ALLOWED_SERVICE);
  check_mach("denied_svc", DENIED_SERVICE);
  compute();
}

static void report_init(const char *api, int rc, char *err) {
  printf("RESULT %s_rc=%d err=%s\n", api, rc, err ? err : "(null)");
  if (err) sandbox_free_error(err);
  fflush(stdout);
}

int main(int argc, char **argv) {
  setvbuf(stdout, NULL, _IOLBF, 0);
  if (argc < 4) {
    fprintf(stderr, "usage: %s <mode> <allowed_file> <denied_file> [tmpdir]\n", argv[0]);
    return 2;
  }
  const char *mode = argv[1], *allowed = argv[2], *denied = argv[3];
  const char *tmpdir = argc > 4 ? argv[4] : "/private/tmp";
  printf("RESULT mode=%s pid=%d\n", mode, getpid());
  vers("dyld_sdk", dyld_get_program_sdk_version());
  vers("dyld_minos", dyld_get_program_min_os_version());

  int port;
  int lfd = make_listener(&port);
  (void)lfd;

  char profile[4096];
  char *err = NULL;
  int rc;

  if (!strcmp(mode, "baseline") || !strcmp(mode, "checks")) {
    /* nothing */
  } else if (!strcmp(mode, "custom-init")) {
    /* Deny everything; allow reading one literal file. The process already
     * has its libraries mapped, so no further file access is needed. */
    snprintf(profile, sizeof profile,
             "(version 1)\n"
             "(deny default)\n"
             "(allow file-read* (literal \"%s\"))\n",
             allowed);
    rc = sandbox_init(profile, 0, &err);
    report_init("sandbox_init", rc, err);
  } else if (!strcmp(mode, "custom-params")) {
    const char *profile_p =
        "(version 1)\n"
        "(deny default)\n"
        "(allow file-read* (literal (param \"ALLOWED_FILE\")))\n";
    const char *params[] = {"ALLOWED_FILE", allowed, NULL};
    rc = sandbox_init_with_parameters(profile_p, 0, params, &err);
    report_init("sandbox_init_with_parameters", rc, err);
  } else if (!strcmp(mode, "named-pure")) {
    rc = sandbox_init(kSBXProfilePureComputation, SANDBOX_NAMED, &err);
    report_init("sandbox_init_named_pure", rc, err);
  } else if (!strcmp(mode, "named-nonet")) {
    rc = sandbox_init(kSBXProfileNoNetwork, SANDBOX_NAMED, &err);
    report_init("sandbox_init_named_nonet", rc, err);
  } else if (!strcmp(mode, "mach")) {
    snprintf(profile, sizeof profile,
             "(version 1)\n"
             "(deny default)\n"
             "(allow file-read* (literal \"%s\"))\n"
             "(allow mach-lookup (global-name \"" ALLOWED_SERVICE "\"))\n",
             allowed);
    rc = sandbox_init(profile, 0, &err);
    report_init("sandbox_init_mach", rc, err);
  } else {
    fprintf(stderr, "unknown mode %s\n", mode);
    return 2;
  }
  run_checks(allowed, denied, tmpdir, port);
  printf("RESULT reached_end=1\n");
  return 0;
}
