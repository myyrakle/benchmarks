#include <stdio.h>
#include <time.h>
#include <stdlib.h>
#include <string.h>

int main()
{
    FILE *file = fopen("small.txt", "r");

    // 파일 크기를 얻습니다.
    fseek(file, 0, SEEK_END);
    long filesize = ftell(file);
    rewind(file);

    int count = 5000;

    char *result = (char *)malloc(filesize * count * 2);
    result[0] = '\0';
    char *buffer = (char *)malloc(filesize + 1);

    clock_t start = clock();

    for (int i = 0; i < count; i++)
    {
        file = fopen("small.txt", "r");

        // 파일의 내용을 읽어옵니다.
        size_t readsize = fread(buffer, 1, filesize, file);
        if (readsize != filesize)
        {
            printf("Failed to read file\n");
            return 1;
        }

        // NULL 종료 문자를 추가합니다.
        buffer[filesize] = '\0';

        // append buffer to result
        strncat(result, buffer, filesize + 1);

        fclose(file);
    }

    clock_t end = clock();

    // write result to file
    FILE *output = fopen("output.txt", "w");
    fwrite(result, 1, strlen(result), output);

    double cpu_time_used = ((double)(end - start)) / CLOCKS_PER_SEC;

    printf("Execution time: %f seconds\n", cpu_time_used);

    return 0;
}
