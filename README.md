# Boot Cam Treiber

### Build everything and package it for the update service
```bash
cd sign_update
cargo build --release
```

## Build Backend with Docker:
```bash
cd backend
docker build -t pi-backend . ; docker create --name temp pi-backend ; docker cp temp:/backend ./backend ; docker rm temp
```
```bash
cd update_service
docker build -t pi-update_service . ; docker create --name temp pi-update_service ; docker cp temp:/update_service ./update_service ; docker rm temp
```

## Build Frontend with npm
```bash
cd frontend
npm run build
```

## Pi Setup

- Activate i2c in `raspi-config` 

### Disable Swap 
```bash 
sudo nano /etc/rpi/swap.conf
```
```
[Main]
Mechanism=none
```

### Install dependencies:
```bash
sudo apt update
sudo apt install libudev-dev pkg-config ffmpeg -y
wget https://github.com/bluenviron/mediamtx/releases/download/v1.17.0/mediamtx_v1.17.0_linux_arm64.tar.gz
tar -xvzf mediamtx_v1.17.0_linux_arm64.tar.gz
mkdir tmp
```


### Disable WIFI power savings mode
```bash
sudo mkdir -p /etc/NetworkManager/conf.d
sudo nano /etc/NetworkManager/conf.d/wifi-powersave.conf
```

Copy in File
```
[connection]
wifi.powersave = 2
```

```bash
sudo systemctl restart NetworkManager
```

```bash
nano mediamtx.yml
```

At the end of the file, add:
```
paths:
  stream:
```
For hotspot:
```
webrtcIPsFromInterfaces: false
webrtcIPsFromInterfacesList: []
webrtcAdditionalHosts: [192.168.50.1]
```

```bash
sudo nano /etc/systemd/system/mediamtx.service
```

Insert the following content:
```
[Unit]
Description=MediaMTX Service
After=network.target

[Service]
Type=simple
WorkingDirectory=/home/pi/
ExecStart=/home/pi/mediamtx
Restart=always
RestartSec=5
User=pi

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable mediamtx.service
sudo systemctl start mediamtx.service
```

Stream accessible at: `https://<your-pi-ip>:8889/stream/`

### Update service Setup
```bash
sudo nano /etc/systemd/system/update_service.service
```

```bash
[Unit]
Description=Update Service
After=network.target

[Service]
Type=simple
WorkingDirectory=/home/pi/update_service
ExecStart=/home/pi/update_service/update_service
Restart=always
RestartSec=5
User=root
Environment=PATH=/usr/local/sbin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable update_service.service
sudo systemctl start update_service.service
```

### Backend Setup
```bash
sudo nano /etc/systemd/system/backend.service
```

```bash
[Unit]
Description=Backend Service
After=network.target

[Service]
Type=simple
WorkingDirectory=/home/pi/treiber
ExecStart=/home/pi/treiber/server
Restart=always
RestartSec=5
User=root
Environment=PATH=/usr/local/sbin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable backend.service
sudo systemctl start backend.service
```


## Optimize boot (after first treiber run)
```bash
sudo systemctl disable --now bluetooth.service
sudo systemctl disable --now NetworkManager-wait-online.service

sudo touch /etc/cloud/cloud-init.disabled
sudo systemctl disable --now cloud-init-local.service
sudo systemctl disable --now cloud-config.service
sudo systemctl disable --now cloud-final.service

sudo nmcli connection modify "netplan-wlan0-Internetz 2.4 GHz" connection.autoconnect no
sudo nmcli connection modify "netplan-eth0" connection.autoconnect no

sudo nmcli connection modify treiber_hotspot connection.autoconnect yes
sudo nmcli connection modify treiber_hotspot connection.autoconnect-priority 100
```