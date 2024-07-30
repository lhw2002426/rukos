#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <pthread.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <errno.h>

#define SOCKET_PATH "/tmp/unix_socket_example"
#define BUFFER_SIZE 256

void* handle_client(void* client_socket) {
    int client_fd = *(int*)client_socket;
    char buffer[BUFFER_SIZE];
    free(client_socket); // Free the allocated memory for client_socket

    // 接收数据
    int bytes_received = recv(client_fd, buffer, BUFFER_SIZE - 1, 0);
    if (bytes_received < 0) {
        perror("recv");
        close(client_fd);
        pthread_exit(NULL);
    }

    buffer[bytes_received] = '\0';
    printf("接收到的数据: %s\n", buffer);

    // 关闭客户端 socket
    close(client_fd);
    pthread_exit(NULL);
}

void start_server() {
    int server_fd, *client_fd;
    struct sockaddr_un server_addr;
    pthread_t thread_id;

    // 创建服务器端 socket
    server_fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (server_fd < 0) {
        perror("socket");
        exit(EXIT_FAILURE);
    }

    // 初始化服务器地址结构
    memset(&server_addr, 0, sizeof(struct sockaddr_un));
    server_addr.sun_family = AF_UNIX;
    strncpy(server_addr.sun_path, SOCKET_PATH, sizeof(server_addr.sun_path) - 1);

    // 如果文件已经存在，先删除
    /*if (unlink(SOCKET_PATH) == -1 && errno != ENOENT) {
        perror("unlink");
        close(server_fd);
        exit(EXIT_FAILURE);
    }*/

    // 绑定 socket 到文件路径
    if (bind(server_fd, (struct sockaddr*)&server_addr, sizeof(struct sockaddr_un)) < 0) {
        perror("bind");
        close(server_fd);
        exit(EXIT_FAILURE);
    }

    // 监听连接
    if (listen(server_fd, 5) < 0) {
        perror("listen");
        close(server_fd);
        exit(EXIT_FAILURE);
    }

    printf("等待客户端连接...\n");

    // 接受客户端连接并创建新线程处理
    while (1) {
        client_fd = malloc(sizeof(int));
        *client_fd = accept(server_fd, NULL, NULL);
        if (*client_fd < 0) {
            perror("accept");
            free(client_fd);
            close(server_fd);
            exit(EXIT_FAILURE);
        }

        printf("客户端连接已建立\n");

        // 创建新线程处理客户端连接
        if (pthread_create(&thread_id, NULL, handle_client, (void*)client_fd) != 0) {
            perror("pthread_create");
            close(*client_fd);
            free(client_fd);
        }

        // 分离线程以防止内存泄漏
        //pthread_detach(thread_id);
    }

    // 关闭服务器端 socket
    close(server_fd);
    unlink(SOCKET_PATH);
}

void start_client() {
    int client_fd;
    struct sockaddr_un server_addr;
    const char* message = "Hello, Unix Socket!";

    // 创建客户端 socket
    client_fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (client_fd < 0) {
        perror("socket");
        exit(EXIT_FAILURE);
    }

    // 初始化服务器地址结构
    memset(&server_addr, 0, sizeof(struct sockaddr_un));
    server_addr.sun_family = AF_UNIX;
    strncpy(server_addr.sun_path, SOCKET_PATH, sizeof(server_addr.sun_path) - 1);

    printf("尝试连接到服务器...\n");

    // 连接到服务器
    if (connect(client_fd, (struct sockaddr*)&server_addr, sizeof(struct sockaddr_un)) < 0) {
        perror("connect");
        close(client_fd);
        exit(EXIT_FAILURE);
    }

    // 发送数据
    if (send(client_fd, message, strlen(message), 0) < 0) {
        perror("send");
        close(client_fd);
        exit(EXIT_FAILURE);
    }

    printf("数据已发送: %s\n", message);

    // 关闭客户端 socket
    close(client_fd);
}

int main() {

    pthread_t server_tid, client_tid;

    if (pthread_create(&server_tid, NULL, start_server, NULL) != 0) {
        perror("Failed to create server thread");
        exit(EXIT_FAILURE);
    }

    if (pthread_create(&client_tid, NULL, start_client, NULL) != 0) {
        perror("Failed to create client thread");
        exit(EXIT_FAILURE);
    }

    pthread_join(server_tid, NULL);
    pthread_join(client_tid, NULL);

    return 0;
}
