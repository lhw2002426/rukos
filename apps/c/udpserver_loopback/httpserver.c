/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <pthread.h>
#include <arpa/inet.h>
#include <sys/socket.h>

#define PORT 5555
#define BUFFER_SIZE 1024

void *handle_client(void *arg) {
    int new_socket = *(int*)arg;
    char buffer[BUFFER_SIZE] = {0};

    read(new_socket, buffer, BUFFER_SIZE);
    printf("Server received: %s\n", buffer);
    send(new_socket, "Hello from server", strlen("Hello from server"), 0);
    printf("Server sent: %s\n", "Hello from server");

    close(new_socket);
    free(arg); // Free the allocated memory for the socket

    return NULL;
}

void *server_thread(void *arg) {
    int sockfd;
    char buffer[BUFFER_SIZE];
    struct sockaddr_in servaddr, cliaddr;
    socklen_t len;

    // 创建socket文件描述符
    if ((sockfd = socket(AF_INET, SOCK_DGRAM, 0)) < 0) {
        perror("socket creation failed");
        exit(EXIT_FAILURE);
    }

    memset(&servaddr, 0, sizeof(servaddr));
    memset(&cliaddr, 0, sizeof(cliaddr));

    // 填充服务器信息
    servaddr.sin_family = AF_INET; // IPv4
    servaddr.sin_addr.s_addr = INADDR_ANY;
    servaddr.sin_port = htons(PORT);

    // 绑定socket与地址
    if (bind(sockfd, (const struct sockaddr *)&servaddr, sizeof(servaddr)) < 0) {
        perror("bind failed");
        close(sockfd);
        exit(EXIT_FAILURE);
    }

    while (1) {
        len = sizeof(cliaddr); // 初始化cliaddr长度

        printf("server recvfrom\n");
        int n = recvfrom(sockfd, buffer, BUFFER_SIZE, MSG_WAITALL,
                         (struct sockaddr *)&cliaddr, &len);
        buffer[n] = '\0';
        printf("Server received: %s\n", buffer);
        
        // 回应消息给客户端
        sendto(sockfd, buffer, n, MSG_CONFIRM,
               (const struct sockaddr *)&cliaddr, len);
        printf("Server sent acknowledgment\n");
    }

    close(sockfd);
}

void *client_thread(void *arg) {
    sleep(1); // Ensure the server is listening before the client tries to connect

    int sockfd;
    //char buffer[BUFFER_SIZE];
    struct sockaddr_in servaddr;

    // 创建socket文件描述符
    if ((sockfd = socket(AF_INET, SOCK_DGRAM, 0)) < 0) {
        perror("socket creation failed");
        exit(EXIT_FAILURE);
    }

    memset(&servaddr, 0, sizeof(servaddr));

    // 填充服务器信息
    servaddr.sin_family = AF_INET;
    servaddr.sin_port = htons(PORT);
    
     // 将IP地址转换为二进制形式并存储在servaddr结构中。
     if (inet_pton(AF_INET,"127.0.0.1",&servaddr.sin_addr)<=0)
     {
         perror("Invalid address/ Address not supported");
         return -1;
     }

   while(1){
       printf("Enter message to send to server: ");
       //fgets(buffer,sizeof(buffer),stdin);
        char* buffer = "hello from client";

       sendto(sockfd,(const char*)buffer,strnlen(buffer,sizeof(buffer)),MSG_CONFIRM,(const struct sockaddr*)&servaddr,sizeof(servaddr));
       printf("Message sent.\n");

       int n,len;

       n=recvfrom(sockfd,(char*)buffer,sizeof(buffer),MSG_WAITALL,(struct sockaddr*)&servaddr,&len);
       buffer[n]='\0';
       printf("Server : %s\n",buffer);
       break;
   }
   close(sockfd);

   return NULL;
}

int main() {
    pthread_t server_tid, client_tid;

    if (pthread_create(&server_tid, NULL, server_thread, NULL) != 0) {
        perror("Failed to create server thread");
        exit(EXIT_FAILURE);
    }

    if (pthread_create(&client_tid, NULL, client_thread, NULL) != 0) {
        perror("Failed to create client thread");
        exit(EXIT_FAILURE);
    }

    pthread_join(server_tid, NULL);
    pthread_join(client_tid, NULL);

    return 0;
}
