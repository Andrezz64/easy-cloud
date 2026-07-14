[← Summary](../core/SUMMARY.md)

# FAQ

## General

### Is Easy Cloud free?
Yes, 100% free and open source (AGPL-3.0). No paid tiers, no premium features.

### Does Easy Cloud store my data?
No. All data stays on your machine. Credentials are stored in localStorage. Zero external servers.

### Which AWS services are supported?
Currently: S3, CloudFormation, Cost Explorer, STS. More coming.

### Does it work with other cloud providers?
Currently AWS only. Multi-cloud support is on the roadmap.

---

## Security

### Are my credentials safe?
Credentials never leave your machine. The app makes direct API calls to AWS — there's no intermediary server. You can verify this in the source code.

### Can I use IAM roles instead of access keys?
Not yet. Currently requires Access Key ID + Secret Access Key. IAM role assumption and SSO support are planned.

---

## Costs

### Does using Easy Cloud cost money on AWS?
The app itself is free. However, some AWS APIs have costs:

| Service | Cost |
|---------|------|
| S3 operations | ~$0.005/1000 requests |
| CloudFormation | **Free** |
| STS | **Free** |
| Cost Explorer | **$0.01/request** |

The billing section uses a 2-hour cache to minimize Cost Explorer charges (~$0.15-0.30/month with normal use).

### Why does the billing page cost money?
AWS charges $0.01 per Cost Explorer API call. Easy Cloud caches data for 2 hours per account. The Refresh button bypasses the cache (use sparingly).

---

## Troubleshooting

### "Validation failed" when deploying a stack
- Check that Logical IDs are alphanumeric (no hyphens: `MyBucket` ✓, `my-bucket` ✗)
- Verify resource property names match AWS documentation
- Use the **Validate** button before deploying
- S3 bucket names must be globally unique

### Upload fails for large files
Files > 10MB use multipart upload automatically. If it still fails:
- Check your IAM permissions include `s3:PutObject` and `s3:CreateMultipartUpload`
- Ensure the bucket doesn't have restrictive policies

### Preview shows "No preview available"
The file might be binary or in an unrecognized format. The preview works for:
- Text files (any extension with valid UTF-8 content)
- Images (PNG, JPG, GIF, WebP)
- Code files (50+ extensions recognized)

### Context menu doesn't appear
The native browser context menu is disabled globally. If the custom menu doesn't show, try clicking in the file table area or on a specific file.

---

## Development

### First build takes forever
Normal — Cargo downloads and compiles ~500 Rust crates on first run. Subsequent builds are incremental and fast (~5-10s).

### How do I add a new AWS service?
See [Contributing Guide](./contributing.md#adding-a-new-aws-service).

### Why Tauri instead of Electron?
- 10x smaller binary size
- 5x less RAM usage
- Rust backend = native performance
- No bundled Chromium
