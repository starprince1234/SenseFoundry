# SenseFoundry

## 本地启动

项目不使用本地真实 `.env` 文件。先把仓库绑定到 Doppler 的同名项目和
`dev_personal` 配置，再由 Doppler 把变量直接注入 Docker Compose：

```bash
doppler setup
doppler run -- docker compose build
doppler run -- docker compose up -d
```

启动后可检查：

```bash
docker compose ps
curl http://localhost:8080/api/v1/health
curl http://localhost:8000/health
curl http://localhost:8001/health
```

手机连接 Windows 个人热点时，后端地址为
`http://192.168.137.1:8080/api/v1`。Android 调试包通过仓库内的 Gradle
wrapper 构建后，用
`adb install -r android/app/build/outputs/apk/debug/app-debug.apk` 安装。
首次使用 ADB 时，需要在手机上确认 USB 调试指纹。

## Minimum Requirements

- 16GB RAM
- 4 CPU cores
- 20GB free disk space
