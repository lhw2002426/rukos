/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#ifndef __SYS_TYPES_H__
#define __SYS_TYPES_H__

#include <stddef.h>

typedef unsigned char u_char;

typedef unsigned mode_t;
typedef uint32_t nlink_t;
typedef int64_t off_t;
typedef uint64_t ino_t;
typedef uint64_t dev_t;
typedef long blksize_t;
typedef int64_t blkcnt_t;

typedef int pid_t;
typedef unsigned uid_t;
typedef unsigned gid_t;

#endif // __SYS_TYPES_H__
