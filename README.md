# hyperx-monitor
A lightweight battery monitor application for the HyperX Cloud Stinger 2 Wireless, built in Rust, for Linux.

## How does this work?
I was able to analyze the data between the host and headset to identify different commands between them.

Battery Level:
```[0x06, 0xFF, 0xBB, 0x02]``` [sending procedure: host -> headset]
- The first byte - [0x06] - defines the Report ID.
- The middle two bytes - [0xFF, 0xBB] - might not represent anything (but uncertain).
- The last byte - [0x02] - represents the command that's sent.
- This set of data, sent from the host to the headset, will return the set of data that includes the battery level. The battery level is placed at index ```[7]``` of the output buffer.

Charging Status:
```[0x06, 0xFF, 0xBB, 0x03, 0x01]``` or ```[0x06, 0xFF, 0xBB, 0x03, 0x00]``` [sending procedure: headset -> host]
- Similarly to the battery level command, this command includes a Report ID [0x06] and two placeholder bytes [0xFF, 0xBB].
- The command that is sent from the headset to the host shows "boolean" characteristics; 1 being true, and 0 being false.