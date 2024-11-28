FROM debian:bookworm-20241111

WORKDIR /projects

RUN apt-get update

RUN apt-get install -y xterm htop vim curl git build-essential libxcursor1 libxrandr2 libxi6 libx11-xcb1 x11-apps netcat-traditional libgl1-mesa-dev

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
