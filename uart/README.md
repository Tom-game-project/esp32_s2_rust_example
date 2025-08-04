# UART

| esp32-s2 PIN | PL2303 |
| ------------ | ------ |
| 17           | RX     |
| 18           | TX     |
| g            | GND    |
| 3v3          | VCC    |

# How To Run

```sh
sudo dmesg | grep pl2303
```

```sh
cargo run 
```

```sh
screen /dev/ttyUSB1 115200
```

