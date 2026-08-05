# Protocol Documentation
This documentation will try to explain the protocol between the Cloud Stinger 2 Wireless and the computer as much as possible.<br>
Looking for the list of [commands](../docs/COMMANDS.md)?

> [!NOTE]
> I am not affiliated with HyperX, nor do I have a qualifying degree to say for certain I know what I'm doing.<br>
> The following is my own interpretation of how the commands are structured.<br>
> Please take this with a pinch of salt if you are to use this as a reference.

## Base Command
The default structure for each packet is:

``[0x06, 0xFF, 0xBB, COMMAND]`` or ``[0x06, 0xFF, 0xBB, COMMAND, PARAMETER]``

|  Byte  |  Value  |  Description  |
|:------:|:-------:|:-------------:|
|0|0x06 (or 6)|Report ID|
|1|0xFF (or 255)|Fixed Value|
|2|0xBB (or 187)|Fixed Value|
|3|COMMAND|Request/Response|
|4|PARAMETER|Request/Response|

``COMMAND`` is the type of command being sent/received as an integer or as hexadecimal.<br>
``PARAMETER`` is the parameter being sent/received as an integer or as hexadecimal.

## Sending a Request
You can send a request by sending an output report to the device.<br>
Sending a command is pretty straightforward. Modify the base command as necessary to request/set as you please.<br>
For example, to get the microphone status, send:

``[0x06, 0xFF, 0xBB, 0x05]``.

If you want to set something, for example the sidetone volume, you would modify the fourth byte to the volume you need:

``[0x06, 0xFF, 0xBB, 0x23, VOLUME]``

## Parsing a Response
Usually, a response will have a parameter in them. For example:

``[0x06, 0xFF, 0xBB, 0x01, 0x03]``

In this case, ``0x01`` defines the ``COMMAND`` and ``0x03`` is the ``PARAMETER``.\
Some commands don't always use the same parameters, however, which makes parsing annoying.
