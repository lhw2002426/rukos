#include <stdio.h>
#include <stdlib.h>
#include <pthread.h>
#include <unistd.h>

#define STACK_SIZE 1024 * 1024  // 定义栈大小为 1 MB

// 线程函数
void* thread_function(void* arg) {
    printf("Thread is running with custom stack.\n");
    sleep(1);  // 模拟一些工作
    printf("Thread has finished work.\n");
    return NULL;
}

int main() {
    pthread_t thread;
    pthread_attr_t attr;
    void* stack;

    // 为线程栈分配内存
    stack = malloc(STACK_SIZE);
    if (stack == NULL) {
        perror("Failed to allocate stack");
        return 1;
    }

    // 初始化线程属性
    pthread_attr_init(&attr);

    // 设置线程栈
    pthread_attr_setstack(&attr, stack, STACK_SIZE);

    // 创建线程，使用自定义栈
    int result = pthread_create(&thread, &attr, thread_function, NULL);
    if (result != 0) {
        perror("Failed to create thread");
        free(stack);  // 创建线程失败时，释放栈内存
        return 1;
    }

    // 等待线程完成
    sleep(2);

    // 销毁线程属性并释放栈内存
    pthread_attr_destroy(&attr);
    free(stack);

    printf("Main thread has finished.\n");
    return 0;
}
