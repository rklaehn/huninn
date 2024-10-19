# Munin

A minimal monitoring and control tool for a number of linux or windows PCs, using [iroh](https://github.com/n0-computer/iroh) for connectivity. This can also serve as a starting point for running an iroh-net server as a linux daemon or as as a windows service.

[Muninn](https://en.wikipedia.org/wiki/Huginn_and_Muninn) is one of the ravens of odin that flies all over the world to bring information to the god odin.

Munin consists of two components, a *server* that gets installed as either a daemon on unix or a windows service on windows, and a cli *client* that acts as a controller. Each server is identified by an iroh node id. The cli can connect to node ids and execute commands to monitor and control the servers. Servers will only accept connections from a configurable set of allowed node ids.

- munin-cli: controller
- munin-daemon: daemon on the machines to be supervised
- munin-proto: common protocol between cli and daemon

## Why?

I wrote this to be able to monitor what my kids do on their computer. I don't want to see their screen, just have a rough idea what they are doing and stop them from playing minecraft or elden ring at 3am when they have school the next day.

There are many extremely powerful solutions for monitoring and control of fleets of computers. However, many of them are very heavy and/or tied to one particular platform. Also, they frequently have a cloud component for management. I needed a lightweight and cross-platform solution that works without a cloud component even if the devices to be monitored are in multiple networks. Also, being a programmer, I found it more fun to write one myself than to figure out the best one to use.

## Installation

First install and run the cli. It will give you a list of commands and tell you its own node id (Ed25519 public key):

```
> munin
I am 2avprmfdzxtokjdomtebo3caylrcefr6c2iciirmwdwglro6ja2a
Usage: munin <COMMAND>

Commands:
...
```

Next, build the daemon or service. You can configure the allowed node ids, but you can also bake in a set of hardcoded allowed node ids. The latter is useful when installing the daemon on multiple machines.

```
MUNIN_ALLOWED_NODES=2avprmfdzxtokjdomtebo3caylrcefr6c2iciirmwdwglro6ja2a cargo build -p munin-daemon
```

Third, install the service. The service binary munin-service.exe is both a windows service and an installer for itself.

```
> munin-service.exe install   
> munin-service.exe start  
Public key: qblg7tefz6jek3hynvdgvjchlj4zqyhizn67ji2fm4ko5ro7wbsq
Allowed nodes                                                                                                   
  2avprmfdzxtokjdomtebo3caylrcefr6c2iciirmwdwglro6ja2a                                                          
Service config path: "C:\\WINDOWS\\system32\\config\\systemprofile\\AppData\\Roaming\\munin-daemon\\config.toml"
```

Now the service is installed as an autostart service and running. You can now remote control this node from munin-cli:

```
> munin list-tasks qblg7tefz6jek3hynvdgvjchlj4zqyhizn67ji2fm4ko5ro7wbsq
72: Secure System
8396: svchost.exe
1224: svchost.exe
2272: svchost.exe
...
```

Addressing the node by node id can get tedious, so you can also define an alias:

```
> munin add-node --name minipc qblg7tefz6jek3hynvdgvjchlj4zqyhizn67ji2fm4ko5ro7wbsq
> munin list-tasks minipc
72: Secure System
8396: svchost.exe
1224: svchost.exe
2272: svchost.exe
...
```
