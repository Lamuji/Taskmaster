#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>

int main() {
    for(int i = 0; i < 10; i++) {
        printf("This program will fail intentionally.\n");
        sleep(2);
    }
    return 0;
}