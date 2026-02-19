# whyOS v0.0.2

**whyOS** is a tiny, unnecessary, meme-powered operating system written in Rust. It doesn’t have a real reason to exist — and that’s exactly why it does.

---

## 🤔 What even is this?

whyOS is a hobby operating system targeting `x86_64`, built from scratch in Rust. It boots, prints funny messages and runs a custom CLI that lets you type bizarre commands like `yeet` and `scream`.

It’s funny. It’s minimal. It’s educational (sort of). It’s… whyOS.

---

## ✨ Features

✅ VGA text mode output  
✅ Keyboard input handling  
✅ A simple CLI with commands  
✅ Shutdown and reboot support  
✅ Works in QEMU and (mostly) on real hardware

---

## 🧰 Requirements

Install required components with:

```bash
rustup override set nightly
rustup component add rust-src
cargo install bootimage
```

---

## 🔨 Build & Run

To build the bootable image:

```bash
cargo bootimage
```

To run in QEMU:

```bash
qemu-system-x86_64 -drive format=raw,file=target/x86_64-blog_os/debug/bootimage-why_os.bin
```

> You can also try it on real hardware and kinda works.

---

## 🧑‍💻 CLI Commands

| Command      | Description                                                                                                                    |
|--------------|--------------------------------------------------------------------------------------------------------------------------------|
| `hello`      | Prints "Hello World!"                                                                                                          |
| `yeet`       | Clears the screen.                                                                                                             |
| `scream`     | Echoes your message back to you.                                                                                               |
| `bye`        | Shutdown the system. (Currently not working on real hardware)                                                                  |
| `whyver`     | Shows the information about the current OS release on this system                                                              |
| `listcolors` | Lists the available colors for this system.                                                                                    |
| `setfg`      | Sets the foreground color (the text color) of the screen. The value must only be one of the ones shown in command `listcolors`.|
| `setfg`      | Sets the background color of the screen. The value must only be one of the ones shown in command `listcolors`.                 |
| `info`       | It explains what every command does.                                                                                           |

---

## 🤝 Contributing

You can help this project grow by:

- Submitting bug reports or issues
- Sending pull requests

---

## 📜 License

MIT
