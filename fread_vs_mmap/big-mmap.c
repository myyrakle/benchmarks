#include <stdio.h>
#include <time.h>
#include <stdlib.h>
#include <string.h>

#include <sys/mman.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>

int main()
{
    // 파일 크기를 얻습니다.
    FILE *file = fopen("measurements.txt", "r");
    fseek(file, 0, SEEK_END);
    long filesize = ftell(file);
    rewind(file);

    clock_t start = clock();

    int file_descriptor = open("measurements.txt", O_RDONLY);
    if (file_descriptor == -1)
    {
        perror("Error opening file for reading");
        exit(EXIT_FAILURE);
    }

    char *buffer = mmap(0, filesize, PROT_READ, MAP_SHARED, file_descriptor, 0);
    if (buffer == MAP_FAILED)
    {
        close(file_descriptor);
        perror("Error mmapping the file");
        exit(EXIT_FAILURE);
    }

    printf("Len %ld\n", strlen(buffer));

    printf("Last Character %c\n", buffer[filesize - 1]);

    clock_t end = clock();

    double cpu_time_used = ((double)(end - start)) / CLOCKS_PER_SEC;

    printf("Execution time: %f seconds\n", cpu_time_used);

    return 0;
}
