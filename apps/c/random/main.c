#define _GNU_SOURCE
#include <sys/random.h>
#include <stdio.h>

int main() {
    printf("Using getrandom to fetch random bytes...\n");
    unsigned char buf[16];
    ssize_t n = getrandom(buf, sizeof(buf), 0);
    if (n < 0) {
        perror("getrandom");
        return 1;
    }

    printf("Random bytes:\n");
    for (int i = 0; i < n; ++i) {
        printf("%02x ", buf[i]);
    }
    printf("\n");

    return 0;
}
