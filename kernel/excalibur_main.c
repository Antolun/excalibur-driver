// SPDX-License-Identifier: GPL-3.0-or-later
/*
 * excalibur_main.c - Linux Kernel Driver for Casper Excalibur Laptops
 *
 * Integrates Rust core logic with Linux ACPI/WMI, HWMON, LED subsystem,
 * and Platform Profile subsystem.
 *
 * Copyright (C) 2024-2026 Solzic0 <solzic0@yandex.com>
 */

#define pr_fmt(fmt) KBUILD_MODNAME ": " fmt

#include <linux/acpi.h>
#include <linux/bitfield.h>
#include <linux/device.h>
#include <linux/dmi.h>
#include <linux/hwmon.h>
#include <linux/leds.h>
#include <linux/module.h>
#include <linux/mutex.h>
#include <linux/platform_profile.h>
#include <linux/slab.h>
#include <linux/string.h>
#include <linux/types.h>
#include <linux/version.h>
#include <linux/wmi.h>

#include "excalibur_core.h"

MODULE_AUTHOR("Solzic0");
MODULE_DESCRIPTION("Casper Excalibur WMI Driver (Rust Core + DKMS)");
MODULE_LICENSE("GPL");
MODULE_VERSION("2.0.0");

/**
 * struct excalibur_zone - per-zone LED state cache
 * @cdev:    LED class device
 * @zone_id: hardware zone identifier
 * @mode:    cached animation mode (0..7)
 * @r:       cached red component (0..255)
 * @g:       cached green component (0..255)
 * @b:       cached blue component (0..255)
 */
struct excalibur_zone {
	struct led_classdev cdev;
	u8 zone_id;
	u8 mode;
	u8 r, g, b;
};

/**
 * struct excalibur_wmi_data - main driver context
 * @wdev:             WMI device handle
 * @has_raw_fanspeed: true for little-endian fan RPM, false for big-endian
 * @lock:             mutex protecting all hardware and state access
 * @ppdev:            platform profile device handle
 * @zones:            left, middle, right, corners
 */
struct excalibur_wmi_data {
	struct wmi_device *wdev;
	bool has_raw_fanspeed;
	struct mutex lock;
	struct device *ppdev;
	struct excalibur_zone zones[EXCALIBUR_TOTAL_ZONE_COUNT];
};

struct excalibur_quirk {
	bool has_raw_fanspeed;
};

static const struct excalibur_quirk excalibur_quirk_old_gen = {
	.has_raw_fanspeed = false,
};

static const struct excalibur_quirk excalibur_quirk_new_gen = {
	.has_raw_fanspeed = true,
};

static int dmi_matched(const struct dmi_system_id *dmi)
{
	pr_info("Identified model: '%s'\n", dmi->ident);
	return 1;
}

static const struct dmi_system_id excalibur_dmi_list[] = {
	{
		.callback    = dmi_matched,
		.ident       = "EXCALIBUR G650",
		.matches     = {
			DMI_MATCH(DMI_SYS_VENDOR,   "CASPER BILGISAYAR SISTEMLERI"),
			DMI_MATCH(DMI_PRODUCT_NAME, "EXCALIBUR G650"),
		},
		.driver_data = (void *)&excalibur_quirk_old_gen,
	},
	{
		.callback    = dmi_matched,
		.ident       = "EXCALIBUR G750",
		.matches     = {
			DMI_MATCH(DMI_SYS_VENDOR,   "CASPER BILGISAYAR SISTEMLERI"),
			DMI_MATCH(DMI_PRODUCT_NAME, "EXCALIBUR G750"),
		},
		.driver_data = (void *)&excalibur_quirk_old_gen,
	},
	{
		.callback    = dmi_matched,
		.ident       = "EXCALIBUR G670",
		.matches     = {
			DMI_MATCH(DMI_SYS_VENDOR,   "CASPER BILGISAYAR SISTEMLERI"),
			DMI_MATCH(DMI_PRODUCT_NAME, "EXCALIBUR G670"),
		},
		.driver_data = (void *)&excalibur_quirk_old_gen,
	},
	{
		.callback    = dmi_matched,
		.ident       = "EXCALIBUR G900",
		.matches     = {
			DMI_MATCH(DMI_SYS_VENDOR,   "CASPER BILGISAYAR SISTEMLERI"),
			DMI_MATCH(DMI_PRODUCT_NAME, "EXCALIBUR G900"),
		},
		.driver_data = (void *)&excalibur_quirk_old_gen,
	},
	{
		.callback    = dmi_matched,
		.ident       = "EXCALIBUR G870",
		.matches     = {
			DMI_MATCH(DMI_SYS_VENDOR,   "CASPER BILGISAYAR SISTEMLERI"),
			DMI_MATCH(DMI_PRODUCT_NAME, "EXCALIBUR G870"),
		},
		.driver_data = (void *)&excalibur_quirk_new_gen,
	},
	{
		.callback    = dmi_matched,
		.ident       = "EXCALIBUR G770",
		.matches     = {
			DMI_MATCH(DMI_SYS_VENDOR,   "CASPER BILGISAYAR SISTEMLERI"),
			DMI_MATCH(DMI_PRODUCT_NAME, "EXCALIBUR G770"),
		},
		.driver_data = (void *)&excalibur_quirk_new_gen,
	},
	{
		.callback    = dmi_matched,
		.ident       = "EXCALIBUR G911",
		.matches     = {
			DMI_MATCH(DMI_SYS_VENDOR,   "CASPER BILGISAYAR SISTEMLERI"),
			DMI_MATCH(DMI_PRODUCT_NAME, "EXCALIBUR G911"),
		},
		.driver_data = (void *)&excalibur_quirk_new_gen,
	},
	{ }
};

/* ================================================================
 * WMI Low-level Communication
 * ================================================================ */

static int excalibur_wmi_write(struct excalibur_wmi_data *drv, u16 cmd,
			       u32 param, u32 data)
{
	struct excalibur_wmi_args args;
	struct acpi_buffer input;

	excalibur_core_create_write_args(cmd, param, data, &args);
	input.length = sizeof(args);
	input.pointer = &args;

	if (ACPI_FAILURE(wmidev_block_set(drv->wdev, 0, &input)))
		return -EIO;
	return 0;
}

static int excalibur_wmi_read(struct excalibur_wmi_data *drv, u16 cmd,
			      struct excalibur_wmi_args *out)
{
	struct excalibur_wmi_args args;
	struct acpi_buffer input;
	union acpi_object *obj;

	excalibur_core_create_read_args(cmd, &args);
	input.length = sizeof(args);
	input.pointer = &args;

	if (ACPI_FAILURE(wmidev_block_set(drv->wdev, 0, &input)))
		return -EIO;

	obj = wmidev_block_query(drv->wdev, 0);
	if (!obj)
		return -EIO;

	if (obj->type != ACPI_TYPE_BUFFER ||
	    obj->buffer.length < sizeof(*out) ||
	    !obj->buffer.pointer) {
		kfree(obj);
		return -EIO;
	}

	memcpy(out, obj->buffer.pointer, sizeof(*out));
	kfree(obj);
	return 0;
}

/* ================================================================
 * LED Subsystem Integration
 * ================================================================ */

static int excalibur_commit_zone(struct excalibur_wmi_data *drv,
				 struct excalibur_zone *zone)
{
	u32 packed = excalibur_core_pack_led(zone->mode,
					     zone->cdev.brightness,
					     zone->r, zone->g, zone->b);

	return excalibur_wmi_write(drv, EXCALIBUR_CMD_SET_LED,
				   zone->zone_id, packed);
}

static struct excalibur_wmi_data *cdev_to_drv(struct led_classdev *cdev)
{
	return dev_get_drvdata(cdev->dev->parent);
}

static struct excalibur_zone *cdev_to_zone(struct led_classdev *cdev)
{
	return container_of(cdev, struct excalibur_zone, cdev);
}

static void excalibur_kbd_brightness_set(struct led_classdev *cdev,
					 enum led_brightness brightness)
{
	struct excalibur_wmi_data *drv = cdev_to_drv(cdev);
	struct excalibur_zone *zone = cdev_to_zone(cdev);
	u32 packed;
	int i;

	mutex_lock(&drv->lock);

	for (i = 0; i < EXCALIBUR_KBD_ZONE_COUNT; i++)
		drv->zones[i].cdev.brightness = brightness;

	packed = excalibur_core_pack_led(zone->mode, brightness,
					 zone->r, zone->g, zone->b);

	excalibur_wmi_write(drv, EXCALIBUR_CMD_SET_LED,
			    EXCALIBUR_ZONE_ALL_KBD, packed);

	mutex_unlock(&drv->lock);
}

static enum led_brightness excalibur_kbd_brightness_get(struct led_classdev *cdev)
{
	return cdev_to_zone(cdev)->cdev.brightness;
}

static void excalibur_corner_brightness_set(struct led_classdev *cdev,
					    enum led_brightness brightness)
{
	struct excalibur_wmi_data *drv = cdev_to_drv(cdev);
	struct excalibur_zone *zone = cdev_to_zone(cdev);

	mutex_lock(&drv->lock);
	zone->cdev.brightness = brightness;
	excalibur_commit_zone(drv, zone);
	mutex_unlock(&drv->lock);
}

static enum led_brightness excalibur_corner_brightness_get(struct led_classdev *cdev)
{
	return cdev_to_zone(cdev)->cdev.brightness;
}

/* Sysfs Attributes per zone */
static ssize_t color_store(struct device *dev, struct device_attribute *attr,
			   const char *buf, size_t count)
{
	struct led_classdev *cdev = dev_get_drvdata(dev);
	struct excalibur_wmi_data *drv = cdev_to_drv(cdev);
	struct excalibur_zone *zone = cdev_to_zone(cdev);
	u8 r, g, b;
	int ret;

	ret = excalibur_core_parse_color_hex(buf, count, &r, &g, &b);
	if (ret)
		return ret;

	mutex_lock(&drv->lock);
	zone->r = r;
	zone->g = g;
	zone->b = b;
	ret = excalibur_commit_zone(drv, zone);
	mutex_unlock(&drv->lock);

	return ret ? ret : count;
}
static DEVICE_ATTR_WO(color);

static ssize_t mode_show(struct device *dev, struct device_attribute *attr,
			 char *buf)
{
	struct led_classdev *cdev = dev_get_drvdata(dev);
	struct excalibur_zone *zone = cdev_to_zone(cdev);

	return sysfs_emit(buf, "%s\n", excalibur_core_mode_name(zone->mode));
}

static ssize_t mode_store(struct device *dev, struct device_attribute *attr,
			  const char *buf, size_t count)
{
	struct led_classdev *cdev = dev_get_drvdata(dev);
	struct excalibur_wmi_data *drv = cdev_to_drv(cdev);
	struct excalibur_zone *zone = cdev_to_zone(cdev);
	u8 mode;
	int ret;

	ret = excalibur_core_parse_mode_str(buf, count, &mode);
	if (ret)
		return ret;

	mutex_lock(&drv->lock);
	zone->mode = mode;
	ret = excalibur_commit_zone(drv, zone);
	mutex_unlock(&drv->lock);

	return ret ? ret : count;
}
static DEVICE_ATTR_RW(mode);

static ssize_t available_modes_show(struct device *dev,
				    struct device_attribute *attr, char *buf)
{
	return sysfs_emit(buf, "%s\n", excalibur_core_available_modes());
}
static DEVICE_ATTR_RO(available_modes);

static ssize_t raw_store(struct device *dev, struct device_attribute *attr,
			 const char *buf, size_t count)
{
	struct led_classdev *cdev = dev_get_drvdata(dev);
	struct excalibur_wmi_data *drv = cdev_to_drv(cdev);
	struct excalibur_zone *zone = cdev_to_zone(cdev);
	u32 data;
	int ret;

	ret = kstrtou32(skip_spaces(buf), 16, &data);
	if (ret)
		return ret;

	mutex_lock(&drv->lock);
	ret = excalibur_wmi_write(drv, EXCALIBUR_CMD_SET_LED,
				  zone->zone_id, data);
	if (!ret) {
		u8 br;
		excalibur_core_unpack_led(data, &zone->mode,
					  &br,
					  &zone->r, &zone->g, &zone->b);
		zone->cdev.brightness = br;
	}
	mutex_unlock(&drv->lock);

	return ret ? ret : count;
}
static DEVICE_ATTR_WO(raw);

static struct attribute *excalibur_zone_attrs[] = {
	&dev_attr_color.attr,
	&dev_attr_mode.attr,
	&dev_attr_available_modes.attr,
	&dev_attr_raw.attr,
	NULL,
};
ATTRIBUTE_GROUPS(excalibur_zone);

/* ================================================================
 * HWMON Subsystem Integration
 * ================================================================ */

static umode_t excalibur_hwmon_is_visible(const void *drvdata,
					  enum hwmon_sensor_types type,
					  u32 attr, int channel)
{
	switch (type) {
	case hwmon_fan:
		return 0444;
	case hwmon_pwm:
		return 0644;
	default:
		return 0;
	}
}

static int excalibur_hwmon_read(struct device *dev,
				enum hwmon_sensor_types type,
				u32 attr, int channel, long *val)
{
	struct excalibur_wmi_data *drv = dev_get_drvdata(dev->parent);
	struct excalibur_wmi_args out = { 0 };
	int ret;

	switch (type) {
	case hwmon_fan:
		if (channel > 1)
			return -EINVAL;
		ret = excalibur_wmi_read(drv, EXCALIBUR_CMD_GET_HW_INFO, &out);
		if (ret)
			return ret;
		*val = excalibur_core_decode_fanspeed(channel == 0 ? out.a4 : out.a5,
						      drv->has_raw_fanspeed);
		return 0;

	case hwmon_pwm:
		if (channel != 0)
			return -EOPNOTSUPP;
		ret = excalibur_wmi_read(drv, EXCALIBUR_CMD_POWERPLAN, &out);
		if (ret)
			return ret;
		*val = (long)out.a2;
		return 0;

	default:
		return -EOPNOTSUPP;
	}
}

static int excalibur_hwmon_read_string(struct device *dev,
				       enum hwmon_sensor_types type, u32 attr,
				       int channel, const char **str)
{
	if (type != hwmon_fan || channel > 1)
		return -EOPNOTSUPP;

	*str = excalibur_core_fan_label(channel);
	return 0;
}

static int excalibur_hwmon_write(struct device *dev,
				 enum hwmon_sensor_types type,
				 u32 attr, int channel, long val)
{
	struct excalibur_wmi_data *drv = dev_get_drvdata(dev->parent);

	if (type != hwmon_pwm || channel != 0)
		return -EOPNOTSUPP;

	if (!excalibur_core_validate_power_plan((u32)val))
		return -EINVAL;

	return excalibur_wmi_write(drv, EXCALIBUR_CMD_POWERPLAN, (u32)val, 0);
}

static const struct hwmon_ops excalibur_hwmon_ops = {
	.is_visible  = excalibur_hwmon_is_visible,
	.read        = excalibur_hwmon_read,
	.read_string = excalibur_hwmon_read_string,
	.write       = excalibur_hwmon_write,
};

static const struct hwmon_channel_info *const excalibur_hwmon_info[] = {
	HWMON_CHANNEL_INFO(fan,
			   HWMON_F_INPUT | HWMON_F_LABEL,
			   HWMON_F_INPUT | HWMON_F_LABEL),
	HWMON_CHANNEL_INFO(pwm, HWMON_PWM_INPUT),
	NULL
};

static const struct hwmon_chip_info excalibur_hwmon_chip_info = {
	.ops  = &excalibur_hwmon_ops,
	.info = excalibur_hwmon_info,
};

/* ================================================================
 * Linux Platform Profile Subsystem Integration
 * ================================================================ */

static int excalibur_profile_probe(void *drvdata, unsigned long *choices)
{
	set_bit(PLATFORM_PROFILE_LOW_POWER, choices);
	set_bit(PLATFORM_PROFILE_QUIET, choices);
	set_bit(PLATFORM_PROFILE_BALANCED, choices);
	set_bit(PLATFORM_PROFILE_PERFORMANCE, choices);
	return 0;
}

static int excalibur_profile_get(struct device *dev,
				 enum platform_profile_option *profile)
{
	struct excalibur_wmi_data *drv = dev_get_drvdata(dev);
	struct excalibur_wmi_args out = { 0 };
	int ret, plan;

	ret = excalibur_wmi_read(drv, EXCALIBUR_CMD_POWERPLAN, &out);
	if (ret)
		return ret;

	plan = excalibur_core_plan_to_platform_profile(out.a2);
	if (plan < 0)
		return plan;

	*profile = (enum platform_profile_option)plan;
	return 0;
}

static int excalibur_profile_set(struct device *dev,
				 enum platform_profile_option profile)
{
	struct excalibur_wmi_data *drv = dev_get_drvdata(dev);
	int plan = excalibur_core_platform_profile_to_plan((int)profile);

	if (plan < 0)
		return plan;

	return excalibur_wmi_write(drv, EXCALIBUR_CMD_POWERPLAN, (u32)plan, 0);
}

static const struct platform_profile_ops excalibur_platform_profile_ops = {
	.probe       = excalibur_profile_probe,
	.profile_get = excalibur_profile_get,
	.profile_set = excalibur_profile_set,
};

/* ================================================================
 * Driver Probe and Remove
 * ================================================================ */

static int excalibur_wmi_probe(struct wmi_device *wdev, const void *context)
{
	const struct excalibur_quirk *quirk = context;
	struct excalibur_wmi_data *drv;
	struct device *hwmon_dev;
	bool model_known;
	int i, ret;

	drv = devm_kzalloc(&wdev->dev, sizeof(*drv), GFP_KERNEL);
	if (!drv)
		return -ENOMEM;

	drv->wdev = wdev;
	dev_set_drvdata(&wdev->dev, drv);

	ret = devm_mutex_init(&wdev->dev, &drv->lock);
	if (ret)
		return ret;

	if (quirk) {
		drv->has_raw_fanspeed = quirk->has_raw_fanspeed;
		model_known = true;
	} else {
		const struct dmi_system_id *dmi_id;

		dmi_id = dmi_first_match(excalibur_dmi_list);
		if (dmi_id) {
			quirk = dmi_id->driver_data;
			drv->has_raw_fanspeed = quirk->has_raw_fanspeed;
			model_known = true;
		} else {
			drv->has_raw_fanspeed = true;
			model_known = false;
		}
	}

	if (!model_known)
		dev_warn(&wdev->dev,
			 "Unrecognised Excalibur model - defaulting to native fanspeed format.\n");

	/* Register LED devices for 4 zones */
	for (i = 0; i < EXCALIBUR_TOTAL_ZONE_COUNT; i++) {
		struct excalibur_zone *zone = &drv->zones[i];
		bool is_corner = (i == EXCALIBUR_TOTAL_ZONE_COUNT - 1);

		zone->zone_id          = excalibur_core_get_zone_id(i);
		zone->mode             = 1; /* static */
		zone->r                = 0xFF;
		zone->g                = 0xFF;
		zone->b                = 0xFF;
		zone->cdev.name        = excalibur_core_get_zone_name(i);
		zone->cdev.max_brightness = EXCALIBUR_MAX_BRIGHTNESS;
		zone->cdev.brightness  = 0;
		zone->cdev.groups      = excalibur_zone_groups;

		if (is_corner) {
			zone->cdev.brightness_set = excalibur_corner_brightness_set;
			zone->cdev.brightness_get = excalibur_corner_brightness_get;
		} else {
			zone->cdev.brightness_set = excalibur_kbd_brightness_set;
			zone->cdev.brightness_get = excalibur_kbd_brightness_get;
		}

		ret = devm_led_classdev_register(&wdev->dev, &zone->cdev);
		if (ret) {
			dev_err(&wdev->dev, "Failed to register LED zone %d: %d\n", i, ret);
			return ret;
		}
	}

	/* Register HWMON device */
	hwmon_dev = devm_hwmon_device_register_with_info(&wdev->dev,
							 "excalibur_wmi", drv,
							 &excalibur_hwmon_chip_info,
							 NULL);
	if (IS_ERR(hwmon_dev)) {
		ret = PTR_ERR(hwmon_dev);
		dev_err(&wdev->dev, "Failed to register hwmon device: %d\n", ret);
		return ret;
	}

	/* Register Platform Profile device */
	drv->ppdev = devm_platform_profile_register(&wdev->dev, "excalibur",
						    drv, &excalibur_platform_profile_ops);
	if (IS_ERR(drv->ppdev)) {
		dev_warn(&wdev->dev, "Platform profile registration failed: %ld (continuing)\n",
			 PTR_ERR(drv->ppdev));
		drv->ppdev = NULL;
	} else {
		dev_info(&wdev->dev, "Platform profile registered successfully\n");
	}

	dev_info(&wdev->dev, "Casper Excalibur driver loaded successfully (Rust Core + DKMS)\n");
	return 0;
}

static const struct wmi_device_id excalibur_wmi_id_table[] = {
	{ .guid_string = EXCALIBUR_WMI_GUID },
	{ }
};
MODULE_DEVICE_TABLE(wmi, excalibur_wmi_id_table);

static struct wmi_driver excalibur_wmi_driver = {
	.driver       = { .name = "excalibur-wmi" },
	.id_table     = excalibur_wmi_id_table,
	.probe        = excalibur_wmi_probe,
	.no_singleton = true,
};

module_wmi_driver(excalibur_wmi_driver);
