xhost + `ipconfig getifaddr en0`
APP_DISPLAY=`ipconfig getifaddr en0` docker compose up
xhost - `ipconfig getifaddr en0`
