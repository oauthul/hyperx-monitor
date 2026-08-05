# Command List + Documentation
This documentation will try to list and explain commands for the Cloud Stinger 2 Wireless.<br>
The documentation will be structured in decimal format for ease of writing.<br>
Looking for how the device [communicates](../docs/PROTOCOL.md)?

> [!NOTE]
> I am not affiliated with HyperX, nor do I have a qualifying degree to say for certain I know what I'm doing.<br>
> The following is my own interpretation of the commands.<br>
> Please take this with a pinch of salt if you are to use this as a reference.

# Command List
|         **Command**        | **Value** | **Description**                                                 | **Parameter/Response Type**  | **Read/Write** |
|:--------------------------:|:---------:|-----------------------------------------------------------------|------------------------------|----------------|
|       Headset Status       |     1     | Finds the current status of the headset.                        | Boolean (no pattern)         | Read           |
|        Battery Level       |     2     | Finds the current battery level of the headset.                 | Range from 1 to 100          | Read           |
|       Charging Status      |     3     | Finds the current charging status of the headset.               | Boolean (default)            | Read           |
|      Microphone Status     |     5     | Finds the current microphone status of the headset.             | Boolean (inverted)           | Read           |
| Auto-Shutdown Time _(Get)_ |     7     | Finds the current auto-shutdown time of the headset in minutes. | Range from 0-255 (byte-size) | Read           |
| Auto-Shutdown Time _(Set)_ |     34    | Sets the desired auto-shutdown time of the headset in minutes.  | Range from 0-255 (byte-size) | Write          |
|   Sidetone Status _(Get)_  |     6     | Finds the current sidetone status of the headset.               | Boolean (default)            | Read           |
|   Sidetone Status _(Set)_  |     33    | Sets the desired sidetone status of the headset.                | Boolean (default)            | Write          |
|   Sidetone Volume _(Get)_  |     11    | Finds the current sidetone volume of the headset.               | Range from 0-255 (byte-size) | Read           |
|   Sidetone Volume _(Set)_  |     35    | Sets the desired sidetone volume of the headset.                | Range from 0-255 (byte-size) | Write          |
|  Noise Gate Status _(Get)_ |     13    | Finds the current noise gate status of the headset.             | Boolean (inverted)           | Read           |
|  Noise Gate Status _(Set)_ |     33    | Sets the desired noise gate status of the headset. (in theory)  | Unknown                      | Write          |
