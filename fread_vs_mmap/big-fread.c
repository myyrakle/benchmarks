#include <stdio.h>
#include <time.h>
#include <stdlib.h>
#include <string.h>

int main()
{
    FILE *file = fopen("measurements.txt", "r");

    // 파일 크기를 얻습니다.
    fseek(file, 0, SEEK_END);
    long filesize = ftell(file);
    rewind(file);

    // 메모리를 할당합니다.
    char *buffer = (char *)malloc(filesize);

    clock_t start = clock();

    // 파일의 내용을 읽어옵니다.
    size_t readsize = fread(buffer, 1, filesize, file);
    if (readsize != filesize)
    {
        printf("Failed to read file\n");
        return 1;
    }

    // NULL 종료 문자를 추가합니다.
    buffer[filesize] = '\0';

    fclose(file);

    clock_t end = clock();

    printf("Len %ld\n", strlen(buffer));

    double cpu_time_used = ((double)(end - start)) / CLOCKS_PER_SEC;

    printf("Execution time: %f seconds\n", cpu_time_used);

    return 0;
}
