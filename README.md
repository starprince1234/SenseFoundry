# SenseFoundry

SenseFoundry 是一套面向现代汉语辞书编纂的 AI 辅助系统。它把真实语料接入、电子卡片、已有义项匹配、未知用法发现、候选义项聚类、证据约束释义、真实例证筛选、专家审核、签名发布和 Android 离线同步组织为一条可追溯流程。

项目的完整产品与工程约束以 [`技术路线.md`](./技术路线.md) 为准。本 README 只描述仓库当前可运行能力、启动方式和已知边界，不把技术路线中的规划项冒充为已实现功能。

## 核心原则

- AI 只生成候选与草稿，不替代语言学专家。
- 用户投稿只进入待核验队列，不直接进入正式辞书。
- 释义生成必须受证据包约束，不得编造来源、日期、作者或例句。
- 正式内容必须经过双人复核，并绑定被审核内容的 SHA-256 摘要。
- Android 同步包必须同时通过摘要与 ECDSA P-256 签名校验；校验失败拒绝导入，不降级。
- 开发和生产密钥完全隔离，真实敏感信息只保存在 Doppler，不落入本地 `.env`。
- 原始语料、审核记录、模型版本、发布版本和回滚点必须保留可追溯关系。

## 当前能力

仓库当前包含以下可运行模块：

- Rust/Axum 模块化后端，已装配身份、投稿、来源核验、辞书导入、模型注册、审计、文本处理、卡片、搜索、义项匹配、义项发现、历时分析、例证、审核、发布与同步路由。
- PostgreSQL + pgvector 事务数据库与版本化迁移。
- OpenSearch 全文检索，宿主调试端口固定为 `9201`，避免与本机 cpolar 的 `9200` 冲突。
- MinIO S3 兼容对象存储。
- Keycloak OIDC 身份服务。
- Python 推理服务，包括目标词嵌入、MLM 替换、重排、聚类、稳定性评估和模型预热。
- OpenAI-compatible 嵌入和对话模型接入，以及证据约束 LLM 网关。
- SvelteKit 编纂管理端原型。
- Kotlin + Jetpack Compose Android 客户端，支持离线检索、义项详情、真实例证、语料投稿、签名增量同步与同步状态。

2026-07-20 在当前 `dev_personal` 配置和 V2307A 真机上完成过以下实测：

- `/v1/embeddings` 使用 `text-embedding-3-small` 返回 1536 维向量。
- `/v1/chat/completions` 使用 Doppler 配置的 `gpt-5.6-luna` 成功返回内容。
- 推理服务加载 `bert-base-chinese` 与 `BAAI/bge-reranker-base`，未启用静默回退。
- Docker Compose 中 PostgreSQL、OpenSearch、MinIO、Keycloak、inference、LLM gateway、backend、frontend 均通过健康检查。
- 发布门禁在双人批准前返回冲突；批准后生成稳定 delta、SHA-256 摘要和 P-256 DER 签名。
- V2307A（Android 16）通过 Windows 个人热点直连后端，并以 ADB streaming 方式安装 APK。
- WorkManager 真机同步成功，Room 中写入正式义项与例证，离线 SQL 查询返回同步内容。

这些结果是一次明确环境下的验收记录，不替代每次开发后的重新验证。

## 系统架构

```text
SvelteKit 编纂端 :15173 ─┐
                        ├─> Rust API :8080 ─> PostgreSQL/pgvector :5432
Android 客户端 ─────────┘         │          OpenSearch :9200(container)
                                  │          MinIO :9000
                                  │          Keycloak :8080(container)
                                  ├─> Inference :8000
                                  └─> LLM Gateway :8001 ─> OpenAI-compatible API

发布链路：双人审核 -> 内容摘要 -> P-256 签名 -> manifest/delta -> Android 验签 -> Room 事务
```

### 服务与端口

| 服务 | Compose 名称 | 宿主端口 | 热点可访问 | 说明 |
| :--- | :--- | :--- | :--- | :--- |
| Android/API 后端 | `backend` | `8080` | 是 | REST API，前缀 `/api/v1` |
| 编纂端 | `frontend` | `15173` | 是 | 宿主/热点端口；容器内部使用 `5173` |
| Keycloak | `keycloak` | `8180` | 是 | OIDC realm 与登录入口 |
| 推理服务 | `inference` | `8000` | 否 | 嵌入、MLM、重排、聚类、稳定性 |
| LLM 网关 | `llm-gateway` | `8001` | 否 | 证据约束释义草拟 |
| PostgreSQL | `postgres` | `15432` | 否 | 宿主调试端口；容器内部仍使用 `5432` |
| OpenSearch | `opensearch` | `9201` | 否 | 容器内部仍使用 `9200` |
| MinIO API/Console | `minio` | `9000/9001` | 否 | S3 API 与管理控制台 |

除 backend、frontend、Keycloak 外，基础设施和 AI 服务只绑定 `127.0.0.1`。热点侧只开放 Android 实际需要的入口。

## 环境要求

推荐开发环境：

- Windows 11 + Docker Desktop（Linux containers）。
- Doppler CLI 3.x，并已登录可访问 `sensefoundry` 项目。
- Git。
- 16 GB 以上内存、4 个以上 CPU 核心、20 GB 以上可用磁盘。
- Android 开发：JDK 17、Android SDK、Platform Tools、Build Tools 35.0.0。
- 真机联调：Android 设备开启开发者选项和 USB 调试。

首次加载中文 MLM 与 reranker 模型需要下载模型文件，耗时取决于网络和磁盘性能。模型缓存保存在 Docker named volume `model_cache`，正常重启不会重复下载。如果宿主网络暂时无法访问 Hugging Face，但缓存已完整包含两套指定模型，可在启动进程中显式设置 `HF_HUB_OFFLINE=1` 与 `TRANSFORMERS_OFFLINE=1`。服务仍会加载并真实执行同一模型；任一文件缺失时启动会失败，不会切换模型或生成替代结果。

## Doppler 与环境变量

本项目禁止创建真实 `.env`、`.env.local`、`.env.dev` 或其他本地密钥文件。

环境映射：

| 用途 | Doppler Project | Config |
| :--- | :--- | :--- |
| 本地开发与真机联调 | `sensefoundry` | `dev_personal` |
| 生产环境 | `sensefoundry` | `prd` |

`.env.example` 和 `.env.prod.example` 只用于说明字段结构，不能填入真实值。开发与生产必须使用不同的数据库凭据、API Key、Keycloak secret 和同步签名密钥对。

### 首次绑定

```powershell
doppler login
doppler setup
```

在交互界面选择：

```text
project: sensefoundry
config:  dev_personal
```

`.doppler.yaml` 已被 `.gitignore` 排除。自动化脚本仍显式传入 project/config，避免误用其他环境。

### 安全自检

以下脚本只打印 `configured`、URL origin/path 等非敏感状态，不打印密钥值：

```powershell
doppler run --project sensefoundry --config dev_personal -- `
  powershell -NoProfile -ExecutionPolicy Bypass `
  -File scripts/verify-doppler-env.ps1
```

必需字段包括：

- `DATABASE_URL`、`POSTGRES_PASSWORD`
- `EMBEDDING_API_URL`、`EMBEDDING_API_KEY`、`EMBEDDING_MODEL`
- `LLM_API_URL`、`LLM_API_KEY`、`LLM_MODEL`
- `OPENSEARCH_PASSWORD`
- `MINIO_ACCESS_KEY`、`MINIO_SECRET_KEY`
- `KEYCLOAK_CLIENT_SECRET`、`KEYCLOAK_ADMIN_PASSWORD`
- `SYNC_SIGNING_PRIVATE_KEY`、`SYNC_SIGNING_PUBLIC_KEY`

同步私钥格式为 P-256 PKCS#8 PEM，只注入 backend；公钥格式为 X.509 SubjectPublicKeyInfo PEM，由 Doppler 在 Android 构建时写入 `BuildConfig`。`prd` 必须单独生成密钥，保存时不要同步到 Development 或其他环境。

## Docker Desktop 启动

仓库提供 `scripts/doppler-compose.ps1`，用于在 Doppler 注入后校验关键变量，并把数据库连接主机规范化为 Compose 内部的 `postgres` 服务。

### 1. 验证 Compose 配置

```powershell
doppler run --project sensefoundry --config dev_personal -- `
  powershell -NoProfile -ExecutionPolicy Bypass `
  -File scripts/doppler-compose.ps1 config --quiet
```

### 2. 构建全部本地镜像

```powershell
doppler run --project sensefoundry --config dev_personal -- `
  powershell -NoProfile -ExecutionPolicy Bypass `
  -File scripts/doppler-compose.ps1 build
```

### 3. 后台启动

```powershell
doppler run --project sensefoundry --config dev_personal -- `
  powershell -NoProfile -ExecutionPolicy Bypass `
  -File scripts/doppler-compose.ps1 up --detach
```

Windows PowerShell 调用脚本时建议使用完整参数 `--detach`，避免把简写 `-d` 误解析为 PowerShell 参数。

### 4. 检查健康状态

由于 Compose 文件包含必需变量表达式，检查命令也应在 Doppler 注入下运行：

```powershell
doppler run --project sensefoundry --config dev_personal -- `
  powershell -NoProfile -Command "docker compose ps"
```

除一次性 `warmup` 容器应以成功状态退出外，长期服务应显示 `running (healthy)`。

基础探针：

```powershell
curl.exe http://127.0.0.1:8080/api/v1/health
curl.exe http://127.0.0.1:8000/health
curl.exe http://127.0.0.1:8001/health
curl.exe http://127.0.0.1:9201/_cluster/health
curl.exe http://127.0.0.1:15173
```

backend `/health` 会真实执行数据库 `SELECT 1`，不是固定字符串探针。

### 常用运维命令

```powershell
# 查看最近日志
doppler run --project sensefoundry --config dev_personal -- `
  powershell -NoProfile -Command "docker compose logs --tail 100 backend"

# 只重建 backend
doppler run --project sensefoundry --config dev_personal -- `
  powershell -NoProfile -ExecutionPolicy Bypass `
  -File scripts/doppler-compose.ps1 build backend

# 停止容器，保留 named volumes
doppler run --project sensefoundry --config dev_personal -- `
  powershell -NoProfile -ExecutionPolicy Bypass `
  -File scripts/doppler-compose.ps1 down
```

不要随意执行 `docker compose down -v`；`-v` 会删除 PostgreSQL、OpenSearch、MinIO 和模型缓存卷。

## AI 接口与验收

项目契约要求供应商提供完整 OpenAI-compatible 端点：

- `POST /v1/embeddings`
- `POST /v1/chat/completions`

不能用本地伪向量、固定文本、不同路径或较低规格模型替代。实际 URL、Key 与模型名全部由 Doppler 注入。

执行云端契约测试：

```powershell
doppler run --project sensefoundry --config dev_personal -- `
  powershell -NoProfile -ExecutionPolicy Bypass `
  -File scripts/verify-ai-services.ps1
```

脚本检查：

- 嵌入响应具有 OpenAI `data[].embedding` 结构。
- 向量非空并输出实际维度。
- chat completion 具有 `choices[0].message.content`。
- 终端不输出 API Key 或完整模型响应。

### 本地推理服务

主要接口：

| 方法 | 路径 | 功能 |
| :--- | :--- | :--- |
| `GET` | `/health` | 服务和模型状态 |
| `POST` | `/embed` | 目标词上下文表示 |
| `POST` | `/mlm-substitute` | 掩码语言模型替换候选 |
| `POST` | `/rerank` | BGE cross-encoder 重排 |
| `POST` | `/cluster` | 层次聚类 |
| `POST` | `/stability` | 重采样稳定性评估 |
| `POST` | `/models/warmup` | 显式加载 MLM 与 reranker |

推理失败必须显式报错；正式验收要求 `fallback_used=false`，不能把静默降级结果视为通过。

### LLM 网关

`POST http://127.0.0.1:8001/draft-definition` 接收：

- 字头与词性；
- 明确的 `evidence_ids`；
- 与这些 ID 一一对应的证据文本。

网关只把请求指定的证据送给模型，并返回实际使用的证据 ID。证据为空或引用不存在的证据 ID 会被拒绝。

## 审核、发布与 Android 同步

当前同步协议：

```text
POST /publication-preview
  -> 计算稳定 delta 字节与 SHA-256
POST /review-tasks
  -> 绑定 reviewed_content_hash 和至少两名不同审核者
POST /review-tasks/{id}/decide
  -> 两次 approve 后进入 COMPLETED
POST /publications
  -> 再次校验审核摘要，使用 P-256 私钥签名
GET /sync-manifests/latest/delta?last_sync_token=N
GET /editions/{id}/delta
  -> Android 检查同步令牌单调递增，下载、验摘要、验签、Room 事务导入
```

Android 不接受小于本地可信版本的 `sync_token`。如果服务端因状态丢失返回更旧令牌，客户端会保留已有 Room 数据和本地令牌，将任务标记为失败，并显示“服务端版本低于本地可信版本，已拒绝回退”。只有经过正式回滚审批并通过协议表达的新签名版本，才能替换现有版本；不能用令牌倒退冒充回滚。

delta 包稳定包含：

- `headword`
- `version_number`
- `senses`
- `examples`

独立验收完整链路：

```powershell
doppler run --project sensefoundry --config dev_personal -- `
  powershell -NoProfile -ExecutionPolicy Bypass `
  -File scripts/verify-publication-sync.ps1
```

该脚本会证明：

1. 双人批准前发布被阻止。
2. 两名不同审核者批准后允许发布。
3. manifest 包含新发布 edition。
4. 下载字节的 SHA-256 与 manifest 一致。
5. 独立 Python `cryptography` 实现能够验证 P-256/SHA-256 DER 签名。

脚本只输出验证状态，不输出私钥、公钥全文、签名或完整同步包。

## Android 客户端

Android 端严格使用技术路线指定的 Kotlin、Jetpack Compose、Room 与 WorkManager，`compileSdk`/`targetSdk` 为 35，JVM toolchain 为 17。

### 当前页面

- **辞典**：离线首页、中文搜索、加载/空/错误状态、义项结果卡片。
- **义项详情**：词性、正式释义、同步例证、版本摘要保护说明和投稿入口。
- **投稿**：文本/URL 两种投稿、字符限制、本机草稿、稳定投稿者 ID、权利与隐私确认、服务端回执。
- **书库**：本地版本/义项/例证统计、WorkManager 实时状态、上次同步时间、手动同步、安全与隐私说明。

网络失败时保留已有离线辞典和投稿草稿；WorkManager 使用指数退避重试。摘要或签名失败属于完整性错误，会直接拒绝导入并显示失败原因，不使用未验证内容。服务端同步令牌回退同样会被拒绝，避免 backend 状态丢失时把客户端已有可信版本误标为旧版本。

### Windows 热点网络

当前调试拓扑约定：

```text
Windows 热点接口：192.168.137.1/24
Android API 地址：http://192.168.137.1:8080/api/v1
```

手机连接电脑个人热点后检查：

```powershell
Get-NetIPAddress -AddressFamily IPv4 | `
  Where-Object IPAddress -eq '192.168.137.1'

F:\Android\SDK\platform-tools\adb.exe shell ip -4 addr show wlan0
F:\Android\SDK\platform-tools\adb.exe shell `
  curl -fsS http://192.168.137.1:8080/api/v1/health
```

如果电脑热点地址不是 `192.168.137.1`，必须同步修改 `android/app/build.gradle.kts` 中的 `API_BASE_URL` 以及 Compose 的热点端口绑定；不要用 ADB reverse 冒充热点直连验收。

### 构建 APK

PowerShell 示例：

```powershell
$env:JAVA_HOME = 'D:\java'
$env:ANDROID_HOME = 'F:\Android\SDK'
$env:ANDROID_SDK_ROOT = 'F:\Android\SDK'

doppler run --project sensefoundry --config dev_personal -- `
  android\gradlew.bat -p android --no-daemon clean assembleDebug
```

Android 构建强制要求 `SYNC_SIGNING_PUBLIC_KEY` 来自 Doppler；未注入、为空或仍为占位值时 Gradle 会失败。APK 输出：

```text
android/app/build/outputs/apk/debug/app-debug.apk
```

### ADB streaming 安装

```powershell
$adb = 'F:\Android\SDK\platform-tools\adb.exe'
& $adb devices -l
& $adb install --streaming -r `
  android\app\build\outputs\apk\debug\app-debug.apk
```

预期输出：

```text
Performing Streamed Install
Success
```

首次连接时需要在手机上确认 USB 调试指纹和安装授权。安装后打开“书库”并点击“同步最新版本”，再回到“辞典”执行离线查询。

## 开发与验证

### Rust

本机没有 Cargo 时可复用后端 Docker builder：

```powershell
docker build --target builder -f backend/Dockerfile `
  -t sensefoundry-backend-builder .
docker run --rm sensefoundry-backend-builder cargo test --workspace
```

### Python

```powershell
python -m pytest
python -m compileall inference llm-gateway scripts
```

### SvelteKit

```powershell
Set-Location frontend
npm ci
npm run check
npm run build
npm test
```

### Android

```powershell
doppler run --project sensefoundry --config dev_personal -- `
  android\gradlew.bat -p android --no-daemon `
  testDebugUnitTest lintDebug assembleDebug
```

### 提交前检查

```powershell
git diff --check
git status --short
git check-ignore .omo android/.gradle android/local.properties `
  android/app/build frontend/build
```

还应确认：

- 仓库中不存在真实 `.env`、PEM 私钥、token 或 API Key。
- `.omo/`、Gradle 缓存、APK、Android build 和 `frontend/build` 未被跟踪。
- Compose 长期服务全部 healthy。
- AI 云端契约、推理接口和发布签名链路均重新执行。
- Android APK 使用 Doppler 公钥重新构建并流式覆盖安装。

## 目录结构

```text
SenseFoundry/
├── android/               Kotlin + Compose + Room + WorkManager 客户端
├── backend/               Rust 模块化后端与 API server
├── corpus-tools/          语料处理辅助工具
├── frontend/              SvelteKit 编纂端
├── inference/             Python 模型推理服务
├── infra/                 PostgreSQL 与 Keycloak 初始化配置
├── llm-gateway/           OpenAI-compatible 证据约束网关
├── migrations/            PostgreSQL 版本化迁移
├── openapi/               API 契约材料
├── scripts/               Doppler、Compose 与端到端验收脚本
├── compose.yaml           本地完整容器编排
├── .env.example           开发字段骨架，不含真实值
├── .env.prod.example      生产字段骨架，不含真实值
└── 技术路线.md             产品、算法、合规与工程权威路线
```

## 常见问题

### OpenSearch 无法绑定 9200

本机 cpolar 可能占用 `9200`。仓库已经把 OpenSearch 的宿主端口映射为：

```text
127.0.0.1:9201 -> container:9200
```

不要停止 cpolar，也不要把容器内部地址改成 `9201`。backend 在 Compose 网络内仍应访问 `http://opensearch:9200`。

### Keycloak 一直 unhealthy

Keycloak 24 的当前容器在内部监听 `8080`。健康检查必须检查容器内 `127.0.0.1:8080`，不能误用管理端口 `9000`。

### frontend 宿主可访问但 unhealthy

Vite 监听 IPv4 时，容器健康检查应使用 `http://127.0.0.1:5173`。部分 Alpine 环境中的 `localhost` 会优先解析为 IPv6，从而得到 connection refused。

### Android 找不到 SDK 或 Build Tools

确认：

```powershell
Test-Path F:\Android\SDK\platform-tools\adb.exe
Get-ChildItem F:\Android\SDK\build-tools
java -version
```

项目要求 JDK 17 和 Build Tools 35.0.0，不要通过降低 `compileSdk` 或 `targetSdk` 规避配置问题。

### PostgreSQL 端口 5432 被 Windows 拒绝绑定

Windows 个人热点、Hyper-V 或 WSL 可能动态保留包含 `5432` 的端口段，即使 `netstat` 没有进程监听也会报 `ports are not available`。仓库使用 `127.0.0.1:15432` 作为 PostgreSQL 宿主调试端口；容器内部和 backend 始终使用标准的 `postgres:5432`。`scripts/doppler-compose.ps1` 会自动把 Doppler 中面向宿主的 `DATABASE_URL` 规范化为容器内地址。

### 手机无法访问电脑

依次检查：

1. 手机 `wlan0` 与电脑热点接口都在 `192.168.137.0/24`。
2. backend 已绑定 `192.168.137.1:8080`。
3. 手机系统 `curl` 能访问 `/api/v1/health`。
4. Windows 防火墙没有阻止 Docker Desktop 的热点接口流量。
5. VPN/代理没有把本地网段错误路由到隧道。

### 同步显示成功但搜索无内容

先确认服务端存在已发布 edition。当前 publication/review/sync 状态保存在 backend 进程内存中，backend 重启后演示发布记录会清空，需要重新运行 `scripts/verify-publication-sync.ps1` 创建已审核版本。此时 Android 会拒绝服务端从 `1` 回退到 `0`，不会覆盖本地可信内容。正式环境必须把审核、发布、manifest 与 delta 元数据迁移到 PostgreSQL/对象存储，不能依赖内存状态。

### 为什么不能创建 `.env`

`.env` 会把真实凭据长期留在开发机、编辑器历史、备份和误提交表面。项目统一使用：

```powershell
doppler run --project sensefoundry --config dev_personal -- <command>
```

Docker Compose 的 `environment` 只引用宿主进程变量，不包含真实密钥。

## 当前已知边界

以下内容仍属于需要继续工程化的部分，不能视为正式生产能力：

- Review、Publication 与 Sync 的业务状态目前是进程内存态，backend 重启会丢失演示发布记录。
- Android 离线 delta 当前包含义项和例证，但尚未包含完整来源书目与授权可见性，因此客户端明确显示来源未下发。
- Android 已具备投稿回执，但完整投稿历史、反馈工单和身份登录页面仍需继续实现。
- SvelteKit 管理端是编纂原型，部分定义修订与发布页面仍需和最新后端契约统一后才能称为完整业务闭环。
- 正式生产还需要持久化发布工件、不可变审计、备份恢复、监控告警、权限验收和独立生产签名密钥。
- `技术路线.md` 中的开发前决策门、语料授权、参考辞书许可和人工评测集仍需项目负责人确认。

这些边界不应通过 mock、固定响应、弱化审核、跳过验签、降级模型或降低 Android SDK 来掩盖。

## 安全提示

- 不要把 `doppler secrets` 的完整输出粘贴到 Issue、日志或聊天记录。
- 不要在 Dockerfile、Compose、Gradle、README 或测试数据中写入真实 token、密码和私钥。
- 不要把开发签名私钥复制到 `prd`；生产应生成独立密钥并执行轮换与吊销流程。
- 不要把未经版权审核的投稿或原文公开到 Android delta。
- Android 签名验证失败时必须调查服务端工件、密钥和传输完整性，不能关闭校验。

## 许可证与数据合规

仓库代码许可证、参考辞书许可、语料存储/训练/展示授权和例证引用规则需要分别确认。拥有代码使用权不等于拥有语料、辞书内容或模型训练数据的再分发权。正式导入任何数据前，请完成 `技术路线.md` 中的法律、版权、个人信息和决策门审查。
