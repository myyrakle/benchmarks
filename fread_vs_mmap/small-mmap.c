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
    FILE *file = fopen("small.txt", "r");
    fseek(file, 0, SEEK_END);
    long filesize = ftell(file);
    rewind(file);

    int count = 5000;

    char *result = (char *)malloc(filesize * count * 2);
    result[0] = '\0';
    char *buffer = (char *)malloc(filesize);

    clock_t start = clock();

    for (int i = 0; i < count; i++)
    {
        int file_descriptor = open("small.txt", O_RDONLY);
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

        // append buffer to result
        strncat(result, buffer, filesize);
    }

    clock_t end = clock();

    // write result to file
    FILE *output = fopen("output.txt", "w");
    fwrite(result, 1, strlen(result), output);

    double cpu_time_used = ((double)(end - start)) / CLOCKS_PER_SEC;

    printf("Execution time: %f seconds\n", cpu_time_used);

    return 0;
}
