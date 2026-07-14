[← Back to README](../README.md)

# 📖 Summary — Easy Cloud Docs

> Find what you need fast. Every link goes straight to the topic.

---

## 🚀 Getting Started

- [Installation (Download)](./getting-started.md#installation)
- [Build from Source](./getting-started.md#build-from-source)
- [First Run — connect AWS account](./getting-started.md#first-run)
- [Required IAM Permissions](./getting-started.md#aws-permissions)
- [Minimal IAM Policy (JSON)](./getting-started.md#example-minimal-iam-policy)
- [Security — where credentials are stored](./getting-started.md#security)

---

## 🪣 S3 File Manager

- [Navigation (breadcrumb, folders)](../guides/s3.md#navigation)
- [Simple & multipart upload (>10MB)](../guides/s3.md#uploading-files)
- [Create empty file](../guides/s3.md#create-empty-file)
- [Download files](../guides/s3.md#file-actions)
- [Preview (text, code, images)](../guides/s3.md#file-actions)
- [Built-in code editor (Ctrl+S saves)](../guides/s3.md#code-editor)
- [Copy / Move objects](../guides/s3.md#via-right-click-context-menu)
- [Rename objects](../guides/s3.md#via-right-click-context-menu)
- [Delete (single and batch)](../guides/s3.md#via-right-click-context-menu)
- [Presigned URLs — temporary share links](../guides/s3.md#presigned-urls-share-links)
- [Copy bucket URL](../guides/s3.md#copy-bucket-url)
- [Context menu (right-click)](../guides/s3.md#via-right-click-context-menu)
- [Permissions — Bucket Policy](../guides/s3.md#bucket-properties)
- [Properties — Versioning, Encryption](../guides/s3.md#bucket-properties)
- [Tags](../guides/s3.md#bucket-properties)

---

## 📦 CloudFormation

- [YAML Editor](../guides/cloudformation.md#using-the-yaml-editor)
- [Visual Builder (S3, SQS, SNS)](../guides/cloudformation.md#using-the-visual-builder)
- [Validate template before deploy](../guides/cloudformation.md#template-validation)
- [Deploy (create stack)](../guides/cloudformation.md#creating-a-stack)
- [Update stack](../guides/cloudformation.md#actions)
- [Delete stack](../guides/cloudformation.md#actions)
- [View deployed template](../guides/cloudformation.md#tabs)
- [Load template into editor](../guides/cloudformation.md#tabs)
- [Estimate cost (AWS Calculator)](../guides/cloudformation.md#tabs)
- [Drift Detection — detect manual changes](../guides/cloudformation.md#drift-detection)
- [Change Sets — preview updates](../guides/cloudformation.md#change-sets)
- [Execute / delete change sets](../guides/cloudformation.md#change-sets)
- [Stack resources (resource list)](../guides/cloudformation.md#tabs)
- [Stack outputs (URLs, ARNs)](../guides/cloudformation.md#tabs)
- [Events & history](../guides/cloudformation.md#tabs)
- [Real-time operation tracker](../guides/cloudformation.md#operation-tracker)
- [Context menu (right-click on stack)](../guides/cloudformation.md#context-menus)
- [Cross-stack Exports / Imports](../guides/cloudformation.md#tabs)

---

## 💰 Billing & Costs

- [⚠️ Cost Explorer API pricing ($0.01/request)](../guides/billing.md#important-api-costs)
- [2-hour cache (per account)](../guides/billing.md#cache-indicator)
- [Summary cards — current month, previous, forecast](../guides/billing.md#dashboard-summary)
- [Cost over time chart](../guides/billing.md#cost-over-time-chart)
- [Period filter (7d, 1m, 3m, 6m, 1y, custom)](../guides/billing.md#period-filter)
- [Top services by cost](../guides/billing.md#top-services)
- [Usage breakdown — actual usage with rates](../guides/billing.md#resource-usage)
- [Free tier detection](../guides/billing.md#resource-usage)
- [Full services table](../guides/billing.md#all-services-table)

---

## 🏗️ Architecture & Development

- [Tech stack (Vue, Tauri, Rust, Pinia)](./architecture.md#tech-stack)
- [Project folder structure](./architecture.md#project-structure)
- [Backend modules (per AWS service)](./architecture.md#backend-architecture)
- [Frontend ↔ Backend communication](./architecture.md#frontend-architecture)
- [Design system (CSS variables)](./architecture.md#design-system)
- [Security model](./architecture.md#security-model)

---

## 🤝 Contributing

- [Fork & clone](./contributing.md#1-fork--clone)
- [Development setup](./contributing.md#development-setup)
- [Adding a new AWS service](./contributing.md#adding-a-new-aws-service)
- [Adding a frontend feature](./contributing.md#adding-a-new-frontend-feature)
- [Code style (frontend)](./contributing.md#frontend-typescriptvue)
- [Code style (backend/Rust)](./contributing.md#backend-rust)
- [Submitting a PR](./contributing.md#5-submit-pr)

---

## ❓ FAQ & Troubleshooting

- [Is it free?](../guides/faq.md#is-easy-cloud-free)
- [Are my credentials safe?](../guides/faq.md#are-my-credentials-safe)
- [Does it cost money on AWS?](../guides/faq.md#does-using-easy-cloud-cost-money-on-aws)
- ["Validation failed" on deploy](../guides/faq.md#validation-failed-when-deploying-a-stack)
- [Upload fails for large files](../guides/faq.md#upload-fails-for-large-files)
- [Preview not working](../guides/faq.md#preview-shows-no-preview-available)
- [Context menu not appearing](../guides/faq.md#context-menu-doesnt-appear)
- [Why Tauri instead of Electron?](../guides/faq.md#why-tauri-instead-of-electron)
- [First build takes forever](../guides/faq.md#first-build-takes-forever)

---

## 📚 Reference

- [AWS SDK — all available APIs](../misc/aws-sdk-capabilities.md)
- [Roadmap](./roadmap.md)

---

## 🔗 External Links

| | |
|---|---|
| 📦 Download | [Latest Release](https://github.com/Andrezz64/easy-cloud/releases/latest) |
| 🐛 Bug Report | [Issues](https://github.com/Andrezz64/easy-cloud/issues) |
| 🏠 Landing | [GitHub Pages](https://andrezz64.github.io/easy-cloud/) |
| 📄 License | AGPL-3.0 |
