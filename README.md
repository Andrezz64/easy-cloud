<div align="center">
  <img src="https://raw.githubusercontent.com/tauri-apps/tauri/HEAD/app-icon.png" width="100" />
  <h1>Easy Cloud</h1>
  <p><strong>A modern, ultra-fast all-in-one Cloud Management Tool.</strong></p>
  <p>Built with Vue 3, Vite, and Tauri (Rust).</p>
</div>

---

## 🌩️ About The Project

**Easy Cloud** is a desktop application designed to provide a seamless, lightning-fast experience for managing your entire AWS infrastructure (and beyond). While our first major module is a blazing-fast S3 client, the ultimate vision encompasses a wide range of cloud services, serving as a unified dashboard for your cloud ecosystem.

By leveraging **Tauri** and **Rust** in the backend, Easy Cloud operates with an incredibly low memory footprint and unparalleled performance. The frontend is powered by **Vue 3** and **Vite**, delivering a sleek, glassmorphism-inspired UI that feels premium and native.

### ✨ Features
- 🚀 **Ultra Lightweight:** Compiled to native binaries with a tiny memory footprint.
- 🏗️ **Infrastructure as Code (CloudFormation):** Deploy, validate, and monitor CloudFormation stacks directly from the app.
- 💰 **Billing & Cost Analysis:** Visual breakdowns of your AWS spend, month-over-month comparisons, and end-of-month forecasting.
- 📁 **Advanced S3 File Management:** Navigate, upload, download, move, copy, and delete files with a beautiful visual interface.
- ☁️ **S3 Compatible:** Works seamlessly with AWS S3 buckets.
- 🔐 **Multi-Account & STS Validation:** Securely switch between different AWS profiles/accounts with instant identity validation.
- 📝 **Built-in Editor:** View and edit text-based files directly from the cloud without downloading them first.
- ⚡ **Lightning Fast Uploads:** Supports multipart uploads for large files natively through Rust.
- 🔗 **Shareable Links:** Generate pre-signed temporary URLs for your private files in seconds.

---

## 🛠️ Tech Stack

- **Frontend:** [Vue 3](https://vuejs.org/) + [Vite](https://vitejs.dev/) + TypeScript
- **Backend:** [Rust](https://www.rust-lang.org/) + [Tauri](https://tauri.app/)
- **AWS SDK:** `aws-sdk-s3`, `aws-sdk-cloudformation` (Rust native)
- **Styling:** Vanilla CSS with custom design tokens (Glassmorphism + Dark Mode)

---

## 🚀 Getting Started

To get a local copy up and running, follow these simple steps.

### Prerequisites

You will need the following installed on your system:
- [Node.js](https://nodejs.org/) (v18 or higher)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri Prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites) (C++ build tools on Windows, Xcode on macOS, etc.)

### Installation

1. Clone the repository
   ```bash
   git clone https://github.com/your-username/easy-cloud.git
   cd easy-cloud
   ```

2. Install NPM packages
   ```bash
   npm install
   ```

3. Run the development server
   ```bash
   npm run tauri dev
   ```
   *The first run will take a few minutes as Cargo downloads and compiles the Rust dependencies.*

---

## 📦 Building for Production

To build the executable (e.g., `.exe` or `.msi` for Windows, `.app` for macOS):

```bash
npm run tauri build
```

You can find the generated installers and raw executables inside the `src-tauri/target/release/` directory.

---

## 🗺️ Roadmap

Check out our [Roadmap](docs/roadmap.md) to see planned features, including our goal to implement **Zero-Knowledge End-to-End Encryption (E2E)**!

---

## 📄 License

Distributed under the GNU Affero General Public License v3.0 (AGPL-3.0). See `LICENSE` for more information.
