# Protocol Documentation
This documentation will try to explain the protocol between the Cloud Stinger 2 Wireless and the computer as much as possible.\
Looking for the list of [commands](../docs/COMMANDS.md)?

> [!NOTE]
> I am not affiliated with HyperX, nor do I have a qualifying degree to say for certain I know what I'm doing.\
> The following is my own interpretation of how the commands are structured.\
> Please take this with a pinch of salt if you are to use this as a reference.

## Base Command
The default structure for each HID packet is:

``[0x06, 0xFF, 0xBB, COMMAND]`` or ``[0x06, 0xFF, 0xBB, COMMAND, PARAMETER]``

|  Byte  |  Value  |  Description  |
|:------:|:-------:|:-------------:|
|0       |0x06     |Report ID      |
|1       |0xFF     |Fixed Value    |
|2       |0xBB     |Fixed Value    |
|3       |COMMAND  |Request/Response|
|4       |PARAMETER|Request/Response|

``0x06`` is the Report ID.\
``0xFF`` and ``0xBB`` are fixed values.\
``COMMAND`` is the type of command being sent/received as an integer or as hexadecimal.\
``PARAMETER`` is the parameter being sent/received as an integer or as hexadecimal.

## Sending a Request
You can send a request by sending an output report to the device.\
Refer to the commands document to see how each command is structured.

## Parsing a Response
Usually, a response will have a parameter in them. For example:

``[0x06, 0xFF, 0xBB, 0x01, 0x03]``

In this case, ``0x01`` defines the ``COMMAND`` and ``0x03`` is the ``PARAMETER``.\
Some commands don't always use the same parameters, refer to the commands document to see how each command may respond with.
