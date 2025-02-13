/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#ifndef __LIMITS__
#define __LIMITS__

#define __LONG_MAX 0x7fffffffffffffffL
#define CHAR_BIT   8
#define SCHAR_MIN  (-128)
#define SCHAR_MAX  127
#define UCHAR_MAX  255
#define SHRT_MIN   (-1 - 0x7fff)
#define SHRT_MAX   0x7fff
#define USHRT_MAX  0xffff
#define INT_MIN    (-1 - 0x7fffffff)
#define INT_MAX    0x7fffffff
#define UINT_MAX   0xffffffffU
#define LONG_MIN   (-LONG_MAX - 1)
#define LONG_MAX   __LONG_MAX
#define ULONG_MAX  (2UL * LONG_MAX + 1)
#define LLONG_MIN  (-LLONG_MAX - 1)
#define LLONG_MAX  0x7fffffffffffffffLL
#define ULLONG_MAX (2ULL * LLONG_MAX + 1)
#define IOV_MAX    1024

#define PTHREAD_STACK_MIN 2048

#define LOGIN_NAME_MAX 256
#ifndef NAME_MAX
#define NAME_MAX 255
#endif
#define TZNAME_MAX 6

#define PATH_MAX  4096
#define SSIZE_MAX LONG_MAX
#define CHAR_MAX  127

#define PTHREAD_KEYS_MAX 1024

#endif
