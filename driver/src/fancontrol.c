// SPDX-License-Identifier: GPL-2.0
#include "fancontrol.h"
#include "core.h"

/*
 * Returns the highest valid fan RPM for manual fan control
 *
 * We could through a higher value at the EC, it may even accept it which would
 * be dangerous, so limit it to what we know is safe based on using Synapse.
 *
 * Each model of Blade seems to have a different upper limit, so we require the
 * product ID to set the correct upper bound
 */
static int get_max_fan_rpm(__u32 product_id)
{
	switch (product_id) {
	case BLADE_2018_ADV:
	case BLADE_2018_BASE:
		return MAX_FAN_RPM_2018_BLADE;
	case BLADE_2019_ADV:
	case BLADE_2019_MERC:
	case BLADE_PRO_2019:
	case BLADE_2019_STEALTH:
    case BLADE_2020_ADV:
		return MAX_FAN_RPM_STEALTH;
	default:
		return MAX_FAN_RPM_DEFAULT;
	}
}

static u8 clamp_fan_rpm(unsigned int rpm, __u32 product_id)
{
	int max_allowed = get_max_fan_rpm(product_id);

	if (rpm > max_allowed)
		return (u8)(max_allowed / 100);
	else if (rpm < ABSOLUTE_MIN_FAN_RPM)
		return (u8)(ABSOLUTE_MIN_FAN_RPM / 100);

	return (u8)(rpm / 100);
}

static int creator_mode_allowed(__u32 product_id)
{
	switch (product_id) {
	case BLADE_2019_ADV:
	case BLADE_2019_MERC:
    case BLADE_2020_ADV:
		return 1;
	default:
		return 0;
	}
}

static int boost_mode_allowed(__u32 product_id)
{
    switch (product_id) {
    case BLADE_2020_ADV:
        return 1;
    default:
        return 0;
    }
}

void set_fan_rpm(unsigned long x, struct device *dev)
{
    struct usb_interface *intf = to_usb_interface(dev->parent);
    struct usb_device *usb_dev = interface_to_usbdev(intf);
    struct razer_laptop *laptop = dev_get_drvdata(dev);

    mutex_lock(&laptop->lock);
    if(laptop->power_mode < 4) // custom mode do not support fan profile
    {
        u8 request_fan_speed;
        struct razer_packet report = {0};
        /* char buffer[90] = {0x00}; */
        if (x != 0) {
            request_fan_speed = clamp_fan_rpm(x, laptop->product_id);
            laptop->fan_rpm = request_fan_speed * 100;
            // All packets
            report = get_razer_report(0x0d, 0x82, 0x04);
            // get current power mode
            report.args[0] = 0x00;
            report.args[1] = 0x01;
            report.args[2] = 0x00;
            report.args[3] = 0x00;
            send_payload(usb_dev, &report);

            // set current power mode with custom rpm
            report = get_razer_report(0x0d, 0x02, 0x04);
            report.args[0] = 0x00;
            report.args[1] = 0x01;
            report.args[2] = laptop->power_mode;
            report.args[3] = laptop->fan_rpm != 0 ? 0x01 : 0x00;
            send_payload(usb_dev, &report);

            report = get_razer_report(0x0d, 0x01, 0x03);
            // Set fan RPM
            report.args[0] = 0x00;
            report.args[1] = 0x01;
            report.args[2] = request_fan_speed;
            send_payload(usb_dev, &report);

            report = get_razer_report(0x0d, 0x82, 0x04);
            report.args[0] = 0x00;
            report.args[1] = 0x02;
            report.args[2] = 0x00;
            report.args[3] = 0x00;
            send_payload(usb_dev, &report);
        } else {
            laptop->fan_rpm = 0;
        }
        // Fan mode
        report = get_razer_report(0x0d, 0x82, 0x04);
        report.args[0] = 0x00;
        report.args[1] = 0x02;
        report.args[2] = laptop->power_mode;
        report.args[3] = laptop->fan_rpm != 0 ? 0x01 : 0x00;
        send_payload(usb_dev, &report);

        if (x != 0) {
            // Set fan RPM
            report = get_razer_report(0x0d, 0x01, 0x03);
            report.args[0] = 0x00;
            report.args[1] = 0x02;
            report.args[2] = request_fan_speed;
            send_payload(usb_dev, &report);
        }
    }
    mutex_unlock(&laptop->lock);
}

int set_power_mode(unsigned long x, struct device *dev)
{
    struct usb_interface *intf = to_usb_interface(dev->parent);
    struct usb_device *usb_dev = interface_to_usbdev(intf);
    struct razer_laptop *laptop = dev_get_drvdata(dev);
    struct razer_packet report = {0};

    mutex_lock(&laptop->lock);
    if (x >= 0 && x <= 2) {
        // Device doesn't support creator mode
        if (x == 2 && !creator_mode_allowed(laptop->product_id)) {
            x = 1;
        }

        laptop->power_mode = x;
        report = get_razer_report(0x0d, 0x02, 0x04);
        report.args[0] = 0x00;
        report.args[1] = 0x01;
        report.args[2] = laptop->power_mode;
        report.args[3] = laptop->fan_rpm != 0 ? 0x01 : 0x00; // Custom RPM ?
        send_payload(usb_dev, &report);
    }
    else if(x == 4)
    {
        laptop->power_mode = x;
        // read power mode
        report = get_razer_report(0x0d, 0x82, 0x04);
        report.args[0] = 0x00;
        report.args[1] = 0x01;
        report.args[2] = 0x00;
        report.args[3] = 0x00;
        send_payload(usb_dev, &report);

        report = get_razer_report(0x0d, 0x02, 0x04);

        // set power mode
        report.args[0] = 0x00;
        report.args[1] = 0x01;
        report.args[2] = laptop->power_mode;
        report.args[3] = 0x00;
        send_payload(usb_dev, &report);

        // Read cpu boost
        report = get_razer_report(0x0d, 0x87, 0x03);
        report.args[0] = 0x00;
        report.args[1] = 0x01;
        report.args[2] = 0x00;
        send_payload(usb_dev, &report);

        // Set cpu boost
        report = get_razer_report(0x0d, 0x07, 0x03);
        report.args[0] = 0x00;
        report.args[1] = 0x01;
        report.args[2] = laptop->cpu_boost;
        send_payload(usb_dev, &report);

        report = get_razer_report(0x0d, 0x87, 0x03);
        // Read gpu boost
        report.args[0] = 0x00;
        report.args[1] = 0x02;
        report.args[2] = 0x00;
        send_payload(usb_dev, &report);

        report = get_razer_report(0x0d, 0x07, 0x03);
        // Set gpu boost
        report.args[0] = 0x00;
        report.args[1] = 0x02;
        report.args[2] = laptop->gpu_boost;
        send_payload(usb_dev, &report);

        report = get_razer_report(0x0d, 0x82, 0x04);
        // read
        report.args[0] = 0x00;
        report.args[1] = 0x02;
        report.args[2] = 0x00;
        report.args[3] = 0x00;
        send_payload(usb_dev, &report);

        report = get_razer_report(0x0d, 0x82, 0x04);
        // read
        report.args[0] = 0x00;
        report.args[1] = 0x02;
        report.args[2] = laptop->power_mode;
        report.args[3] = 0x00;
        send_payload(usb_dev, &report);
    }
    mutex_unlock(&laptop->lock);

    return 0;
}

int get_power_mode(struct device *dev)
{
    struct usb_interface *intf = to_usb_interface(dev->parent);
    struct usb_device *usb_dev = interface_to_usbdev(intf);
    struct razer_laptop *laptop = dev_get_drvdata(dev);

    int power_mode = 0;
    struct razer_packet report = {0};
    struct razer_packet response = {0};
    mutex_lock(&laptop->lock);
    report = get_razer_report(0x0d, 0x82, 0x04);
    report.args[0] = 0x00;
    report.args[1] = 0x01;
    report.args[2] = 0x00;
    report.args[3] = 0x00;
    response = send_payload(usb_dev, &report);

    power_mode = response.args[2];
    mutex_unlock(&laptop->lock);

    return power_mode;
}

int set_gpu_boost_mode(unsigned long gpu_boost, struct device *dev)
{
    struct usb_interface *intf = to_usb_interface(dev->parent);
    struct usb_device *usb_dev = interface_to_usbdev(intf);
    struct razer_laptop *laptop = dev_get_drvdata(dev);

    struct razer_packet report = {0};
    struct razer_packet response = {0};
    mutex_lock(&laptop->lock);
    laptop->gpu_boost = gpu_boost;
    report = get_razer_report(0x0d, 0x82, 0x04);
    report.args[0] = 0x00;
    report.args[1] = 0x01;
    report.args[2] = 0x00;
    report.args[3] = 0x00;
    response = send_payload(usb_dev, &report);

    if(response.args[2] == 4)
    {
        report = get_razer_report(0x0d, 0x87, 0x03);
        // Read gpu boost
        report.args[0] = 0x00;
        report.args[1] = 0x02;
        report.args[2] = 0x00;
        send_payload(usb_dev, &report);

        report = get_razer_report(0x0d, 0x07, 0x03);
        // Set gpu boost
        report.args[0] = 0x00;
        report.args[1] = 0x02;
        report.args[2] = laptop->gpu_boost;
        send_payload(usb_dev, &report);
    }
    mutex_unlock(&laptop->lock);

    return 0;
}

int get_gpu_boost_mode(struct device *dev)
{
    struct usb_interface *intf = to_usb_interface(dev->parent);
    struct usb_device *usb_dev = interface_to_usbdev(intf);
    struct razer_laptop *laptop = dev_get_drvdata(dev);

    int boost = 0;
    struct razer_packet report = {0};
    struct razer_packet response = {0};
    mutex_lock(&laptop->lock);
    report = get_razer_report(0x0d, 0x82, 0x04);
    report.args[0] = 0x00;
    report.args[1] = 0x01;
    report.args[2] = 0x00;
    report.args[3] = 0x00;
    response = send_payload(usb_dev, &report);

    if(response.args[2] == 4)
    {
        report = get_razer_report(0x0d, 0x87, 0x03);
        // Read gpu boost
        report.args[0] = 0x00;
        report.args[1] = 0x02;
        report.args[2] = 0x00;
        response = send_payload(usb_dev, &report);
        boost = response.args[2];
    }
    mutex_unlock(&laptop->lock);

    return boost;
}

int set_cpu_boost_mode(unsigned long cpu_boost, struct device *dev)
{
    struct usb_interface *intf = to_usb_interface(dev->parent);
    struct usb_device *usb_dev = interface_to_usbdev(intf);
    struct razer_laptop *laptop = dev_get_drvdata(dev);

    struct razer_packet report = {0};
    struct razer_packet response = {0};
    mutex_lock(&laptop->lock);
    if(cpu_boost == 3 && !boost_mode_allowed(laptop->product_id))
    {
        cpu_boost = 2;
    }

    laptop->cpu_boost = cpu_boost;
    report = get_razer_report(0x0d, 0x82, 0x04);
    report.args[0] = 0x00;
    report.args[1] = 0x01;
    report.args[2] = 0x00;
    report.args[3] = 0x00;
    response = send_payload(usb_dev, &report);

    if(response.args[2] == 4)
    {
        report = get_razer_report(0x0d, 0x87, 0x03);
        // Read gpu boost
        report.args[0] = 0x00;
        report.args[1] = 0x01;
        report.args[2] = 0x00;
        send_payload(usb_dev, &report);

        report = get_razer_report(0x0d, 0x07, 0x03);
        // Set gpu boost
        report.args[0] = 0x00;
        report.args[1] = 0x01;
        report.args[2] = laptop->cpu_boost;
        send_payload(usb_dev, &report);
    }
    mutex_unlock(&laptop->lock);

    return 0;
}

int get_cpu_boost_mode(struct device *dev)
{
    struct usb_interface *intf = to_usb_interface(dev->parent);
    struct usb_device *usb_dev = interface_to_usbdev(intf);
    struct razer_laptop *laptop = dev_get_drvdata(dev);

    int boost = 0;
    struct razer_packet report = {0};
    struct razer_packet response = {0};
    mutex_lock(&laptop->lock);
    report = get_razer_report(0x0d, 0x82, 0x04);
    report.args[0] = 0x00;
    report.args[1] = 0x01;
    report.args[2] = 0x00;
    report.args[3] = 0x00;
    response = send_payload(usb_dev, &report);

    if(response.args[2] == 4)
    {
        report = get_razer_report(0x0d, 0x87, 0x03);
        // Read gpu boost
        report.args[0] = 0x00;
        report.args[1] = 0x01;
        report.args[2] = 0x00;
        response = send_payload(usb_dev, &report);
        boost = response.args[2];
    }
    mutex_unlock(&laptop->lock);

    return boost;
}
