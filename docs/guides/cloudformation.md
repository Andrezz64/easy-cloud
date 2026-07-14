[← Summary](../core/SUMMARY.md)

# CloudFormation

## Overview

The CloudFormation view has two main panels:
- **Left sidebar** — List of deployed stacks
- **Right panel** — YAML editor (when no stack selected) or Stack detail (when a stack is selected)

## Creating a Stack

### Using the YAML Editor
1. Write or paste your CloudFormation YAML in the editor
2. Click **Validate** to check syntax
3. Click **Deploy** → enter a stack name → **Confirm Deploy**
4. The deploy tracker opens showing real-time progress

### Using the Visual Builder
1. Switch to the **Builder** tab
2. Select a resource type (S3, SQS, SNS)
3. Fill in the configuration fields
4. The YAML is generated automatically in the preview
5. Click **Open in Editor** to move it to the code editor
6. Deploy as usual

## Stack Detail

Click on any stack in the sidebar to view its details:

### Tabs

| Tab | Description |
|-----|-------------|
| **Overview** | Stack ID, description, timestamps, parameters, capabilities, IAM role |
| **Resources** | All resources with logical ID, type, and status |
| **Outputs** | Stack outputs (URLs, ARNs, exported values) |
| **Template** | View the deployed template YAML, load to editor, estimate cost |
| **Drift** | Detect if resources were changed outside CloudFormation |
| **Changes** | Create and manage change sets (preview updates) |
| **Events** | Full event history with timestamps and reasons |

### Actions
- **Update** — Apply the current editor template as an update
- **Delete** — Delete the stack and all its resources

## Drift Detection

Drift means someone (or something) modified a resource directly (via console, CLI, etc.) instead of through CloudFormation.

1. Select a stack → go to **Drift** tab
2. Click **Detect Drift**
3. Wait for detection to complete (polls automatically)
4. Results show each resource as:
   - 🟢 `IN_SYNC` — matches the template
   - 🔴 `MODIFIED` — properties differ from expected
   - 🔴 `DELETED` — resource was deleted outside CF

## Change Sets

Change sets let you preview what an update will do **before applying it**:

1. Edit your template in the editor
2. Select a stack → go to **Changes** tab
3. Click **Create Change Set** → enter a name
4. Wait for AWS to analyze (~2-5 seconds)
5. Click **View Changes** to see:
   - 🟢 **Add** — new resource will be created
   - 🟡 **Modify** — existing resource will be updated
   - 🔴 **Remove** — resource will be deleted
   - ⚠️ **REPLACE** badge — resource will be destroyed and recreated
6. If satisfied → **Execute** applies the changes
7. If not → **Delete** discards the change set

## Operation Tracker

When you deploy, update, or delete a stack, a floating tracker appears showing:
- Operation type (Deploy / Update / Delete)
- Overall status badge
- Progress bar (resources completed / total)
- Per-resource status with timestamps
- Elapsed time
- Auto-closes when the operation completes

## Context Menus

**Right-click on a stack:**
- View Details
- View Template
- Detect Drift
- Change Sets
- Update Stack
- Delete Stack

**Right-click on empty area:**
- Deploy New Stack
- Validate Template
- Refresh Stacks

## Template Validation

Click **Validate** to check your template before deploying. Common errors:
- Invalid Logical IDs (must be alphanumeric, no hyphens)
- Unknown resource types
- Missing required properties
- Invalid property values
