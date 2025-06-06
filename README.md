# Hypixel Screentime

A screentime tool to beautifully visualise your time spent on Hypixel.

## Setup

1. The **Rust compiler** and **Cargo** are required to build Hypixel Screentime. ([install](https://www.rust-lang.org/tools/install))

```sh
cargo install --git https://github.com/siriusmart/hypixel-screentime
```

2. Create a configuration folder, for example `/home/yourname/.config/hypixel-screentime/`
3. Generate the configuration files with
```sh
CONFIG=/home/yourname/.config/hypixel-screentime hypixel-screentime
```
4. Edit the `master.json` configuration file.

```js
{
  "keys": [
    "your hypixel API key"
  ],
  "interval": 120, // interval between fetching user info, in seconds
  "port": 8010,    // port to bind to
  "expire": 70,    // delete records older than this, in days
  "discord_token": "DISCORD TOKEN", // keep this empty if you don't want to use the discord features
  "broadcast_channels": {           // channels to broadcast events to
      "1376901981276209172": {
          "online": "Online: {user}",
          "resumed": "Online: {user} (resumed session)",
          "offline": "Offline: {user} (played for {duration})",
          "users": [ // list of users whose event should be broadcasted to the channel
              "Siri",
              "Soup"
          ]
      },
      "1279889041399222384": {
          "online": "Online: {user}",
          "resumed": "Online: {user} (resumed session)",
          "offline": "Offline: {user} (played for {duration})",
          "users": [
              // ...
          ]
      }
  },
  "merge": 300, // if users logout and log back in with 300 seconds, merge with the previous record
  "users": [    // note that "name" does not have to be the user's actual username
      {
          "name": "Siri",
          "uuid": "2e9eb33e4bc44b189e6f4fae98258e3c"
      },
      {
          "name": "Soup",
          "uuid": "177fa057bb114169ad329cb3f1b15fec"
      },
      {
          "name": "odqnger",
          "uuid": "4c23e4cafac14684b63b228cba0658dd"
      },
      {
          "name": "Reddy",
          "uuid": "bee9b76c236141849d698cb0aa72ba7b"
      }
  ]
}
```
5. Run the command again to start the server.
```sh
CONFIG=/home/yourname/.config/hypixel-screentime hypixel-screentime
```
