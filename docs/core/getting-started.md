[← Summary](./SUMMARY.md)

# Getting Started

## Requirements

- Windows 10+ / macOS 11+ / Linux
- An AWS account with Access Key ID and Secret Access Key
- IAM permissions for the services you want to use (S3, CloudFormation, Cost Explorer)

## Installation

### Download (recommended)

1. Go to [Releases](https://github.com/Andrezz64/easy-cloud/releases/latest)
2. Download the installer for your OS (`.msi` for Windows)
3. Run the installer and open Easy Cloud

### Build from source

```bash
# Clone
git clone https://github.com/Andrezz64/easy-cloud.git
cd easy-cloud

# Install dependencies
npm install

# Run in development
npm run tauri dev

# Build for production
npm run tauri build
```

**Prerequisites for building:**
- [Node.js](https://nodejs.org/) v18+
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri Prerequisites](https://tauri.app/start/prerequisites/)

## First Run

1. Open Easy Cloud
2. On the Dashboard, click **"Add AWS Account"**
3. Enter:
   - **Account Name** — a friendly label (e.g., "Production")
   - **Region** — your default AWS region (e.g., `us-east-1`)
   - **Access Key ID** — your IAM access key
   - **Secret Access Key** — your IAM secret key
4. Click **"Connect Account"** — Easy Cloud verifies credentials via STS
5. Once connected, the Dashboard shows your resources

## AWS Permissions

Easy Cloud uses the following AWS services. Your IAM user/role needs permissions for the services you want to use:

| Service | Minimum permissions |
|---------|-------------------|
| STS | `sts:GetCallerIdentity` |
| S3 | `s3:*` (or scoped to specific buckets) |
| CloudFormation | `cloudformation:*` |
| Cost Explorer | `ce:GetCostAndUsage`, `ce:GetCostForecast` |

### Example minimal IAM policy

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "sts:GetCallerIdentity",
        "s3:*",
        "cloudformation:*",
        "ce:GetCostAndUsage",
        "ce:GetCostForecast"
      ],
      "Resource": "*"
    }
  ]
}
```

## Security

- Credentials are stored **locally on your machine only**
- No data is sent to any external server
- All API calls go directly from your machine to AWS
- The app is fully open source — inspect the code anytime
