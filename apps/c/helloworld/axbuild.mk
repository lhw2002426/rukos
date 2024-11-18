app-objs=main.o

#ARGS = /home/lhw/redis-7.0.12/src/redis-server, /home/lhw/redis.conf
ARGS = /bin/main
ENVS = 
V9P_PATH=${APP}/rootfs

# make run ARCH=aarch64 A=apps/c/dl V9P=y MUSL=y LOG=debug