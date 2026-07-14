[← Summary](./SUMMARY.md)

# Features Overview

## Dashboard

The main Dashboard provides a consolidated view of your AWS account:

- **Stat cards** — S3 buckets count, CF stacks count, active/failed stacks, current month cost
- **Stacks list** — Recent CloudFormation stacks with status badges
- **Buckets list** — Quick access to your S3 buckets
- **Multi-account** — Add multiple AWS accounts and switch between them

## S3 File Manager

Full file browser with desktop-like interaction:

- Browse files and folders with breadcrumb navigation
- Upload files (auto multipart for files > 10MB)
- Download files with save dialog
- Create folders and empty files
- Preview text, code, and images inline
- **In-app code editor** with syntax highlighting (Ctrl+S saves to S3)
- Copy, move, rename objects
- Batch delete selected objects
- Generate presigned URLs (temporary share links)
- Right-click context menus on files and empty areas
- Copy bucket URL with one click

## CloudFormation

Infrastructure as Code management:

- **Editor** — YAML editor with syntax highlighting
- **Builder** — Visual form to generate CloudFormation templates (S3, SQS, SNS)
- **Deploy** — Create stacks with real-time event tracking
- **Update** — Update existing stacks
- **Delete** — Delete stacks with progress tracking
- **Validate** — Check template syntax before deploying
- **Template View** — See the deployed template, load it into editor
- **Drift Detection** — Check if resources were modified outside CloudFormation
- **Change Sets** — Preview what will change before applying an update
- **Exports/Imports** — View cross-stack dependencies
- **Cost Estimation** — Get AWS Calculator link for your template

## Billing & Cost Explorer

Cost analysis with smart caching (2h TTL to minimize API costs):

- **Summary cards** — Current month, last month, forecast
- **Daily/Monthly chart** — Cost over time with adaptive granularity
- **Top services** — Bar chart of most expensive services
- **Usage breakdown** — Per-service usage with unit price calculation
- **Free tier detection** — Shows which resources are within free tier
- **Period filter** — 7 days, 1 month, 3 months, 6 months, 1 year, custom range
- **Per-account cache** — Switching accounts uses cached data when available
