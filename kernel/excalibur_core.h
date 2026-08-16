/* SPDX-License-Identifier: GPL-3.0-or-later */
/*
 * excalibur_core.h - C FFI Prototypes for Rust Core Logic
 *
 * Copyright (C) 2024-2026 Solzic0 <solzic0@yandex.com>
 */

#ifndef _EXCALIBUR_CORE_H_
#define _EXCALIBUR_CORE_H_

#include <linux/types.h>
#include <linux/errno.h>

#define EXCALIBUR_WMI_GUID "644C5791-B7B0-4123-A90B-E93876E0DAAD"

#define EXCALIBUR_CMD_READ            0xfa00
#define EXCALIBUR_CMD_WRITE           0xfb00
#define EXCALIBUR_CMD_GET_HW_INFO     0x0200
#define EXCALIBUR_CMD_SET_LED         0x0100
#define EXCALIBUR_CMD_POWERPLAN       0x0300

#define EXCALIBUR_ZONE_LEFT           0x05
#define EXCALIBUR_ZONE_MIDDLE         0x04
#define EXCALIBUR_ZONE_RIGHT          0x03
#define EXCALIBUR_ZONE_ALL_KBD        0x06
#define EXCALIBUR_ZONE_CORNERS        0x07

#define EXCALIBUR_KBD_ZONE_COUNT      3
#define EXCALIBUR_TOTAL_ZONE_COUNT    4
#define EXCALIBUR_MAX_BRIGHTNESS      2

/* WMI packet structure matching Rust ExcaliburWmiArgs */
struct excalibur_wmi_args {
	u16 a0;
	u16 a1;
	u32 a2;
	u32 a3;
	u32 a4;
	u32 a5;
	u32 a6;
	u32 rev0;
	u32 rev1;
};

/* Rust Core FFI functions */
extern const char *excalibur_core_get_zone_name(size_t index);
extern u8 excalibur_core_get_zone_id(size_t index);
extern u32 excalibur_core_pack_led(u8 mode, u8 brightness, u8 r, u8 g, u8 b);
extern void excalibur_core_unpack_led(u32 packed, u8 *mode, u8 *brightness, u8 *r, u8 *g, u8 *b);
extern int excalibur_core_parse_color_hex(const char *str_ptr, size_t len, u8 *r, u8 *g, u8 *b);
extern int excalibur_core_parse_mode_str(const char *str_ptr, size_t len, u8 *mode_out);
extern const char *excalibur_core_mode_name(u8 mode);
extern const char *excalibur_core_available_modes(void);
extern u16 excalibur_core_decode_fanspeed(u32 raw, bool has_raw_fanspeed);
extern const char *excalibur_core_fan_label(size_t channel);
extern bool excalibur_core_validate_power_plan(u32 plan);
extern int excalibur_core_platform_profile_to_plan(int profile);
extern int excalibur_core_plan_to_platform_profile(u32 plan);
extern void excalibur_core_create_write_args(u16 sub_cmd, u32 param, u32 data, struct excalibur_wmi_args *out);
extern void excalibur_core_create_read_args(u16 sub_cmd, struct excalibur_wmi_args *out);

#endif /* _EXCALIBUR_CORE_H_ */
