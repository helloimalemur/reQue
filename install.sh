#!/bin/bash
##  if service exists stop it
if [[ -f /etc/systemd/system/reque.service ]]; then systemctl stop reque; else echo "na"; fi


## erase install folder and recreate it
rm -rf /var/lib/reque/
mkdir /var/lib/reque/
mkdir /var/lib/reque/output/
SERVICE_USER=$(cat config/Settings.toml | grep service_user | cut -d ' ' -f 3 | sed 's/\"//g')
chown -R "$SERVICE_USER":"$SERVICE_USER" /var/lib/reque/

## run bulid
cargo build --release

## copy binary and host_list.txt to install dir
cp target/release/reque /var/lib/reque/reque
cp -r config/ /var/lib/reque/
cp -r run.sh /var/lib/reque/


## clean build to free up space taken
#cargo clean

## copy service file and reload systemd daemon
cp  reque.service /etc/systemd/system/reque.service
systemctl daemon-reload

## enable and start service
systemctl enable reque
systemctl start reque
