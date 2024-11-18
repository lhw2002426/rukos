#define _GNU_SOURCE
#include <sched.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/syscall.h>
#include <dlfcn.h>
#include <unistd.h>
#include <linux/sched.h>
#include <errno.h>
#include <pthread.h>
#include <sys/mman.h>
#include <stdarg.h>

// 定义指向原始的 __pthread_initialize_minimal_internal 的函数指针
void (*real___pthread_initialize_minimal_internal)(void) = NULL;

void __pthread_initialize_minimal_internal(void) {
    if (!real___pthread_initialize_minimal_internal) {
        real___pthread_initialize_minimal_internal = dlsym(RTLD_NEXT, "__pthread_initialize_minimal_internal");
    }
    
    printf("Intercepting __pthread_initialize_minimal_internal call\n");

    // 调用原始的 __pthread_initialize_minimal_internal（可以在此处注释掉来验证不调用的情况）
    real___pthread_initialize_minimal_internal();
}

// 定义指向原始的 pthread_create 的函数指针
int (*real_pthread_create)(pthread_t *, const pthread_attr_t *, void *(*)(void *), void *) = NULL;

// 自定义的 pthread_create 实现
int pthread_create(pthread_t *thread, const pthread_attr_t *attr, void *(*start_routine)(void *), void *arg) {
    if (!real_pthread_create) {
        // 使用 dlsym 获取原始的 pthread_create 地址
        real_pthread_create = dlsym(RTLD_NEXT, "pthread_create");
        if (!real_pthread_create) {
            fprintf(stderr, "Error locating original pthread_create\n");
            return -1;
        }
    }

    printf("Intercepted pthread_create call\n");

    int result = 0;
    // 调用原始的 pthread_create
    result = real_pthread_create(thread, attr, start_routine, arg);

    // 记录调用结果
    if (result == 0) {
        printf("pthread_create successful: Thread ID = %lu\n", *thread);
    } else {
        fprintf(stderr, "pthread_create failed with error code %d\n", result);
    }

    return result;
}

// 定义指向原始的 pthread_join 的函数指针
int (*real_pthread_join)(pthread_t, void **) = NULL;

// 自定义的 pthread_join 实现
int pthread_join(pthread_t thread, void **retval) {
    if (!real_pthread_join) {
        // 使用 dlsym 获取原始的 pthread_join 地址
        real_pthread_join = dlsym(RTLD_NEXT, "pthread_join");
        if (!real_pthread_join) {
            fprintf(stderr, "Error locating original pthread_join\n");
            return -1;
        }
    }

    printf("Intercepted pthread_join call for thread ID = %lu\n", thread);

    // 调用原始的 pthread_join
    int result = real_pthread_join(thread, retval);

    // 记录返回结果
    if (result == 0) {
        printf("pthread_join successful for thread ID = %lu\n", thread);
    } else {
        fprintf(stderr, "pthread_join failed with error code %d for thread ID = %lu\n", result, thread);
    }

    return result;
}

/*
// 使用 mmap 替代 malloc 实现
void* malloc(size_t size) {
    // 使用 mmap 分配内存
    void* ptr = mmap(NULL, size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (ptr == MAP_FAILED) {
        perror("mmap failed in custom malloc");
        return NULL;
    }

    printf("Allocated %zu bytes using mmap at %p\n", size, ptr);
    return ptr;
}

// 自定义的 free 函数，使用 munmap 释放内存
void free(void* ptr) {
    if (ptr == NULL) {
        return;
    }

    // 获取页面大小并释放内存
    size_t page_size = getpagesize();
    if (munmap(ptr, page_size) == -1) {
        perror("munmap failed in custom free");
    } else {
        printf("Freed memory at %p using munmap\n", ptr);
    }
}
*/

// 原始 clone 和 clone3 函数的函数指针
typedef int (*clone_fn)(int (*)(void *), void *, int, void *, ...);
typedef int (*clone3_fn)(struct clone_args *, size_t);

clone_fn real_clone = NULL;
clone3_fn real_clone3 = NULL;

// 拦截 clone 函数
int clone(int (*fn)(void *), void *child_stack, int flags, void *arg, ...) {
    if (!real_clone) {
        // 获取原始的 clone 函数地址
        real_clone = (clone_fn)dlsym(RTLD_NEXT, "clone");
        if (!real_clone) {
            fprintf(stderr, "Error locating original clone\n");
            exit(EXIT_FAILURE);
        }
    }

    // 输出 clone 参数
    printf("Intercepted clone call:\n");
    printf("  fn: %p\n", (void *)fn);
    printf("  child_stack: %p\n", child_stack);
    printf("  flags: 0x%x\n", flags);
    printf("  arg: %p\n", arg);

    // 处理可变参数
    va_list args;
    va_start(args, arg);
    void *ptid = va_arg(args, void*);
    void *ctid = va_arg(args, void*);
    void *tls = va_arg(args, void*);
    va_end(args);

    printf("  ptid: %p\n", ptid);
    printf("  ctid: %p\n", ctid);
    printf("  tls: %p\n", tls);

    // 调用原始的 clone 函数
    return real_clone(fn, child_stack, flags, arg, ptid, ctid, tls);
}

// 拦截 clone3 函数
int clone3(struct clone_args *cl_args, size_t size) {
    if (!real_clone3) {
        // 获取原始的 clone3 函数地址
        real_clone3 = (clone3_fn)dlsym(RTLD_NEXT, "clone3");
        if (!real_clone3) {
            fprintf(stderr, "Error locating original clone3\n");
            exit(EXIT_FAILURE);
        }
    }

    // 输出 clone3 参数
    printf("Intercepted clone3 call:\n");
    if (cl_args) {
        printf("  flags: 0x%llx\n", cl_args->flags);
        printf("  pidfd: %p\n", (void *)cl_args->pidfd);
        printf("  child_tid: %p\n", (void *)cl_args->child_tid);
        printf("  parent_tid: %p\n", (void *)cl_args->parent_tid);
        printf("  exit_signal: %d\n", cl_args->exit_signal);
        printf("  stack: %p\n", cl_args->stack);
        printf("  stack_size: %llu\n", cl_args->stack_size);
        printf("  tls: %p\n", cl_args->tls);
        printf("  set_tid: %p\n", (void *)cl_args->set_tid);
        printf("  set_tid_size: %llu\n", cl_args->set_tid_size);
        printf("  cgroup: %llu\n", cl_args->cgroup);
    } else {
        printf("  clone_args is NULL\n");
    }

    // 调用原始的 clone3 函数
    return real_clone3(cl_args, size);
}