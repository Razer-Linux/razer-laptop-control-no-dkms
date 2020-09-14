#include "chroma.h"

struct row_data matrix[5];


int displayMatrix(struct usb_device *usb) {
    int row;
    for (row=0; row<=5; row++) {
        sendRowDataToProfile(usb, row);
    }
    return 0;
}

int sendRowDataToProfile(struct usb_device *usb, int row_number) {
    struct razer_packet packet = {0};
    packet = get_razer_report(0x03, 0x0b, 0x34);

    packet.args[0] = 0xff;
    packet.args[1] = row_number;
    packet.args[3] = 0x0f;
    memcpy(&packet.args[7], &matrix[row_number].keys, 45);
    send_payload(usb, &packet);

    return 0;
}

int displayProfile(struct usb_device *usb, int profileNum) {
    struct razer_packet packet = {0};
    packet = get_razer_report(0x03, 0x0a, 0x02);

    packet.args[0] = 0x05;
    packet.args[1] = 0x00;
    send_payload(usb, &packet);
    return 0;
}

int sendBrightness(struct usb_device *usb, __u8 brightness) {
    struct razer_packet packet = {0};
    packet = get_razer_report(0x03, 0x03, 0x03);

    packet.args[0] = 0x01;
    packet.args[1] = 0x05;
    packet.args[2] = brightness;
    send_payload(usb, &packet);
    return 0;
}

int getBrightness(struct usb_device *usb) {
    struct razer_packet req = {0};
    struct razer_packet resp = {0};

    req = get_razer_report(0x03, 0x83, 0x03);

    req.args[0] = 0x01;
    req.args[1] = 0x05;
    req.args[2] = 0x00;
    resp = send_payload(usb, &req);

    return resp.args[2];
}
