# T3000 Update Version webhook

### Run on Linux

`SECRET_KEY="your_secret_key" FILE_PATH="ProductPath.ini" cargo run`

#### As a service:

```ini
[Unit]
Description=Rust-based T3000 INI version updater webhook
After=network.target

[Service]
User=root
WorkingDirectory=/var/www/t3-version-updater
Environment=PORT=3024
Environment=SECRET_KEY="your_secret_key"
Environment=FILE_PATH="/var/www/html/ftp/firmware/ProductPath.ini"
ExecStart=/var/www/t3-version-updater/target/release/t3-update-version
Restart=always

[Install]
WantedBy=multi-user.target
```

### Run on Windows powershell

`$Env:SECRET_KEY="your_secret_key";$Env:FILE_PATH="ProductPath.ini";$Env:PORT=3000; cargo run`


### Example request

`https://t3-version-updater.bravocontrols.com/update-version`

`Method: Post`

`Header: X-Secret-Key: your_secret_key`

`body: {"version": "20230804", "url": "http://yoururl"}`
