# Overview
This documentation will try to list and explain commands for the Cloud Stinger 2 Wireless.<br>
Looking for how the device [communicates](../docs/PROTOCOL.md)?

> [!NOTE]
> I am not affiliated with HyperX, nor do I have a qualifying degree to say for certain I know what I'm doing.<br>
> The following is my own interpretation of the commands.<br>
> Please take this with a pinch of salt if you are to use this as a reference.

# Command List
|         **Command**        | **Value** | **Description**                                                 | **Parameter/Response Type**  | **Read/Write** |
|:--------------------------:|:---------:|-----------------------------------------------------------------|------------------------------|----------------|
|       [Headset Status](#headset-status)       |     1     | Finds the current status of the headset.                        | Boolean (no pattern)         | Read           |
|        [Battery Level](#battery-level)       |     2     | Finds the current battery level of the headset.                 | Range from 1 to 100          | Read           |
|       [Charging Status](#charging-status)      |     3     | Finds the current charging status of the headset.               | Boolean (default)            | Read           |
|      [Microphone Status](#microphone-status)     |     5     | Finds the current microphone status of the headset.             | Boolean (inverted)           | Read           |
|   [Get Auto-Shutdown Time](#get-auto-shutdown-time)   |     7     | Finds the current auto-shutdown time of the headset in minutes. | Range from 0-255 (byte-size) | Read           |
|   [Set Auto-Shutdown Time](#set-auto-shutdown-time)   |     34    | Sets the desired auto-shutdown time of the headset in minutes.  | Range from 0-255 (byte-size) | Write          |
|     [Get Sidetone Status](#get-sidetone-status)    |     6     | Finds the current sidetone status of the headset.               | Boolean (default)            | Read           |
|     [Set Sidetone Status](#set-sidetone-status)    |     33    | Sets the desired sidetone status of the headset.                | Boolean (default)            | Write          |
|     [Get Sidetone Volume](#get-sidetone-volume)    |     11    | Finds the current sidetone volume of the headset.               | Range from 0-255 (byte-size) | Read           |
|     [Set Sidetone Volume](#set-sidetone-volume)    |     35    | Sets the desired sidetone volume of the headset.                | Range from 0-255 (byte-size) | Write          |
|    [Get Noise Gate Status](#get-noise-gate-status)   |     13    | Finds the current noise gate status of the headset.             | Boolean (inverted)           | Read           |
|    [Set Noise Gate Status](#set-noise-gate-status)   |     33    | Sets the desired noise gate status of the headset. (in theory)  | Boolean (inverted)           | Write          |

## Headset Status
**Description**: Finds the current status of the headset.</br>
**Command Value**: `1` or `0x01`</br>
**Requesting Command Structure**: `0x06, 0xFF, 0xBB, 0x01`</br>
**Response**: Boolean (no pattern)
- The command returns a custom boolean response.
    - Valid response values:
        - `1` or `0x01`: `on`
        - `4` or `0x04`: `on`
        - `3` or `0x03`: `off`
</br>

**Read/Write**: Read-only</br>
**Response Example(s)**: `0x06, 0xFF, 0xBB, 0x01, 0x04` (device status is on), `0x06, 0xFF, 0xBB, 0x01, 0x03` (device status is off)

## Battery Level
**Description**: Finds the current battery level of the headset.</br>
**Command Value**: `2` or `0x02`</br>
**Requesting Command Structure**: `0x06, 0xFF, 0xBB, 0x02`</br>
**Response**: Range between 1-100</br>
- The battery level is found at the 8th position of the response, as seen the example below with a level of `31` or `0x1F`.
- The battery level can only be valid between `1` to `100`, otherwise the device is either dead or reporting an invalid battery level.

**Read/Write**: Read-only</br>
**Response Example(s)**: `0x06, 0xFF, 0xBB, 0x02, 0x00, 0xE, 0x98, 0x1F` (hexadecimal), `6, 255, 187, 2, 0, 14, 152, 31` (decimal)</br>

## Charging Status
**Description**: Finds the current charging status of the headset.</br>
**Command Value**: `3` or `0x03`</br>
**Requesting Command Structure**: `0x06, 0xFF, 0xBB, 0x03`</br>
**Response**: Boolean (default)</br>
- Valid response values:
    - `1` or `0x01`: `charging`
    - `0` or `0x00`: `not charging`

**Read/Write**: Read-only</br>
**Response Example(s)**: `0x06, 0xFF, 0xBB, 0x03, 0x01` (charging status is true), `0x06, 0xFF, 0xBB, 0x03, 0x00` (charging status is false)

## Microphone Status
**Description**: Finds the current microphone status of the headset.</br>
**Command Value**: `5` or `0x05`</br>
**Requesting Command Structure**: `0x06, 0xFF, 0xBB, 0x05`</br>
**Response**: Boolean (inverted)</br>
- Valid response values:
    - `0` or `0x00`: `on`
    - `1` or `0x01`: `off`

**Read/Write**: Read-only; but can be modified by rotating the microphone up and down.</br>
**Response Example(s)**: `0x06, 0xFF, 0xBB, 0x05, 0x00` (microphone status is on), `0x06, 0xFF, 0xBB, 0x05, 0x01` (microphone status is off)

## Get Auto-Shutdown Time
**Description**: Finds the current auto-shutdown time of the headset in minutes (assumed).</br>
**Command Value**: `7` or `0x07`</br>
**Requesting Command Structure**: `0x06, 0xFF, 0xBB, 0x07`</br>
**Response**: Range from 0-255 (byte-size)</br>
- Valid response values:
    - Any value between `0` and `255`

**Read/Write**: Read-only</br>
**Response Example(s)**: `0x06, 0xFF, 0xBB, 0x07, 0x14` or `6, 255, 187, 7, 20` (shutdown time is `20` or `0x14`)

## Set Auto-Shutdown Time
**Description**: Sets the desired auto-shutdown time of the headset in minutes (assumed).</br>
**Command Value**: `34` or `0x22`</br>
**Requesting Command Structure**: `0x06, 0xFF, 0xBB, 0x22, TIME`</br>
**Parameter/Response Types**: Range from 0-255 (byte-size)</br>
- Valid response values:
    - Any value between `0` and `255`

**Read/Write**: Write</br>
**Response Example(s)**: `0x06, 0xFF, 0xBB, 0x22, TIME` or `6, 255, 187, 7, TIME` (shutdown in `TIME` minutes)

## Get Sidetone Status
**Description**: Finds the current sidetone status of the headset.</br>
**Command Value**: `6` or `0x06`</br>
**Requesting Command Structure**: `0x06, 0xFF, 0xBB, 0x06`</br>
**Response**: Boolean (default)</br>
- Valid response values:
    - `1` or `0x01`: `on`
    - `0` or `0x00`: `off`

**Read/Write**: Read-only</br>
**Response Example(s)**: `0x06, 0xFF, 0xBB, 0x06, 0x00` (sidetone status is off)

## Set Sidetone Status
**Description**:  Sets the desired sidetone status of the headset.</br>
**Command Value**: `33` or `0x21`</br>
**Requesting Command Structure**: `0x06, 0xFF, 0xBB, 0x21, STATUS`</br>
**Parameter/Response Types**: Boolean (default)</br>
- Valid response values:
    - `1` or `0x01`: `on`
    - `0` or `0x00`: `off`

**Read/Write**: Write</br>
**Response Example(s)**: `0x06, 0xFF, 0xBB, 0x21, STATUS` (sidetone status defined by `STATUS`)
> This command value is **assumed** to execute both a Noise Gate *and* Sidetone Status command.</br>

## Get Sidetone Volume
**Description**: Finds the current sidetone status of the headset.</br>
**Command Value**: `11` or `0x0B`</br>
**Requesting Command Structure**: `0x06, 0xFF, 0xBB, 0x0B`</br>
**Response Type**: Range from 0-255 (byte-size)</br>
- Valid response values:
    - Any value between `0` and `255`

**Read/Write**: Read-only</br>
**Response Example(s)**: `0x06, 0xFF, 0xBB, 0x0B, 0x0A` or `6, 255, 187, 11, 10` (sidetone volume is `10` or `0x0A`)

## Set Sidetone Volume
**Description**:  Sets the desired sidetone volume of the headset.</br>
**Command Value**: `35` or `0x23`</br>
**Requesting Command Structure**: `0x06, 0xFF, 0xBB, 0x23, VOLUME`</br>
**Parameter/Response Types**: Range from 0-255 (byte-size)</br>
- Valid response values:
    - Any value between `0` and `255`

**Read/Write**: Write</br>
**Response Example(s)**: `0x06, 0xFF, 0xBB, 0x23, VOLUME` (sidetone volume defined by `VOLUME`)

## Get Noise Gate Status
**Description**: Finds the current noise gate status of the headset.</br>
**Command Value**: `13` or `0x0D`</br>
**Requesting Command Structure**: `0x06, 0xFF, 0xBB, 0x0D`</br>
**Response Type**: Boolean (inverted)</br>
- Valid response values:
    - `0` or `0x00`: `on`
    - `1` or `0x01`: `off`

**Read/Write**: Read-only</br>
**Response Example(s)**: `0x06, 0xFF, 0xBB, 0x0D, 0x01` (noise gate is off)

## Set Noise Gate Status
**Description**:  Sets the desired noise gate status of the headset.</br>
**Command Value**: `33` or `0x21`</br>
**Requesting Command Structure**: `0x06, 0xFF, 0xBB, 0x21, STATUS`</br>
**Parameter/Response Types**: Boolean (inverted)</br>
- Valid response values:
    - `0` or `0x00`: `on`
    - `1` or `0x01`: `off`

**Read/Write**: Write</br>
**Response Example(s)**: `0x06, 0xFF, 0xBB, 0x21, STATUS` (noise gate status defined by `STATUS`)</br>
> This command value is **assumed** to execute both a Noise Gate *and* Sidetone Status command.</br>