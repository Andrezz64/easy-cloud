<template>
  <div class="cf-view">
    <!-- Header -->
    <div class="page-header">
      <div>
        <h1>CloudFormation</h1>
        <p class="text-secondary text-sm">Build and deploy infrastructure as code</p>
      </div>
      <div class="page-actions">
        <button 
          class="btn-secondary" 
          @click="fetchStacks" 
          :disabled="loadingStacks || !accountsStore.activeAccountId"
        >
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" :class="{ spinning: loadingStacks }">
            <path d="M14 8A6 6 0 1 1 8 2" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
            <path d="M14 2V6H10" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          Refresh
        </button>
        <button 
          class="btn-primary" 
          @click="openDeployModal" 
          :disabled="!accountsStore.activeAccountId || !templateYaml.trim()"
        >
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
            <path d="M8 2V14M8 2L4 6M8 2L12 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          Deploy
        </button>
        <button 
          class="btn-secondary" 
          @click="validateTemplate" 
          :disabled="!accountsStore.activeAccountId || !templateYaml.trim() || validating"
        >
          {{ validating ? 'Validating...' : '✓ Validate' }}
        </button>
      </div>
    </div>

    <!-- No Account State -->
    <div v-if="!accountsStore.activeAccountId" class="empty-state">
      <div class="empty-icon">
        <svg width="40" height="40" viewBox="0 0 16 16" fill="none">
          <path d="M2 4L8 1L14 4V12L8 15L2 12V4Z" stroke="currentColor" stroke-width="1" stroke-linejoin="round"/>
          <path d="M8 7V15" stroke="currentColor" stroke-width="1"/>
          <path d="M2 4L8 7L14 4" stroke="currentColor" stroke-width="1"/>
        </svg>
      </div>
      <p class="text-secondary">Select an AWS account from the sidebar to manage CloudFormation stacks.</p>
    </div>

    <!-- Main Layout -->
    <div v-else class="cf-layout">
      <!-- Left: Stacks -->
      <aside class="stacks-panel">
        <div class="stacks-header">
          <h3>Stacks</h3>
          <span class="badge badge--neutral" v-if="stacks.length > 0">{{ stacks.length }}</span>
        </div>

        <div v-if="stacksError" class="error-banner">
          <span>{{ stacksError }}</span>
        </div>

        <!-- Loading -->
        <div v-else-if="loadingStacks" class="stacks-loading">
          <div v-for="i in 4" :key="i" class="stack-skeleton">
            <div class="skeleton" style="width: 60%; height: 14px;"></div>
            <div class="skeleton" style="width: 35%; height: 12px; margin-top: 6px;"></div>
          </div>
        </div>

        <!-- Empty -->
        <div v-else-if="stacks.length === 0" class="stacks-empty">
          <p class="text-secondary text-sm">No stacks deployed yet.</p>
        </div>

        <!-- Stack List -->
        <div v-else class="stacks-list" @contextmenu.prevent="openStacksAreaCtx($event)">
          <div 
            v-for="stack in stacks" 
            :key="stack.name" 
            class="stack-item"
            :class="{ 'stack-item--active': selectedStack === stack.name }"
            @click="selectStack(stack.name)"
            @contextmenu.prevent.stop="openStackCtx($event, stack)"
          >
            <div class="stack-name">{{ stack.name }}</div>
            <div class="stack-meta">
              <span :class="statusBadgeClass(stack.status)" class="badge">
                {{ formatStatus(stack.status) }}
              </span>
              <span class="text-tertiary text-sm">{{ formatDate(stack.creation_time) }}</span>
            </div>
          </div>
        </div>
      </aside>

      <!-- Right: Editor OR Stack Detail -->
      <div class="editor-panel" v-if="!selectedStack">
        <!-- Tabs -->
        <div class="editor-tabs">
          <button 
            class="tab" 
            :class="{ 'tab--active': viewMode === 'code' }" 
            @click="viewMode = 'code'"
          >
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
              <path d="M5 4L1 8L5 12" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
              <path d="M11 4L15 8L11 12" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
              <path d="M9 2L7 14" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
            </svg>
            YAML Editor
          </button>
          <button 
            class="tab" 
            :class="{ 'tab--active': viewMode === 'builder' }" 
            @click="viewMode = 'builder'"
          >
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
              <rect x="2" y="2" width="5" height="5" rx="1" stroke="currentColor" stroke-width="1.3"/>
              <rect x="9" y="2" width="5" height="5" rx="1" stroke="currentColor" stroke-width="1.3"/>
              <rect x="2" y="9" width="5" height="5" rx="1" stroke="currentColor" stroke-width="1.3"/>
              <path d="M11.5 10V14M9.5 12H13.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
            </svg>
            Builder
          </button>
        </div>

        <!-- Code Editor -->
        <div v-show="viewMode === 'code'" class="editor-content">
          <codemirror
            v-model="templateYaml"
            placeholder="Paste your AWS CloudFormation YAML here..."
            :style="{ height: '100%' }"
            :autofocus="true"
            :indent-with-tab="true"
            :tab-size="2"
            :extensions="extensions"
            @change="onTemplateChange"
          />
        </div>

        <!-- UI Builder -->
        <div v-show="viewMode === 'builder'" class="builder-content">
          <div class="builder-layout">
            <!-- Resource Selector -->
            <div class="builder-sidebar">
              <h3>Resource Type</h3>
              <div class="resource-type-list">
                <button
                  v-for="rt in resourceTypes"
                  :key="rt.id"
                  class="resource-type-btn"
                  :class="{ 'resource-type-btn--active': builderForm.type === rt.id }"
                  @click="selectResourceType(rt.id)"
                >
                  <div class="rt-icon" v-html="rt.icon"></div>
                  <div class="rt-info">
                    <span class="rt-name">{{ rt.label }}</span>
                    <span class="rt-desc">{{ rt.shortDesc }}</span>
                  </div>
                </button>
              </div>
            </div>

            <!-- Config Panel -->
            <div class="builder-config">
              <div class="config-header">
                <h3>{{ currentResourceType?.label }}</h3>
                <span class="badge badge--neutral">{{ currentResourceType?.cfType }}</span>
              </div>
              <p class="text-secondary text-sm config-desc">{{ currentResourceType?.description }}</p>

              <!-- Common Fields -->
              <div class="config-section">
                <div class="config-section-title">General</div>
                <div class="field-group">
                  <div class="field">
                    <div class="field-header">
                      <label>Logical ID</label>
                      <button class="tip-btn" @click="toggleTip('logicalId')">?</button>
                    </div>
                    <input v-model="builderForm.logicalId" type="text" class="glass-input" placeholder="MyResource" @input="sanitizeLogicalId(); generateYaml()" />
                    <div v-if="activeTip === 'logicalId'" class="tip-box">
                      <strong>Logical ID</strong> is the unique identifier for this resource within the template. Other resources can reference it using <code>!Ref</code> or <code>!GetAtt</code>. Must be alphanumeric only (no hyphens, spaces or special characters).
                    </div>
                  </div>
                </div>
              </div>

              <!-- S3 Config -->
              <template v-if="builderForm.type === 's3'">
                <div class="config-section">
                  <div class="config-section-title">Bucket Settings</div>
                  <div class="field-group">
                    <div class="field">
                      <div class="field-header">
                        <label>Bucket Name <span class="optional">(optional)</span></label>
                        <button class="tip-btn" @click="toggleTip('s3BucketName')">?</button>
                      </div>
                      <input v-model="builderForm.s3.bucketName" type="text" class="glass-input" placeholder="my-unique-bucket-name" @input="generateYaml" />
                      <div v-if="activeTip === 's3BucketName'" class="tip-box">
                        <strong>BucketName</strong> must be globally unique across all AWS accounts. If omitted, CloudFormation generates a unique name. Use lowercase letters, numbers, and hyphens only (3-63 chars).
                      </div>
                    </div>
                    <div class="field">
                      <div class="field-header">
                        <label>Access Control</label>
                        <button class="tip-btn" @click="toggleTip('s3AccessControl')">?</button>
                      </div>
                      <select v-model="builderForm.s3.accessControl" class="glass-select" @change="generateYaml">
                        <option value="">None (recommended)</option>
                        <option value="Private">Private</option>
                        <option value="PublicRead">Public Read</option>
                        <option value="PublicReadWrite">Public Read Write</option>
                        <option value="AuthenticatedRead">Authenticated Read</option>
                        <option value="LogDeliveryWrite">Log Delivery Write</option>
                        <option value="BucketOwnerRead">Bucket Owner Read</option>
                        <option value="BucketOwnerFullControl">Bucket Owner Full Control</option>
                      </select>
                      <div v-if="activeTip === 's3AccessControl'" class="tip-box">
                        <strong>AccessControl</strong> (ACL) is a legacy access method. AWS recommends using Bucket Policies instead. If set to "None", the bucket uses S3 Object Ownership with ACLs disabled (most secure default).
                      </div>
                    </div>
                  </div>
                </div>

                <div class="config-section">
                  <div class="config-section-title">Versioning &amp; Encryption</div>
                  <div class="field-group">
                    <div class="field">
                      <div class="field-header">
                        <label>Versioning</label>
                        <button class="tip-btn" @click="toggleTip('s3Versioning')">?</button>
                      </div>
                      <select v-model="builderForm.s3.versioning" class="glass-select" @change="generateYaml">
                        <option value="">Disabled</option>
                        <option value="Enabled">Enabled</option>
                        <option value="Suspended">Suspended</option>
                      </select>
                      <div v-if="activeTip === 's3Versioning'" class="tip-box">
                        <strong>Versioning</strong> keeps multiple variants of an object. When enabled, every overwrite or delete creates a new version instead of replacing the object. Useful for recovery from accidental deletions.
                      </div>
                    </div>
                    <div class="field">
                      <div class="field-header">
                        <label>Encryption</label>
                        <button class="tip-btn" @click="toggleTip('s3Encryption')">?</button>
                      </div>
                      <select v-model="builderForm.s3.encryption" class="glass-select" @change="generateYaml">
                        <option value="">Default (SSE-S3)</option>
                        <option value="AES256">SSE-S3 (AES-256)</option>
                        <option value="aws:kms">SSE-KMS</option>
                      </select>
                      <div v-if="activeTip === 's3Encryption'" class="tip-box">
                        <strong>Server-Side Encryption</strong> encrypts data at rest. <em>SSE-S3</em> uses Amazon-managed keys (free). <em>SSE-KMS</em> uses AWS KMS keys with additional auditing via CloudTrail and per-key permissions.
                      </div>
                    </div>
                  </div>
                </div>

                <div class="config-section">
                  <div class="config-section-title">Public Access</div>
                  <div class="field-group">
                    <div class="field">
                      <div class="field-header">
                        <label>Block Public Access</label>
                        <button class="tip-btn" @click="toggleTip('s3PublicAccess')">?</button>
                      </div>
                      <select v-model="builderForm.s3.blockPublicAccess" class="glass-select" @change="generateYaml">
                        <option value="all">Block All (recommended)</option>
                        <option value="none">Allow (not recommended)</option>
                      </select>
                      <div v-if="activeTip === 's3PublicAccess'" class="tip-box">
                        <strong>Block Public Access</strong> overrides any bucket policies or ACLs that could grant public access. AWS strongly recommends keeping this enabled unless you explicitly need public access (e.g., static websites).
                      </div>
                    </div>
                  </div>
                </div>

                <div class="config-section">
                  <div class="config-section-title">Static Website Hosting</div>
                  <div class="field-group">
                    <div class="field">
                      <div class="field-header">
                        <label>Enable Website Hosting</label>
                        <button class="tip-btn" @click="toggleTip('s3Website')">?</button>
                      </div>
                      <select v-model="builderForm.s3.websiteEnabled" class="glass-select" @change="generateYaml">
                        <option value="">Disabled</option>
                        <option value="true">Enabled</option>
                      </select>
                      <div v-if="activeTip === 's3Website'" class="tip-box">
                        <strong>WebsiteConfiguration</strong> turns your bucket into a static website host. S3 will serve objects as web pages via a public endpoint like <code>http://bucket-name.s3-website-region.amazonaws.com</code>. You must also disable Block Public Access and add a bucket policy allowing <code>s3:GetObject</code>.
                      </div>
                    </div>
                    <template v-if="builderForm.s3.websiteEnabled">
                      <div class="field">
                        <div class="field-header">
                          <label>Index Document</label>
                          <button class="tip-btn" @click="toggleTip('s3IndexDoc')">?</button>
                        </div>
                        <input v-model="builderForm.s3.indexDocument" type="text" class="glass-input" placeholder="index.html" @input="generateYaml" />
                        <div v-if="activeTip === 's3IndexDoc'" class="tip-box">
                          <strong>IndexDocument</strong> is the default page served when users access the root URL or a folder path. Typically <code>index.html</code>.
                        </div>
                      </div>
                      <div class="field">
                        <div class="field-header">
                          <label>Error Document <span class="optional">(optional)</span></label>
                          <button class="tip-btn" @click="toggleTip('s3ErrorDoc')">?</button>
                        </div>
                        <input v-model="builderForm.s3.errorDocument" type="text" class="glass-input" placeholder="error.html" @input="generateYaml" />
                        <div v-if="activeTip === 's3ErrorDoc'" class="tip-box">
                          <strong>ErrorDocument</strong> is the page S3 returns for 4xx errors (like 404 Not Found). Useful for SPAs where you want to redirect all routes to a single page.
                        </div>
                      </div>
                    </template>
                  </div>
                </div>

                <div class="config-section">
                  <div class="config-section-title">Lifecycle &amp; Tags</div>
                  <div class="field-group">
                    <div class="field">
                      <div class="field-header">
                        <label>Deletion Policy</label>
                        <button class="tip-btn" @click="toggleTip('s3DeletionPolicy')">?</button>
                      </div>
                      <select v-model="builderForm.s3.deletionPolicy" class="glass-select" @change="generateYaml">
                        <option value="">Delete (default)</option>
                        <option value="Retain">Retain</option>
                        <option value="Snapshot">Snapshot</option>
                      </select>
                      <div v-if="activeTip === 's3DeletionPolicy'" class="tip-box">
                        <strong>DeletionPolicy</strong> controls what happens when the stack is deleted. <em>Delete</em> removes the bucket (must be empty). <em>Retain</em> keeps the bucket even after stack deletion — useful for data protection.
                      </div>
                    </div>
                  </div>
                </div>
              </template>

              <!-- SQS Config -->
              <template v-if="builderForm.type === 'sqs'">
                <div class="config-section">
                  <div class="config-section-title">Queue Settings</div>
                  <div class="field-group">
                    <div class="field">
                      <div class="field-header">
                        <label>Queue Name <span class="optional">(optional)</span></label>
                        <button class="tip-btn" @click="toggleTip('sqsName')">?</button>
                      </div>
                      <input v-model="builderForm.sqs.queueName" type="text" class="glass-input" placeholder="my-queue" @input="generateYaml" />
                      <div v-if="activeTip === 'sqsName'" class="tip-box">
                        <strong>QueueName</strong> if omitted, CloudFormation generates a unique name. FIFO queues must end with <code>.fifo</code>. Max 80 chars.
                      </div>
                    </div>
                    <div class="field">
                      <div class="field-header">
                        <label>Queue Type</label>
                        <button class="tip-btn" @click="toggleTip('sqsFifo')">?</button>
                      </div>
                      <select v-model="builderForm.sqs.fifoQueue" class="glass-select" @change="generateYaml">
                        <option value="">Standard</option>
                        <option value="true">FIFO (First-In-First-Out)</option>
                      </select>
                      <div v-if="activeTip === 'sqsFifo'" class="tip-box">
                        <strong>Standard</strong> queues offer maximum throughput, best-effort ordering, and at-least-once delivery. <strong>FIFO</strong> queues guarantee exactly-once processing and strict ordering, but with lower throughput (300 msg/s without batching).
                      </div>
                    </div>
                  </div>
                </div>

                <div class="config-section">
                  <div class="config-section-title">Timing &amp; Retention</div>
                  <div class="field-group">
                    <div class="field">
                      <div class="field-header">
                        <label>Visibility Timeout (seconds)</label>
                        <button class="tip-btn" @click="toggleTip('sqsVisibility')">?</button>
                      </div>
                      <input v-model="builderForm.sqs.visibilityTimeout" type="number" class="glass-input" placeholder="30" min="0" max="43200" @input="generateYaml" />
                      <div v-if="activeTip === 'sqsVisibility'" class="tip-box">
                        <strong>VisibilityTimeout</strong> is the time a message is hidden from other consumers after being read. If the consumer doesn't delete it within this window, the message becomes visible again. Default: 30s. Max: 12 hours.
                      </div>
                    </div>
                    <div class="field">
                      <div class="field-header">
                        <label>Message Retention (seconds)</label>
                        <button class="tip-btn" @click="toggleTip('sqsRetention')">?</button>
                      </div>
                      <input v-model="builderForm.sqs.messageRetention" type="number" class="glass-input" placeholder="345600" min="60" max="1209600" @input="generateYaml" />
                      <div v-if="activeTip === 'sqsRetention'" class="tip-box">
                        <strong>MessageRetentionPeriod</strong> controls how long unprocessed messages stay in the queue before being automatically deleted. Default: 4 days (345600s). Max: 14 days.
                      </div>
                    </div>
                    <div class="field">
                      <div class="field-header">
                        <label>Delay (seconds)</label>
                        <button class="tip-btn" @click="toggleTip('sqsDelay')">?</button>
                      </div>
                      <input v-model="builderForm.sqs.delaySeconds" type="number" class="glass-input" placeholder="0" min="0" max="900" @input="generateYaml" />
                      <div v-if="activeTip === 'sqsDelay'" class="tip-box">
                        <strong>DelaySeconds</strong> postpones delivery of new messages. During the delay period, messages are invisible to consumers. Useful for scheduling or rate limiting. Default: 0. Max: 15 minutes.
                      </div>
                    </div>
                    <div class="field">
                      <div class="field-header">
                        <label>Max Message Size (bytes)</label>
                        <button class="tip-btn" @click="toggleTip('sqsMaxSize')">?</button>
                      </div>
                      <input v-model="builderForm.sqs.maxMessageSize" type="number" class="glass-input" placeholder="262144" min="1024" max="262144" @input="generateYaml" />
                      <div v-if="activeTip === 'sqsMaxSize'" class="tip-box">
                        <strong>MaximumMessageSize</strong> is the max payload per message. Default and max: 256 KB (262144 bytes). For larger payloads, use the Extended Client Library to store the body in S3.
                      </div>
                    </div>
                  </div>
                </div>

                <div class="config-section">
                  <div class="config-section-title">Dead Letter Queue</div>
                  <div class="field-group">
                    <div class="field">
                      <div class="field-header">
                        <label>Max Receive Count</label>
                        <button class="tip-btn" @click="toggleTip('sqsDLQ')">?</button>
                      </div>
                      <input v-model="builderForm.sqs.maxReceiveCount" type="number" class="glass-input" placeholder="Leave empty to disable DLQ" min="1" @input="generateYaml" />
                      <div v-if="activeTip === 'sqsDLQ'" class="tip-box">
                        <strong>Dead Letter Queue</strong> captures messages that fail processing after N attempts. Set <em>maxReceiveCount</em> to the number of retries before sending to DLQ. Requires a separate DLQ resource (not generated here).
                      </div>
                    </div>
                  </div>
                </div>
              </template>

              <!-- SNS Config -->
              <template v-if="builderForm.type === 'sns'">
                <div class="config-section">
                  <div class="config-section-title">Topic Settings</div>
                  <div class="field-group">
                    <div class="field">
                      <div class="field-header">
                        <label>Topic Name <span class="optional">(optional)</span></label>
                        <button class="tip-btn" @click="toggleTip('snsName')">?</button>
                      </div>
                      <input v-model="builderForm.sns.topicName" type="text" class="glass-input" placeholder="my-topic" @input="generateYaml" />
                      <div v-if="activeTip === 'snsName'" class="tip-box">
                        <strong>TopicName</strong> is the display name for the topic. If omitted, CloudFormation generates a unique name. FIFO topics must end with <code>.fifo</code>.
                      </div>
                    </div>
                    <div class="field">
                      <div class="field-header">
                        <label>Display Name</label>
                        <button class="tip-btn" @click="toggleTip('snsDisplayName')">?</button>
                      </div>
                      <input v-model="builderForm.sns.displayName" type="text" class="glass-input" placeholder="My Notifications" @input="generateYaml" />
                      <div v-if="activeTip === 'snsDisplayName'" class="tip-box">
                        <strong>DisplayName</strong> appears in SMS messages and email "From" fields. Limited to 10 chars for SMS.
                      </div>
                    </div>
                    <div class="field">
                      <div class="field-header">
                        <label>Topic Type</label>
                        <button class="tip-btn" @click="toggleTip('snsFifo')">?</button>
                      </div>
                      <select v-model="builderForm.sns.fifoTopic" class="glass-select" @change="generateYaml">
                        <option value="">Standard</option>
                        <option value="true">FIFO</option>
                      </select>
                      <div v-if="activeTip === 'snsFifo'" class="tip-box">
                        <strong>FIFO topics</strong> provide strict message ordering and deduplication. They can only deliver to FIFO SQS queues. Standard topics offer higher throughput and support all subscription protocols (HTTP, email, Lambda, etc.).
                      </div>
                    </div>
                    <div class="field">
                      <div class="field-header">
                        <label>KMS Key (encryption)</label>
                        <button class="tip-btn" @click="toggleTip('snsKms')">?</button>
                      </div>
                      <input v-model="builderForm.sns.kmsMasterKeyId" type="text" class="glass-input" placeholder="alias/aws/sns or KMS key ARN" @input="generateYaml" />
                      <div v-if="activeTip === 'snsKms'" class="tip-box">
                        <strong>KmsMasterKeyId</strong> enables server-side encryption for the topic. Use <code>alias/aws/sns</code> for the default AWS-managed key, or provide a custom KMS key ARN for more control.
                      </div>
                    </div>
                  </div>
                </div>
              </template>

              <!-- YAML Preview -->
              <div class="config-section">
                <div class="builder-preview">
                  <div class="builder-preview-header">
                    <span class="text-sm text-secondary">Generated YAML</span>
                    <button class="btn-secondary" style="padding: 0.25rem 0.5rem; font-size: 0.6875rem;" @click="viewMode = 'code'">
                      Open in Editor →
                    </button>
                  </div>
                  <pre class="preview-code">{{ templateYaml }}</pre>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Stack Detail Panel -->
      <div class="editor-panel" v-else>
        <div class="detail-header">
          <div class="detail-header-left">
            <button class="btn-back-sm" @click="selectedStack = null" title="Back to editor">
              <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
                <path d="M10 4L6 8L10 12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
              </svg>
            </button>
            <h3>{{ selectedStack }}</h3>
            <span v-if="stackDetails?.status" :class="statusBadgeClass(stackDetails.status)" class="badge">
              {{ formatStatus(stackDetails.status) }}
            </span>
          </div>
          <div class="detail-actions">
            <button class="btn-secondary" @click="updateSelectedStack" :disabled="!templateYaml.trim()">
              <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
                <path d="M8 2V14M8 2L4 6M8 2L12 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
              Update
            </button>
            <button class="btn-danger" @click="confirmDeleteStack">Delete</button>
          </div>
        </div>

        <!-- Detail Tabs -->
        <div class="editor-tabs">
          <button class="tab" :class="{ 'tab--active': detailTab === 'overview' }" @click="detailTab = 'overview'">Overview</button>
          <button class="tab" :class="{ 'tab--active': detailTab === 'resources' }" @click="detailTab = 'resources'; loadResources()">Resources</button>
          <button class="tab" :class="{ 'tab--active': detailTab === 'outputs' }" @click="detailTab = 'outputs'">Outputs</button>
          <button class="tab" :class="{ 'tab--active': detailTab === 'template' }" @click="detailTab = 'template'; loadStackTemplate()">Template</button>
          <button class="tab" :class="{ 'tab--active': detailTab === 'drift' }" @click="detailTab = 'drift'">Drift</button>
          <button class="tab" :class="{ 'tab--active': detailTab === 'changes' }" @click="detailTab = 'changes'; loadChangeSets()">Changes</button>
          <button class="tab" :class="{ 'tab--active': detailTab === 'events' }" @click="detailTab = 'events'; loadEvents()">Events</button>
        </div>

        <!-- Overview Tab -->
        <div v-if="detailTab === 'overview'" class="detail-content">
          <div v-if="loadingDetails" class="detail-loading">Loading...</div>
          <template v-else-if="stackDetails">
            <div class="detail-grid">
              <div class="detail-field">
                <span class="detail-label">Stack ID</span>
                <span class="detail-value mono">{{ stackDetails.stack_id || '—' }}</span>
              </div>
              <div class="detail-field">
                <span class="detail-label">Description</span>
                <span class="detail-value">{{ stackDetails.description || '—' }}</span>
              </div>
              <div class="detail-field">
                <span class="detail-label">Created</span>
                <span class="detail-value">{{ formatDateFull(stackDetails.creation_time) }}</span>
              </div>
              <div class="detail-field">
                <span class="detail-label">Last Updated</span>
                <span class="detail-value">{{ formatDateFull(stackDetails.last_updated_time) }}</span>
              </div>
              <div class="detail-field" v-if="stackDetails.status_reason">
                <span class="detail-label">Status Reason</span>
                <span class="detail-value detail-value--warn">{{ stackDetails.status_reason }}</span>
              </div>
              <div class="detail-field" v-if="stackDetails.role_arn">
                <span class="detail-label">IAM Role</span>
                <span class="detail-value mono">{{ stackDetails.role_arn }}</span>
              </div>
              <div class="detail-field" v-if="stackDetails.capabilities?.length">
                <span class="detail-label">Capabilities</span>
                <span class="detail-value">{{ stackDetails.capabilities.join(', ') }}</span>
              </div>
            </div>
            <div v-if="stackDetails.parameters?.length" class="detail-section">
              <h4>Parameters</h4>
              <div class="detail-table">
                <div class="detail-table-row" v-for="p in stackDetails.parameters" :key="p.key">
                  <span class="detail-table-key">{{ p.key }}</span>
                  <span class="detail-table-val">{{ p.value }}</span>
                </div>
              </div>
            </div>
          </template>
        </div>

        <!-- Resources Tab -->
        <div v-if="detailTab === 'resources'" class="detail-content">
          <div v-if="loadingResources" class="detail-loading">Loading resources...</div>
          <div v-else-if="stackResources.length === 0" class="detail-empty">No resources found.</div>
          <div v-else class="resource-table">
            <div class="resource-table-header">
              <span>Logical ID</span>
              <span>Type</span>
              <span>Status</span>
            </div>
            <div v-for="r in stackResources" :key="r.logical_resource_id" class="resource-table-row">
              <span class="resource-logical-id">{{ r.logical_resource_id }}</span>
              <span class="resource-type">{{ r.resource_type?.replace('AWS::', '') }}</span>
              <span :class="statusBadgeClass(r.resource_status || '')" class="badge">{{ formatStatus(r.resource_status || '') }}</span>
            </div>
          </div>
        </div>

        <!-- Outputs Tab -->
        <div v-if="detailTab === 'outputs'" class="detail-content">
          <div v-if="!stackDetails?.outputs?.length" class="detail-empty">No outputs defined.</div>
          <div v-else class="detail-table">
            <div class="detail-table-row" v-for="o in stackDetails?.outputs" :key="o.key">
              <div class="output-row">
                <span class="detail-table-key">{{ o.key }}</span>
                <span class="detail-table-val mono">{{ o.value }}</span>
                <span v-if="o.description" class="text-tertiary text-sm">{{ o.description }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Events Tab -->
        <div v-if="detailTab === 'events'" class="detail-content">
          <div v-if="loadingEvents" class="detail-loading">Loading events...</div>
          <div v-else-if="stackEvents.length === 0" class="detail-empty">No events.</div>
          <div v-else class="events-table">
            <div v-for="ev in stackEvents" :key="ev.event_id" class="event-row" :class="eventRowClass(ev.resource_status)">
              <span class="event-time">{{ formatTime(ev.timestamp) }}</span>
              <span class="event-resource">{{ ev.logical_resource_id }}</span>
              <span :class="statusBadgeClass(ev.resource_status || '')" class="badge badge-sm">{{ formatStatus(ev.resource_status || '') }}</span>
              <span v-if="ev.resource_status_reason" class="event-reason text-tertiary">{{ ev.resource_status_reason }}</span>
            </div>
          </div>
        </div>

        <!-- Template Tab -->
        <div v-if="detailTab === 'template'" class="detail-content">
          <div class="template-actions">
            <button class="btn-secondary" @click="loadStackTemplate" :disabled="loadingTemplate">↻ Refresh</button>
            <button class="btn-secondary" @click="loadToEditor">Load to Editor</button>
            <button class="btn-secondary" @click="estimateCost" :disabled="!stackTemplateBody">💰 Estimate Cost</button>
          </div>
          <div v-if="loadingTemplate" class="detail-loading">Loading template...</div>
          <pre v-else-if="stackTemplateBody" class="template-code">{{ stackTemplateBody }}</pre>
          <div v-else class="detail-empty">No template available.</div>
        </div>

        <!-- Drift Tab -->
        <div v-if="detailTab === 'drift'" class="detail-content">
          <div class="drift-actions">
            <button class="btn-primary" @click="startDriftDetection" :disabled="driftDetecting">
              {{ driftDetecting ? 'Detecting...' : '🔍 Detect Drift' }}
            </button>
            <button class="btn-secondary" @click="loadDriftResults" :disabled="!driftDetectionId">View Results</button>
            <span v-if="driftStatus" class="badge" :class="driftStatusClass">{{ driftStatus }}</span>
          </div>
          <div v-if="driftResources.length > 0" class="drift-results">
            <div class="resource-table">
              <div class="resource-table-header">
                <span>Resource</span>
                <span>Type</span>
                <span>Drift Status</span>
              </div>
              <div v-for="d in driftResources" :key="d.logical_resource_id" class="resource-table-row" :class="driftRowClass(d.drift_status)">
                <span class="resource-logical-id">{{ d.logical_resource_id }}</span>
                <span class="resource-type">{{ d.resource_type?.replace('AWS::', '') }}</span>
                <span class="badge" :class="driftBadgeClass(d.drift_status)">{{ d.drift_status }}</span>
              </div>
            </div>
          </div>
          <div v-else-if="!driftDetecting" class="detail-empty">
            <p class="text-tertiary text-sm">Click "Detect Drift" to check if resources were modified outside CloudFormation.</p>
          </div>
        </div>

        <!-- Changes Tab -->
        <div v-if="detailTab === 'changes'" class="detail-content">
          <div class="drift-actions">
            <button class="btn-primary" @click="openCreateChangeSet" :disabled="!templateYaml.trim()">+ Create Change Set</button>
            <button class="btn-secondary" @click="loadChangeSets">↻ Refresh</button>
          </div>
          <div v-if="loadingChangeSets" class="detail-loading">Loading change sets...</div>
          <div v-else-if="changeSets.length === 0" class="detail-empty">
            <p class="text-tertiary text-sm">No change sets. Create one to preview updates before applying.</p>
          </div>
          <div v-else class="change-sets-list">
            <div v-for="cs in changeSets" :key="cs.change_set_id" class="change-set-card">
              <div class="cs-header">
                <span class="cs-name">{{ cs.change_set_name }}</span>
                <span class="badge badge-sm" :class="statusBadgeClass(cs.status || '')">{{ cs.status }}</span>
              </div>
              <div v-if="cs.description" class="text-tertiary text-sm">{{ cs.description }}</div>
              <div class="cs-actions">
                <button class="btn-secondary" style="font-size:0.6875rem;padding:0.25rem 0.5rem" @click="viewChangeSet(cs.change_set_name!)">View Changes</button>
                <button v-if="cs.execution_status === 'AVAILABLE'" class="btn-primary" style="font-size:0.6875rem;padding:0.25rem 0.5rem" @click="executeChangeSetAction(cs.change_set_name!)">Execute</button>
                <button class="btn-danger" style="font-size:0.6875rem;padding:0.25rem 0.5rem" @click="deleteChangeSetAction(cs.change_set_name!)">Delete</button>
              </div>
              <!-- Expanded changes -->
              <div v-if="expandedChangeSet === cs.change_set_name && changeSetChanges.length > 0" class="cs-changes">
                <div v-for="ch in changeSetChanges" :key="ch.logical_resource_id" class="cs-change-row">
                  <span class="cs-action-badge" :class="'cs-action--' + (ch.action || '').toLowerCase()">{{ ch.action }}</span>
                  <span class="cs-change-id">{{ ch.logical_resource_id }}</span>
                  <span class="resource-type">{{ ch.resource_type?.replace('AWS::', '') }}</span>
                  <span v-if="ch.replacement === 'True'" class="badge badge--error badge-sm">REPLACE</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Deploy Tracker Panel -->
    <div v-if="deployTrackerVisible" class="deploy-tracker-overlay">
      <div class="deploy-tracker">
        <div class="deploy-tracker-header">
          <div class="deploy-tracker-title">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" :class="{ spinning: deployInProgress }">
              <path d="M14 8A6 6 0 1 1 8 2" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
            <h3>{{ trackedOperation }}: {{ trackedStackName }}</h3>
            <span :class="deployOverallStatusClass" class="badge">{{ deployOverallStatus }}</span>
          </div>
          <button class="btn-icon" @click="closeTracker" title="Close">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
              <path d="M4 4L12 12M12 4L4 12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          </button>
        </div>

        <div class="deploy-tracker-body">
          <!-- Progress Bar -->
          <div class="deploy-progress-bar">
            <div class="deploy-progress-fill" :style="{ width: deployProgressPercent + '%' }" :class="deployProgressBarClass"></div>
          </div>
          <div class="deploy-progress-label">
            <span class="text-sm text-secondary">{{ completedResources }} / {{ totalResources }} resources</span>
            <span class="text-sm text-tertiary">{{ deployElapsedTime }}</span>
          </div>

          <!-- Events List -->
          <div class="deploy-events-list">
            <div 
              v-for="event in groupedDeployEvents" 
              :key="event.logical_resource_id || event.event_id" 
              class="deploy-event-item"
              :class="eventItemClass(event.resource_status)"
            >
              <div class="deploy-event-icon">
                <svg v-if="isEventSuccess(event.resource_status)" width="14" height="14" viewBox="0 0 16 16" fill="none">
                  <circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="1.3"/>
                  <path d="M5.5 8L7 9.5L10.5 6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
                <svg v-else-if="isEventFailed(event.resource_status)" width="14" height="14" viewBox="0 0 16 16" fill="none">
                  <circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="1.3"/>
                  <path d="M6 6L10 10M10 6L6 10" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
                </svg>
                <svg v-else width="14" height="14" viewBox="0 0 16 16" fill="none" class="spinning">
                  <path d="M14 8A6 6 0 1 1 8 2" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
                </svg>
              </div>
              <div class="deploy-event-info">
                <div class="deploy-event-resource">
                  <span class="deploy-event-logical-id">{{ event.logical_resource_id }}</span>
                  <span class="deploy-event-type text-tertiary">{{ formatResourceType(event.resource_type) }}</span>
                </div>
                <div class="deploy-event-status">
                  <span :class="eventStatusClass(event.resource_status)" class="badge badge-sm">{{ formatEventStatus(event.resource_status) }}</span>
                  <span v-if="event.resource_status_reason" class="deploy-event-reason text-sm text-tertiary">{{ event.resource_status_reason }}</span>
                </div>
              </div>
              <div class="deploy-event-time text-sm text-tertiary">
                {{ formatEventTime(event.timestamp) }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Context Menus -->
    <Teleport to="body">
      <div v-if="cfCtxMenu.visible" class="context-menu" :style="cfCtxMenuStyle" @click.stop>
        <!-- Stack context menu -->
        <template v-if="cfCtxMenu.type === 'stack' && cfCtxMenu.target">
          <button class="context-menu-item" @click="selectStack(cfCtxMenu.target.name); closeCfCtx()">
            <span>📋</span> View Details
          </button>
          <button class="context-menu-item" @click="selectStack(cfCtxMenu.target.name); detailTab = 'template'; loadStackTemplate(); closeCfCtx()">
            <span>📄</span> View Template
          </button>
          <button class="context-menu-item" @click="selectStack(cfCtxMenu.target.name); detailTab = 'drift'; closeCfCtx()">
            <span>🔍</span> Detect Drift
          </button>
          <button class="context-menu-item" @click="selectStack(cfCtxMenu.target.name); detailTab = 'changes'; loadChangeSets(); closeCfCtx()">
            <span>📝</span> Change Sets
          </button>
          <div class="context-menu-divider"></div>
          <button class="context-menu-item" @click="selectedStack = cfCtxMenu.target.name; updateSelectedStack(); closeCfCtx()">
            <span>⬆</span> Update Stack
          </button>
          <button class="context-menu-item context-menu-item--danger" @click="selectedStack = cfCtxMenu.target.name; confirmDeleteStack(); closeCfCtx()">
            <span>✕</span> Delete Stack
          </button>
        </template>
        <!-- Stacks area context menu -->
        <template v-if="cfCtxMenu.type === 'area'">
          <button class="context-menu-item" @click="openDeployModal(); closeCfCtx()">
            <span>🚀</span> Deploy New Stack
          </button>
          <button class="context-menu-item" @click="validateTemplate(); closeCfCtx()">
            <span>✓</span> Validate Template
          </button>
          <div class="context-menu-divider"></div>
          <button class="context-menu-item" @click="fetchStacks(); closeCfCtx()">
            <span>↻</span> Refresh Stacks
          </button>
        </template>
      </div>
    </Teleport>

    <!-- Create Change Set Modal -->
    <ModalDialog :visible="showChangeSetModal" title="Create Change Set" size="sm" @close="showChangeSetModal = false">
      <div class="form-group" style="margin-bottom:0.75rem">
        <label>Change Set Name</label>
        <input v-model="newChangeSetName" type="text" class="glass-input" placeholder="my-update-preview" @keyup.enter="createNewChangeSet" />
      </div>
      <div class="form-group">
        <label>Description (optional)</label>
        <input v-model="newChangeSetDesc" type="text" class="glass-input" placeholder="What's changing..." />
      </div>
      <template #footer>
        <button class="btn-secondary" @click="showChangeSetModal = false">Cancel</button>
        <button class="btn-primary" @click="createNewChangeSet" :disabled="!newChangeSetName.trim()">Create</button>
      </template>
    </ModalDialog>

    <!-- Deploy Modal -->
    <ModalDialog :visible="showDeployModal" title="Deploy Stack" size="sm" @close="showDeployModal = false">
      <div class="deploy-form">
        <div class="form-group">
          <label>Stack Name</label>
          <input 
            v-model="deployStackName" 
            type="text" 
            class="glass-input" 
            placeholder="my-stack-name"
            @keyup.enter="confirmDeploy"
          />
          <span class="form-hint">Must be unique within your AWS account and region</span>
        </div>

        <div class="deploy-summary">
          <div class="deploy-summary-row">
            <span class="text-secondary text-sm">Account</span>
            <span class="text-sm">{{ activeAccountName }}</span>
          </div>
          <div class="deploy-summary-row">
            <span class="text-secondary text-sm">Region</span>
            <span class="text-sm">{{ activeAccountRegion }}</span>
          </div>
          <div class="deploy-summary-row">
            <span class="text-secondary text-sm">Template</span>
            <span class="text-sm">{{ templateYaml.split('\n').length }} lines</span>
          </div>
        </div>
      </div>

      <template #footer>
        <button class="btn-secondary" @click="showDeployModal = false">Cancel</button>
        <button 
          class="btn-primary" 
          @click="confirmDeploy" 
          :disabled="!deployStackName.trim() || isDeploying"
        >
          <svg v-if="isDeploying" width="14" height="14" viewBox="0 0 16 16" fill="none" class="spinning">
            <path d="M14 8A6 6 0 1 1 8 2" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
          {{ isDeploying ? 'Deploying...' : 'Confirm Deploy' }}
        </button>
      </template>
    </ModalDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, inject } from 'vue';
import { useAwsAccountsStore } from '../store/awsAccounts';
import { invoke } from '@tauri-apps/api/core';
import ModalDialog from '../components/ModalDialog.vue';

// CodeMirror
import { Codemirror } from 'vue-codemirror';
import { yaml } from '@codemirror/lang-yaml';
import { oneDark } from '@codemirror/theme-one-dark';

const accountsStore = useAwsAccountsStore();
const toast = inject<any>('toast');
const viewMode = ref<'builder' | 'code'>('code');
const activeTip = ref<string | null>(null);

function toggleTip(id: string) {
  activeTip.value = activeTip.value === id ? null : id;
}

// Resource types registry
const resourceTypes = [
  { id: 's3', label: 'S3 Bucket', icon: '<svg width="18" height="18" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3"><path d="M2 4C2 4 4 2 8 2C12 2 14 4 14 4V12C14 12 12 14 8 14C4 14 2 12 2 12V4Z"/><path d="M2 4C2 4 4 6 8 6C12 6 14 4 14 4"/><path d="M2 8C2 8 4 10 8 10C12 10 14 8 14 8"/></svg>', cfType: 'AWS::S3::Bucket', shortDesc: 'Object storage', description: 'Amazon S3 provides scalable object storage with high durability. Use it for static assets, backups, data lakes, and application data.' },
  { id: 'sqs', label: 'SQS Queue', icon: '<svg width="18" height="18" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3"><rect x="2" y="4" width="12" height="8" rx="1.5"/><path d="M5 7H11" stroke-linecap="round"/><path d="M5 9H9" stroke-linecap="round"/></svg>', cfType: 'AWS::SQS::Queue', shortDesc: 'Message queue', description: 'Amazon SQS is a fully managed message queuing service for decoupling and scaling microservices, distributed systems, and serverless applications.' },
  { id: 'sns', label: 'SNS Topic', icon: '<svg width="18" height="18" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3"><path d="M8 2V10" stroke-linecap="round"/><path d="M5 5L8 2L11 5" stroke-linecap="round" stroke-linejoin="round"/><circle cx="8" cy="12" r="2"/><path d="M4 8C3 9 3 11 4 12" stroke-linecap="round"/><path d="M12 8C13 9 13 11 12 12" stroke-linecap="round"/></svg>', cfType: 'AWS::SNS::Topic', shortDesc: 'Pub/sub messaging', description: 'Amazon SNS is a pub/sub messaging service for sending notifications to multiple subscribers via HTTP, email, SMS, SQS, or Lambda.' },
];

const currentResourceType = computed(() => resourceTypes.find(r => r.id === builderForm.value.type));

// Editor config
const templateYaml = ref(`AWSTemplateFormatVersion: '2010-09-09'
Description: 'Easy Cloud - Basic S3 Bucket'
Resources:
  MyS3Bucket:
    Type: 'AWS::S3::Bucket'
    Properties:
      BucketName: !Sub "easycloud-bucket-\${AWS::AccountId}-\${AWS::Region}"
`);
const extensions = [yaml(), oneDark];

const builderForm = ref({
  type: 's3',
  logicalId: 'MyResource',
  s3: {
    bucketName: '',
    accessControl: '',
    versioning: '',
    encryption: '',
    blockPublicAccess: 'all',
    deletionPolicy: '',
    websiteEnabled: '',
    indexDocument: 'index.html',
    errorDocument: '',
  },
  sqs: {
    queueName: '',
    fifoQueue: '',
    visibilityTimeout: '',
    messageRetention: '',
    delaySeconds: '',
    maxMessageSize: '',
    maxReceiveCount: '',
  },
  sns: {
    topicName: '',
    displayName: '',
    fifoTopic: '',
    kmsMasterKeyId: '',
  },
});

function selectResourceType(id: string) {
  builderForm.value.type = id;
  activeTip.value = null;
  generateYaml();
}

function sanitizeLogicalId() {
  builderForm.value.logicalId = builderForm.value.logicalId.replace(/[^a-zA-Z0-9]/g, '');
}

const generateYaml = () => {
  // Logical ID must be alphanumeric only (no hyphens, underscores, or spaces)
  const rawId = builderForm.value.logicalId || 'MyResource';
  const id = rawId.replace(/[^a-zA-Z0-9]/g, '');
  let lines: string[] = [
    "AWSTemplateFormatVersion: '2010-09-09'",
    "Description: 'Generated by Easy Cloud Builder'",
    "Resources:",
    `  ${id || 'MyResource'}:`,
  ];

  if (builderForm.value.type === 's3') {
    const s3 = builderForm.value.s3;
    if (s3.deletionPolicy) {
      lines.push(`    DeletionPolicy: ${s3.deletionPolicy}`);
    }
    lines.push("    Type: 'AWS::S3::Bucket'");
    const props: string[] = [];
    if (s3.bucketName) props.push(`      BucketName: '${s3.bucketName}'`);
    if (s3.accessControl) props.push(`      AccessControl: ${s3.accessControl}`);
    if (s3.versioning) {
      props.push("      VersioningConfiguration:");
      props.push(`        Status: ${s3.versioning}`);
    }
    if (s3.encryption) {
      props.push("      BucketEncryption:");
      props.push("        ServerSideEncryptionConfiguration:");
      props.push("          - ServerSideEncryptionByDefault:");
      props.push(`              SSEAlgorithm: ${s3.encryption}`);
    }
    if (s3.blockPublicAccess === 'all') {
      props.push("      PublicAccessBlockConfiguration:");
      props.push("        BlockPublicAcls: true");
      props.push("        BlockPublicPolicy: true");
      props.push("        IgnorePublicAcls: true");
      props.push("        RestrictPublicBuckets: true");
    }
    if (s3.websiteEnabled) {
      props.push("      WebsiteConfiguration:");
      props.push(`        IndexDocument: ${s3.indexDocument || 'index.html'}`);
      if (s3.errorDocument) {
        props.push(`        ErrorDocument: ${s3.errorDocument}`);
      }
    }
    if (props.length > 0) {
      lines.push("    Properties:");
      lines.push(...props);
    }
  } else if (builderForm.value.type === 'sqs') {
    const sqs = builderForm.value.sqs;
    lines.push("    Type: 'AWS::SQS::Queue'");
    const props: string[] = [];
    if (sqs.queueName) props.push(`      QueueName: '${sqs.queueName}'`);
    if (sqs.fifoQueue) {
      props.push("      FifoQueue: true");
      props.push("      ContentBasedDeduplication: true");
    }
    if (sqs.visibilityTimeout) props.push(`      VisibilityTimeout: ${sqs.visibilityTimeout}`);
    if (sqs.messageRetention) props.push(`      MessageRetentionPeriod: ${sqs.messageRetention}`);
    if (sqs.delaySeconds) props.push(`      DelaySeconds: ${sqs.delaySeconds}`);
    if (sqs.maxMessageSize) props.push(`      MaximumMessageSize: ${sqs.maxMessageSize}`);
    if (props.length > 0) {
      lines.push("    Properties:");
      lines.push(...props);
    }
  } else if (builderForm.value.type === 'sns') {
    const sns = builderForm.value.sns;
    lines.push("    Type: 'AWS::SNS::Topic'");
    const props: string[] = [];
    if (sns.topicName) props.push(`      TopicName: '${sns.topicName}'`);
    if (sns.displayName) props.push(`      DisplayName: '${sns.displayName}'`);
    if (sns.fifoTopic) {
      props.push("      FifoTopic: true");
      props.push("      ContentBasedDeduplication: true");
    }
    if (sns.kmsMasterKeyId) props.push(`      KmsMasterKeyId: '${sns.kmsMasterKeyId}'`);
    if (props.length > 0) {
      lines.push("    Properties:");
      lines.push(...props);
    }
  }

  templateYaml.value = lines.join('\n') + '\n';
};

// Stacks
interface CfStack {
  name: string;
  status: string;
  creation_time: string | null;
}
const stacks = ref<CfStack[]>([]);
const loadingStacks = ref(false);
const stacksError = ref('');

// Stack Detail
const selectedStack = ref<string | null>(null);
const detailTab = ref<'overview' | 'resources' | 'outputs' | 'events' | 'template' | 'drift' | 'changes'>('overview');
const stackDetails = ref<any>(null);
const loadingDetails = ref(false);
const stackResources = ref<any[]>([]);
const loadingResources = ref(false);
const stackEvents = ref<any[]>([]);
const loadingEvents = ref(false);
const validating = ref(false);

// Template tab
const stackTemplateBody = ref('');
const loadingTemplate = ref(false);

// Drift tab
const driftDetecting = ref(false);
const driftDetectionId = ref('');
const driftStatus = ref('');
const driftResources = ref<any[]>([]);

// Change Sets tab
const changeSets = ref<any[]>([]);
const loadingChangeSets = ref(false);
const expandedChangeSet = ref<string | null>(null);
const changeSetChanges = ref<any[]>([]);
const showChangeSetModal = ref(false);
const newChangeSetName = ref('');
const newChangeSetDesc = ref('');

// Context Menu
const cfCtxMenu = ref<{ visible: boolean; type: 'stack' | 'area'; target: any; x: number; y: number }>({
  visible: false, type: 'area', target: null, x: 0, y: 0
});
const cfCtxMenuStyle = computed(() => {
  const m = cfCtxMenu.value;
  let x = m.x, y = m.y;
  if (x + 200 > window.innerWidth) x = window.innerWidth - 208;
  if (y + 250 > window.innerHeight) y = window.innerHeight - 258;
  return { left: `${x}px`, top: `${y}px` };
});

// Deploy
const showDeployModal = ref(false);
const deployStackName = ref('');
const isDeploying = ref(false);

// Deploy Tracker
interface StackEvent {
  event_id: string;
  logical_resource_id: string | null;
  physical_resource_id: string | null;
  resource_type: string | null;
  resource_status: string | null;
  resource_status_reason: string | null;
  timestamp: string | null;
}

const deployTrackerVisible = ref(false);
const trackedStackName = ref('');
const trackedOperation = ref<'Deploy' | 'Update' | 'Delete'>('Deploy');
const deployEvents = ref<StackEvent[]>([]);
const deployStartTime = ref<number>(0);
const deployElapsedTime = ref('0s');
let pollingInterval: ReturnType<typeof setInterval> | null = null;
let elapsedInterval: ReturnType<typeof setInterval> | null = null;

const deployInProgress = computed(() => {
  if (deployEvents.value.length === 0) return true;
  // Find the stack-level event (most recent for this stack)
  const stackEvent = deployEvents.value.find(
    e => e.resource_type === 'AWS::CloudFormation::Stack'
      && (e.logical_resource_id === trackedStackName.value || e.physical_resource_id?.includes(trackedStackName.value))
  );
  if (!stackEvent) return true;
  const status = stackEvent.resource_status || '';
  // Terminal states: anything that ends in COMPLETE, FAILED, or ROLLBACK_COMPLETE
  const isTerminal = (status.endsWith('_COMPLETE') || status.endsWith('_FAILED') || status === 'DELETE_COMPLETE');
  return !isTerminal;
});

const deployOverallStatus = computed(() => {
  if (deployEvents.value.length === 0) return 'Initiating...';
  const stackEvent = deployEvents.value.find(
    e => e.resource_type === 'AWS::CloudFormation::Stack'
      && (e.logical_resource_id === trackedStackName.value || e.physical_resource_id?.includes(trackedStackName.value))
  );
  if (!stackEvent) return 'In Progress';
  return formatEventStatus(stackEvent.resource_status || '');
});

const deployOverallStatusClass = computed(() => {
  const stackEvent = deployEvents.value.find(
    e => e.resource_type === 'AWS::CloudFormation::Stack'
      && (e.logical_resource_id === trackedStackName.value || e.physical_resource_id?.includes(trackedStackName.value))
  );
  if (!stackEvent) return 'badge--warning';
  return eventStatusClass(stackEvent.resource_status || '');
});

const groupedDeployEvents = computed(() => {
  // Group by logical_resource_id, keep the most recent event per resource
  const map = new Map<string, StackEvent>();
  // Events come newest first, so iterate reversed to keep the latest
  for (const event of deployEvents.value) {
    const id = event.logical_resource_id || 'unknown';
    if (!map.has(id)) {
      map.set(id, event);
    }
  }
  return Array.from(map.values());
});

const totalResources = computed(() => {
  // Count unique resources excluding the stack itself
  return groupedDeployEvents.value.filter(
    e => e.resource_type !== 'AWS::CloudFormation::Stack'
  ).length || 1;
});

const completedResources = computed(() => {
  return groupedDeployEvents.value.filter(e => {
    if (e.resource_type === 'AWS::CloudFormation::Stack') return false;
    const s = e.resource_status || '';
    return s.includes('COMPLETE') || s.includes('FAILED');
  }).length;
});

const deployProgressPercent = computed(() => {
  if (totalResources.value === 0) return 0;
  return Math.round((completedResources.value / totalResources.value) * 100);
});

const deployProgressBarClass = computed(() => {
  const hasFailed = deployEvents.value.some(e => {
    const s = e.resource_status || '';
    return s.includes('FAILED') || s.includes('ROLLBACK');
  });
  if (hasFailed) return 'progress-error';
  if (!deployInProgress.value) return 'progress-success';
  return 'progress-active';
});

function isEventSuccess(status: string | null) {
  if (!status) return false;
  return status.includes('COMPLETE') && !status.includes('ROLLBACK');
}

function isEventFailed(status: string | null) {
  if (!status) return false;
  return status.includes('FAILED') || status.includes('ROLLBACK');
}

function eventStatusClass(status: string | null) {
  if (!status) return 'badge--neutral';
  if (isEventSuccess(status)) return 'badge--success';
  if (isEventFailed(status)) return 'badge--error';
  if (status.includes('IN_PROGRESS')) return 'badge--warning';
  return 'badge--neutral';
}

function eventItemClass(status: string | null) {
  if (!status) return '';
  if (isEventSuccess(status)) return 'event-item--success';
  if (isEventFailed(status)) return 'event-item--failed';
  if (status.includes('IN_PROGRESS')) return 'event-item--progress';
  return '';
}

function formatEventStatus(status: string | null) {
  if (!status) return '';
  return status.replace(/_/g, ' ').toLowerCase().replace(/\b\w/g, c => c.toUpperCase());
}

function formatResourceType(type: string | null) {
  if (!type) return '';
  return type.replace('AWS::', '').replace(/::/g, ' › ');
}

function formatEventTime(timestamp: string | null) {
  if (!timestamp) return '';
  const d = new Date(timestamp);
  return d.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
}

async function pollStackEvents() {
  const activeId = accountsStore.activeAccountId;
  if (!activeId) return;
  const acc = accountsStore.accounts.find(a => a.id === activeId);
  if (!acc) return;

  try {
    const res: any = await invoke('describe_stack_events', {
      accessKeyId: acc.accessKeyId,
      secretAccessKey: acc.secretAccessKey,
      region: acc.region,
      stackName: trackedStackName.value,
    });

    if (res.success && res.events) {
      deployEvents.value = res.events;

      // Check if operation finished
      if (!deployInProgress.value) {
        stopPolling();
        fetchStacks();
      }
    } else if (!res.success) {
      // Stack not found — likely deleted successfully
      if (trackedOperation.value === 'Delete') {
        // Mark as completed
        deployEvents.value = [{
          event_id: 'delete-complete',
          logical_resource_id: trackedStackName.value,
          physical_resource_id: null,
          resource_type: 'AWS::CloudFormation::Stack',
          resource_status: 'DELETE_COMPLETE',
          resource_status_reason: 'Stack successfully deleted',
          timestamp: new Date().toISOString(),
        }];
        stopPolling();
        fetchStacks();
      }
    }
  } catch (err) {
    // If we get an error during delete polling, stack is likely gone
    if (trackedOperation.value === 'Delete') {
      deployEvents.value = [{
        event_id: 'delete-complete',
        logical_resource_id: trackedStackName.value,
        physical_resource_id: null,
        resource_type: 'AWS::CloudFormation::Stack',
        resource_status: 'DELETE_COMPLETE',
        resource_status_reason: 'Stack successfully deleted',
        timestamp: new Date().toISOString(),
      }];
      stopPolling();
      fetchStacks();
    }
  }
}

function startPolling(stackName: string, operation: 'Deploy' | 'Update' | 'Delete' = 'Deploy') {
  trackedStackName.value = stackName;
  trackedOperation.value = operation;
  deployEvents.value = [];
  deployTrackerVisible.value = true;
  deployStartTime.value = Date.now();

  // Update elapsed time every second
  elapsedInterval = setInterval(() => {
    const elapsed = Math.floor((Date.now() - deployStartTime.value) / 1000);
    if (elapsed < 60) {
      deployElapsedTime.value = `${elapsed}s`;
    } else {
      const min = Math.floor(elapsed / 60);
      const sec = elapsed % 60;
      deployElapsedTime.value = `${min}m ${sec}s`;
    }
  }, 1000);

  // Poll events every 3 seconds
  pollStackEvents();
  pollingInterval = setInterval(pollStackEvents, 3000);
}

function stopPolling() {
  if (pollingInterval) {
    clearInterval(pollingInterval);
    pollingInterval = null;
  }
  if (elapsedInterval) {
    clearInterval(elapsedInterval);
    elapsedInterval = null;
  }
}

function closeTracker() {
  stopPolling();
  deployTrackerVisible.value = false;
}

onUnmounted(() => {
  stopPolling();
});

const activeAccountName = computed(() => {
  const acc = accountsStore.accounts.find(a => a.id === accountsStore.activeAccountId);
  return acc?.name || '—';
});

const activeAccountRegion = computed(() => {
  const acc = accountsStore.accounts.find(a => a.id === accountsStore.activeAccountId);
  return acc?.region || '—';
});

onMounted(() => {
  if (accountsStore.activeAccountId) {
    fetchStacks();
  }
  document.addEventListener('click', () => { cfCtxMenu.value = { ...cfCtxMenu.value, visible: false }; });
});

watch(() => accountsStore.activeAccountId, (newId) => {
  if (newId) fetchStacks();
  else stacks.value = [];
});

const onTemplateChange = () => {};

const openDeployModal = () => {
  deployStackName.value = '';
  showDeployModal.value = true;
};

const confirmDeploy = async () => {
  if (!deployStackName.value.trim()) return;
  
  const activeId = accountsStore.activeAccountId;
  if (!activeId) return;
  const acc = accountsStore.accounts.find(a => a.id === activeId);
  if (!acc) return;

  isDeploying.value = true;
  
  try {
    const res: any = await invoke('deploy_cf_stack', {
      accessKeyId: acc.accessKeyId,
      secretAccessKey: acc.secretAccessKey,
      region: acc.region,
      stackName: deployStackName.value,
      templateBody: templateYaml.value
    });
    
    if (res.success) {
      toast?.success(`Stack "${deployStackName.value}" deployment initiated`, 'Deploy Started');
      showDeployModal.value = false;
      startPolling(deployStackName.value);
    } else {
      toast?.error(res.error_message || 'Deploy failed', 'Deploy Error');
    }
  } catch (err: any) {
    toast?.error(err.message || err.toString(), 'Deploy Error');
  } finally {
    isDeploying.value = false;
  }
};

const fetchStacks = async () => {
  const activeId = accountsStore.activeAccountId;
  if (!activeId) return;
  const acc = accountsStore.accounts.find(a => a.id === activeId);
  if (!acc) return;

  loadingStacks.value = true;
  stacksError.value = '';
  
  try {
    const res: any = await invoke('list_cf_stacks', {
      accessKeyId: acc.accessKeyId,
      secretAccessKey: acc.secretAccessKey,
      region: acc.region
    });
    
    if (res.success) {
      stacks.value = res.stacks || [];
    } else {
      stacksError.value = res.error_message || 'Failed to fetch CF Stacks.';
    }
  } catch (err: any) {
    stacksError.value = err.message || err.toString();
  } finally {
    loadingStacks.value = false;
  }
};

const formatDate = (dateStr: string | null) => {
  if (!dateStr) return '';
  const d = new Date(dateStr);
  return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
};

const formatStatus = (status: string) => {
  return status.replace(/_/g, ' ').toLowerCase().replace(/\b\w/g, c => c.toUpperCase());
};

const statusBadgeClass = (status: string) => {
  if (status.includes('COMPLETE') && !status.includes('DELETE')) return 'badge--success';
  if (status.includes('FAILED') || status.includes('ROLLBACK')) return 'badge--error';
  if (status.includes('IN_PROGRESS')) return 'badge--warning';
  return 'badge--neutral';
};

// Stack Detail functions
async function selectStack(name: string) {
  selectedStack.value = name;
  detailTab.value = 'overview';
  stackDetails.value = null;
  stackResources.value = [];
  stackEvents.value = [];
  loadStackDetails();
}

async function loadStackDetails() {
  if (!selectedStack.value) return;
  const acc = accountsStore.accounts.find(a => a.id === accountsStore.activeAccountId);
  if (!acc) return;
  loadingDetails.value = true;
  try {
    const res: any = await invoke('get_stack_details', {
      accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey,
      region: acc.region, stackName: selectedStack.value,
    });
    if (res.success) stackDetails.value = res;
  } catch (_) {}
  finally { loadingDetails.value = false; }
}

async function loadResources() {
  if (!selectedStack.value) return;
  const acc = accountsStore.accounts.find(a => a.id === accountsStore.activeAccountId);
  if (!acc) return;
  loadingResources.value = true;
  try {
    const res: any = await invoke('list_stack_resources', {
      accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey,
      region: acc.region, stackName: selectedStack.value,
    });
    if (res.success) stackResources.value = res.resources || [];
  } catch (_) {}
  finally { loadingResources.value = false; }
}

async function loadEvents() {
  if (!selectedStack.value) return;
  const acc = accountsStore.accounts.find(a => a.id === accountsStore.activeAccountId);
  if (!acc) return;
  loadingEvents.value = true;
  try {
    const res: any = await invoke('describe_stack_events', {
      accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey,
      region: acc.region, stackName: selectedStack.value,
    });
    if (res.success) stackEvents.value = res.events || [];
  } catch (_) {}
  finally { loadingEvents.value = false; }
}

async function deleteStack(name: string) {
  const acc = accountsStore.accounts.find(a => a.id === accountsStore.activeAccountId);
  if (!acc) return;
  try {
    const res: any = await invoke('delete_cf_stack', {
      accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey,
      region: acc.region, stackName: name,
    });
    if (res.success) {
      toast?.success(`Stack "${name}" delete initiated`, 'Delete Started');
      selectedStack.value = null;
      startPolling(name, 'Delete');
    } else {
      toast?.error(res.error_message || 'Delete failed', 'Error');
    }
  } catch (e: any) { toast?.error(e.message || e.toString(), 'Error'); }
}

function confirmDeleteStack() {
  if (!selectedStack.value) return;
  if (confirm(`Are you sure you want to delete stack "${selectedStack.value}"? This will destroy all resources in the stack.`)) {
    deleteStack(selectedStack.value);
  }
}

async function updateSelectedStack() {
  if (!selectedStack.value || !templateYaml.value.trim()) return;
  const acc = accountsStore.accounts.find(a => a.id === accountsStore.activeAccountId);
  if (!acc) return;
  try {
    const res: any = await invoke('update_cf_stack', {
      accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey,
      region: acc.region, stackName: selectedStack.value,
      templateBody: templateYaml.value,
    });
    if (res.success) {
      toast?.success(`Stack "${selectedStack.value}" update initiated`, 'Update Started');
      startPolling(selectedStack.value, 'Update');
    } else {
      toast?.error(res.error_message || 'Update failed', 'Error');
    }
  } catch (e: any) { toast?.error(e.message || e.toString(), 'Error'); }
}

async function validateTemplate() {
  const acc = accountsStore.accounts.find(a => a.id === accountsStore.activeAccountId);
  if (!acc || !templateYaml.value.trim()) return;
  validating.value = true;
  try {
    const res: any = await invoke('validate_cf_template', {
      accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey,
      region: acc.region, templateBody: templateYaml.value,
    });
    if (res.valid) {
      toast?.success('Template is valid', 'Validation Passed');
    } else {
      toast?.error(res.error_message || 'Template is invalid', 'Validation Failed');
    }
  } catch (e: any) { toast?.error(e.message || e.toString(), 'Validation Error'); }
  finally { validating.value = false; }
}

function formatDateFull(dateStr: string | null | undefined) {
  if (!dateStr) return '—';
  const d = new Date(dateStr);
  return d.toLocaleString('en-US', { month: 'short', day: 'numeric', year: 'numeric', hour: '2-digit', minute: '2-digit' });
}

function formatTime(dateStr: string | null | undefined) {
  if (!dateStr) return '—';
  const d = new Date(dateStr);
  return d.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
}

function eventRowClass(status: string | null | undefined) {
  if (!status) return '';
  if (status.includes('FAILED') || status.includes('ROLLBACK')) return 'event-row--error';
  if (status.includes('COMPLETE')) return 'event-row--success';
  return '';
}

// Context Menu
function openStackCtx(event: MouseEvent, stack: any) {
  cfCtxMenu.value = { visible: true, type: 'stack', target: stack, x: event.clientX, y: event.clientY };
}
function openStacksAreaCtx(event: MouseEvent) {
  cfCtxMenu.value = { visible: true, type: 'area', target: null, x: event.clientX, y: event.clientY };
}
function closeCfCtx() {
  cfCtxMenu.value = { ...cfCtxMenu.value, visible: false };
}

// Template tab
async function loadStackTemplate() {
  if (!selectedStack.value) return;
  const acc = accountsStore.accounts.find(a => a.id === accountsStore.activeAccountId);
  if (!acc) return;
  loadingTemplate.value = true;
  try {
    const res: any = await invoke('get_stack_template', {
      accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey,
      region: acc.region, stackName: selectedStack.value,
    });
    if (res.success) stackTemplateBody.value = res.template_body || '';
  } catch (_) {}
  finally { loadingTemplate.value = false; }
}

function loadToEditor() {
  if (stackTemplateBody.value) {
    templateYaml.value = stackTemplateBody.value;
    toast?.success('Template loaded into editor', 'Done');
  }
}

async function estimateCost() {
  const acc = accountsStore.accounts.find(a => a.id === accountsStore.activeAccountId);
  if (!acc) return;
  const body = stackTemplateBody.value || templateYaml.value;
  if (!body.trim()) return;
  try {
    const res: any = await invoke('estimate_template_cost', {
      accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey,
      region: acc.region, templateBody: body,
    });
    if (res.success && res.url) {
      window.open(res.url, '_blank');
    } else {
      toast?.error(res.error_message || 'Failed to estimate cost', 'Error');
    }
  } catch (e: any) { toast?.error(e.message || e.toString(), 'Error'); }
}

// Drift detection
async function startDriftDetection() {
  if (!selectedStack.value) return;
  const acc = accountsStore.accounts.find(a => a.id === accountsStore.activeAccountId);
  if (!acc) return;
  driftDetecting.value = true;
  driftResources.value = [];
  driftStatus.value = '';
  try {
    const res: any = await invoke('detect_stack_drift', {
      accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey,
      region: acc.region, stackName: selectedStack.value,
    });
    if (res.success && res.detection_id) {
      driftDetectionId.value = res.detection_id;
      toast?.info('Drift detection started. Checking status...', 'Drift');
      pollDriftStatus();
    } else {
      toast?.error(res.error_message || 'Failed', 'Drift Error');
      driftDetecting.value = false;
    }
  } catch (e: any) { toast?.error(e.message, 'Error'); driftDetecting.value = false; }
}

async function pollDriftStatus() {
  const acc = accountsStore.accounts.find(a => a.id === accountsStore.activeAccountId);
  if (!acc || !driftDetectionId.value) return;
  
  const check = async (): Promise<boolean> => {
    const res: any = await invoke('describe_drift_detection_status', {
      accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey,
      region: acc.region, detectionId: driftDetectionId.value,
    });
    if (res.success) {
      if (res.detection_status === 'DETECTION_COMPLETE') {
        driftStatus.value = res.drift_status || 'IN_SYNC';
        driftDetecting.value = false;
        loadDriftResults();
        return true;
      } else if (res.detection_status === 'DETECTION_FAILED') {
        driftStatus.value = 'FAILED';
        driftDetecting.value = false;
        return true;
      }
    }
    return false;
  };
  
  // Poll every 2s up to 30s
  for (let i = 0; i < 15; i++) {
    await new Promise(r => setTimeout(r, 2000));
    if (await check()) return;
  }
  driftDetecting.value = false;
  toast?.warning('Drift detection still in progress. Click "View Results" later.', 'Drift');
}

async function loadDriftResults() {
  if (!selectedStack.value) return;
  const acc = accountsStore.accounts.find(a => a.id === accountsStore.activeAccountId);
  if (!acc) return;
  try {
    const res: any = await invoke('describe_stack_resource_drifts', {
      accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey,
      region: acc.region, stackName: selectedStack.value,
    });
    if (res.success) driftResources.value = res.drifts || [];
  } catch (_) {}
}

function driftStatusClass(): string {
  if (driftStatus.value === 'IN_SYNC') return 'badge--success';
  if (driftStatus.value === 'DRIFTED') return 'badge--error';
  return 'badge--warning';
}

function driftRowClass(status: string | null | undefined): string {
  if (status === 'MODIFIED' || status === 'DELETED') return 'event-row--error';
  if (status === 'IN_SYNC') return '';
  return '';
}

function driftBadgeClass(status: string | null | undefined): string {
  if (status === 'IN_SYNC') return 'badge--success';
  if (status === 'MODIFIED' || status === 'DELETED') return 'badge--error';
  return 'badge--warning';
}

// Change Sets
async function loadChangeSets() {
  if (!selectedStack.value) return;
  const acc = accountsStore.accounts.find(a => a.id === accountsStore.activeAccountId);
  if (!acc) return;
  loadingChangeSets.value = true;
  try {
    const res: any = await invoke('list_change_sets', {
      accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey,
      region: acc.region, stackName: selectedStack.value,
    });
    if (res.success) changeSets.value = res.change_sets || [];
  } catch (_) {}
  finally { loadingChangeSets.value = false; }
}

function openCreateChangeSet() {
  newChangeSetName.value = `update-${Date.now().toString(36)}`;
  newChangeSetDesc.value = '';
  showChangeSetModal.value = true;
}

async function createNewChangeSet() {
  if (!selectedStack.value || !newChangeSetName.value.trim()) return;
  const acc = accountsStore.accounts.find(a => a.id === accountsStore.activeAccountId);
  if (!acc) return;
  try {
    const res: any = await invoke('create_change_set', {
      accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey,
      region: acc.region, stackName: selectedStack.value,
      changeSetName: newChangeSetName.value.trim(),
      templateBody: templateYaml.value,
      description: newChangeSetDesc.value || null,
    });
    if (res.success) {
      toast?.success('Change set created', 'Done');
      showChangeSetModal.value = false;
      setTimeout(() => loadChangeSets(), 2000); // give AWS time to process
    } else {
      toast?.error(res.error_message || 'Failed', 'Error');
    }
  } catch (e: any) { toast?.error(e.message, 'Error'); }
}

async function viewChangeSet(name: string) {
  if (expandedChangeSet.value === name) { expandedChangeSet.value = null; return; }
  if (!selectedStack.value) return;
  const acc = accountsStore.accounts.find(a => a.id === accountsStore.activeAccountId);
  if (!acc) return;
  try {
    const res: any = await invoke('describe_change_set', {
      accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey,
      region: acc.region, stackName: selectedStack.value, changeSetName: name,
    });
    if (res.success) {
      changeSetChanges.value = res.changes || [];
      expandedChangeSet.value = name;
    }
  } catch (_) {}
}

async function executeChangeSetAction(name: string) {
  if (!selectedStack.value) return;
  if (!confirm(`Execute change set "${name}"? This will apply the changes to your stack.`)) return;
  const acc = accountsStore.accounts.find(a => a.id === accountsStore.activeAccountId);
  if (!acc) return;
  try {
    const res: any = await invoke('execute_change_set', {
      accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey,
      region: acc.region, stackName: selectedStack.value, changeSetName: name,
    });
    if (res.success) {
      toast?.success('Change set executing', 'Done');
      startPolling(selectedStack.value, 'Update');
    } else toast?.error(res.error_message || 'Failed', 'Error');
  } catch (e: any) { toast?.error(e.message, 'Error'); }
}

async function deleteChangeSetAction(name: string) {
  if (!selectedStack.value) return;
  const acc = accountsStore.accounts.find(a => a.id === accountsStore.activeAccountId);
  if (!acc) return;
  try {
    const res: any = await invoke('delete_change_set', {
      accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey,
      region: acc.region, stackName: selectedStack.value, changeSetName: name,
    });
    if (res.success) { toast?.info('Change set deleted', 'Done'); loadChangeSets(); }
    else toast?.error(res.error_message || 'Failed', 'Error');
  } catch (e: any) { toast?.error(e.message, 'Error'); }
}
</script>

<style scoped>
.cf-view {
  height: 100%;
  display: flex;
  flex-direction: column;
}

/* Header */
.page-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 1.5rem;
}

.page-header h1 {
  margin-bottom: 0.25rem;
}

.page-actions {
  display: flex;
  gap: 0.5rem;
}

/* Empty State */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 4rem 2rem;
  gap: 1rem;
  flex: 1;
}

.empty-icon {
  color: var(--text-tertiary);
}

/* Layout */
.cf-layout {
  display: flex;
  gap: 1rem;
  flex: 1;
  min-height: 0;
}

/* Stacks Panel */
.stacks-panel {
  width: 280px;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
  padding: 1.25rem;
  overflow: hidden;
}

.stacks-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.stacks-header h3 {
  margin: 0;
}

.stacks-loading {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.stack-skeleton {
  padding: 0.75rem;
  border-radius: var(--radius-sm);
  background: var(--bg-secondary);
}

.stacks-empty {
  padding: 2rem 1rem;
  text-align: center;
}

.stacks-list {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  overflow-y: auto;
  flex: 1;
}

.stack-item {
  padding: 0.75rem;
  border-radius: var(--radius-sm);
  background: var(--bg-secondary);
  border: 1px solid transparent;
  transition: all var(--transition-fast);
  cursor: pointer;
}

.stack-item:hover {
  border-color: var(--border-default);
}

.stack-item--active {
  border-color: var(--accent);
  background: var(--accent-subtle);
}

.stack-name {
  font-size: 0.8125rem;
  font-weight: 500;
  margin-bottom: 0.375rem;
  word-break: break-all;
}

.stack-meta {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

/* Editor Panel */
.editor-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
  overflow: hidden;
  background: var(--bg-primary);
}

.editor-tabs {
  display: flex;
  border-bottom: 1px solid var(--border-default);
  padding: 0 0.5rem;
}

.tab {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  background: none;
  border: none;
  color: var(--text-tertiary);
  font-size: 0.8125rem;
  font-weight: 450;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  transition: all var(--transition-fast);
}

.tab:hover {
  color: var(--text-secondary);
}

.tab--active {
  color: var(--text-primary);
  border-bottom-color: var(--text-primary);
}

.editor-content {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

:deep(.cm-editor) {
  height: 100%;
  background: var(--bg-primary) !important;
}

:deep(.cm-scroller) {
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 0.8125rem;
}

:deep(.cm-gutters) {
  background: var(--bg-primary) !important;
  border-right: 1px solid var(--border-default) !important;
}

/* Builder */
.builder-content {
  flex: 1;
  overflow-y: auto;
}

.builder-layout {
  display: flex;
  height: 100%;
}

.builder-sidebar {
  width: 200px;
  border-right: 1px solid var(--border-default);
  padding: 1.25rem 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  overflow-y: auto;
}

.builder-sidebar h3 {
  font-size: 0.6875rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-tertiary);
  font-weight: 500;
  padding: 0 0.5rem;
  margin: 0;
}

.resource-type-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.resource-type-btn {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding: 0.5rem 0.625rem;
  background: none;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  cursor: pointer;
  text-align: left;
  transition: all var(--transition-fast);
  width: 100%;
}

.resource-type-btn:hover {
  background: var(--bg-hover);
}

.resource-type-btn--active {
  background: var(--bg-elevated);
  border-color: var(--border-default);
}

.rt-icon {
  font-size: 1.125rem;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: var(--text-primary);
}

.rt-info {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.rt-name {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--text-primary);
}

.rt-desc {
  font-size: 0.625rem;
  color: var(--text-tertiary);
}

/* Config Panel */
.builder-config {
  flex: 1;
  padding: 1.5rem;
  overflow-y: auto;
}

.config-header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 0.25rem;
}

.config-header h3 {
  margin: 0;
}

.config-desc {
  margin-bottom: 1.5rem;
}

.config-section {
  margin-bottom: 1.5rem;
}

.config-section-title {
  font-size: 0.6875rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-tertiary);
  font-weight: 500;
  margin-bottom: 0.75rem;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid var(--border-default);
}

.field-group {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  max-width: 400px;
}

.field-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.field-header label {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--text-secondary);
}

.optional {
  color: var(--text-tertiary);
  font-weight: 400;
}

.tip-btn {
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: 50%;
  color: var(--text-tertiary);
  font-size: 0.625rem;
  font-weight: 700;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.tip-btn:hover {
  color: var(--text-primary);
  border-color: var(--border-hover);
  background: var(--bg-hover);
}

.tip-box {
  padding: 0.75rem;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  color: var(--text-secondary);
  line-height: 1.6;
  animation: tipFadeIn 0.15s ease;
}

.tip-box strong {
  color: var(--text-primary);
}

.tip-box code {
  background: var(--bg-secondary);
  border: 1px solid var(--border-default);
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.6875rem;
}

.tip-box em {
  color: var(--text-primary);
  font-style: normal;
  font-weight: 500;
}

@keyframes tipFadeIn {
  from { opacity: 0; transform: translateY(-4px); }
  to { opacity: 1; transform: translateY(0); }
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  margin-bottom: 1rem;
}

.form-group label {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--text-secondary);
}

.form-hint {
  font-size: 0.6875rem;
  color: var(--text-tertiary);
}

.builder-preview {
  margin-top: 1.5rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.builder-preview-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.625rem 0.875rem;
  border-bottom: 1px solid var(--border-default);
}

.preview-code {
  padding: 1rem;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.75rem;
  color: var(--text-secondary);
  line-height: 1.7;
  overflow-x: auto;
  white-space: pre;
  margin: 0;
}

/* Deploy Modal */
.deploy-form {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.deploy-summary {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 0.875rem;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
}

.deploy-summary-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

/* Error Banner */
.error-banner {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.625rem 0.875rem;
  background: var(--error-subtle);
  border: 1px solid rgba(238, 0, 0, 0.15);
  border-radius: var(--radius-sm);
  color: #ff6166;
  font-size: 0.8125rem;
}

/* Utilities */
.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* Deploy Tracker */
.deploy-tracker-overlay {
  position: fixed;
  bottom: 1.5rem;
  right: 1.5rem;
  z-index: 1000;
  width: 520px;
  max-height: 70vh;
  display: flex;
  flex-direction: column;
  animation: trackerSlideIn 0.2s ease;
}

@keyframes trackerSlideIn {
  from { opacity: 0; transform: translateY(12px); }
  to { opacity: 1; transform: translateY(0); }
}

.deploy-tracker {
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3), 0 2px 8px rgba(0, 0, 0, 0.2);
  overflow: hidden;
  max-height: 70vh;
}

.deploy-tracker-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.875rem 1.25rem;
  border-bottom: 1px solid var(--border-default);
  background: var(--bg-secondary);
}

.deploy-tracker-title {
  display: flex;
  align-items: center;
  gap: 0.625rem;
}

.deploy-tracker-title h3 {
  margin: 0;
  font-size: 0.875rem;
  font-weight: 600;
}

.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: none;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  color: var(--text-tertiary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-icon:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
  border-color: var(--border-default);
}

.deploy-tracker-body {
  padding: 1rem 1.25rem;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 0.875rem;
}

/* Progress Bar */
.deploy-progress-bar {
  height: 4px;
  background: var(--bg-elevated);
  border-radius: 2px;
  overflow: hidden;
}

.deploy-progress-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.5s ease;
}

.progress-active {
  background: var(--accent-primary, #3b82f6);
  animation: progressPulse 1.5s ease infinite;
}

.progress-success {
  background: #22c55e;
}

.progress-error {
  background: #ef4444;
}

@keyframes progressPulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}

.deploy-progress-label {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

/* Events */
.deploy-events-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.deploy-event-item {
  display: flex;
  align-items: flex-start;
  gap: 0.625rem;
  padding: 0.5rem 0.625rem;
  border-radius: var(--radius-sm);
  border: 1px solid transparent;
  transition: all var(--transition-fast);
}

.deploy-event-item:hover {
  background: var(--bg-hover);
}

.event-item--success .deploy-event-icon {
  color: #22c55e;
}

.event-item--failed .deploy-event-icon {
  color: #ef4444;
}

.event-item--progress .deploy-event-icon {
  color: var(--accent-primary, #3b82f6);
}

.deploy-event-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  flex-shrink: 0;
  margin-top: 1px;
}

.deploy-event-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.deploy-event-resource {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.deploy-event-logical-id {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--text-primary);
}

.deploy-event-type {
  font-size: 0.6875rem;
}

.deploy-event-status {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.badge-sm {
  font-size: 0.625rem;
  padding: 0.125rem 0.375rem;
}

.deploy-event-reason {
  font-size: 0.6875rem;
  color: var(--text-tertiary);
  word-break: break-word;
}

.deploy-event-time {
  flex-shrink: 0;
  font-size: 0.6875rem;
}

/* Stack Detail Panel */
.detail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.875rem 1.25rem;
  border-bottom: 1px solid var(--border-default);
}

.detail-header-left {
  display: flex;
  align-items: center;
  gap: 0.625rem;
}

.detail-header-left h3 {
  margin: 0;
  font-size: 0.875rem;
}

.btn-back-sm {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-back-sm:hover {
  color: var(--text-primary);
  border-color: var(--border-hover);
}

.detail-actions {
  display: flex;
  gap: 0.5rem;
}

.detail-content {
  flex: 1;
  overflow-y: auto;
  padding: 1.25rem;
}

.detail-loading, .detail-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  color: var(--text-tertiary);
  font-size: 0.8125rem;
}

.detail-grid {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.detail-field {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.detail-label {
  font-size: 0.6875rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-tertiary);
  font-weight: 500;
}

.detail-value {
  font-size: 0.8125rem;
  color: var(--text-primary);
  word-break: break-all;
}

.detail-value.mono {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.75rem;
}

.detail-value--warn {
  color: #f5a623;
}

.detail-section {
  margin-top: 1.5rem;
}

.detail-section h4 {
  font-size: 0.75rem;
  color: var(--text-secondary);
  margin-bottom: 0.625rem;
  padding-bottom: 0.375rem;
  border-bottom: 1px solid var(--border-default);
}

.detail-table {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.detail-table-row {
  display: flex;
  gap: 1rem;
  padding: 0.375rem 0;
  border-bottom: 1px solid var(--border-default);
}

.detail-table-row:last-child {
  border-bottom: none;
}

.detail-table-key {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--text-secondary);
  min-width: 140px;
}

.detail-table-val {
  font-size: 0.75rem;
  color: var(--text-primary);
  word-break: break-all;
}

.output-row {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

/* Resource Table */
.resource-table {
  display: flex;
  flex-direction: column;
}

.resource-table-header {
  display: grid;
  grid-template-columns: 1.2fr 1.2fr 0.8fr;
  gap: 0.5rem;
  padding: 0.5rem 0;
  font-size: 0.6875rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-tertiary);
  font-weight: 500;
  border-bottom: 1px solid var(--border-default);
}

.resource-table-row {
  display: grid;
  grid-template-columns: 1.2fr 1.2fr 0.8fr;
  gap: 0.5rem;
  padding: 0.5rem 0;
  border-bottom: 1px solid var(--border-default);
  align-items: center;
}

.resource-table-row:last-child {
  border-bottom: none;
}

.resource-logical-id {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--text-primary);
}

.resource-type {
  font-size: 0.75rem;
  color: var(--text-secondary);
  font-family: 'JetBrains Mono', monospace;
}

/* Events Table */
.events-table {
  display: flex;
  flex-direction: column;
  max-height: 500px;
  overflow-y: auto;
}

.event-row {
  display: grid;
  grid-template-columns: 80px 1fr auto;
  gap: 0.5rem;
  padding: 0.4rem 0;
  border-bottom: 1px solid var(--border-default);
  align-items: center;
  font-size: 0.75rem;
}

.event-row:last-child {
  border-bottom: none;
}

.event-row--error {
  background: rgba(238, 0, 0, 0.03);
}

.event-row--success {
  background: rgba(12, 206, 107, 0.02);
}

.event-time {
  color: var(--text-tertiary);
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.6875rem;
}

.event-resource {
  font-weight: 500;
  color: var(--text-secondary);
}

.event-reason {
  grid-column: 1 / -1;
  font-size: 0.6875rem;
  padding-left: 80px;
  padding-top: 0.125rem;
}

.badge-sm {
  font-size: 0.625rem;
  padding: 0.125rem 0.375rem;
}

/* Template Tab */
.template-actions, .drift-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.template-code {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.75rem;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--text-secondary);
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  padding: 1rem;
  margin: 0;
  max-height: 500px;
  overflow-y: auto;
}

/* Drift */
.drift-results { margin-top: 0.75rem; }

/* Change Sets */
.change-sets-list { display: flex; flex-direction: column; gap: 0.5rem; }
.change-set-card { padding: 0.75rem; border: 1px solid var(--border-default); border-radius: var(--radius-sm); background: var(--bg-secondary); }
.cs-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 0.25rem; }
.cs-name { font-size: 0.8125rem; font-weight: 500; color: var(--text-primary); }
.cs-actions { display: flex; gap: 0.375rem; margin-top: 0.5rem; }
.cs-changes { margin-top: 0.75rem; border-top: 1px solid var(--border-default); padding-top: 0.625rem; display: flex; flex-direction: column; gap: 0.25rem; }
.cs-change-row { display: flex; align-items: center; gap: 0.5rem; padding: 0.25rem 0; font-size: 0.75rem; }
.cs-action-badge { font-size: 0.625rem; font-weight: 600; padding: 0.125rem 0.375rem; border-radius: 3px; text-transform: uppercase; }
.cs-action--add { background: rgba(12, 206, 107, 0.1); color: #22c55e; }
.cs-action--modify { background: rgba(245, 166, 35, 0.1); color: #f5a623; }
.cs-action--remove { background: rgba(238, 0, 0, 0.1); color: #ef4444; }
.cs-change-id { font-weight: 500; color: var(--text-primary); }

/* Context Menu (CF) */
.context-menu { position: fixed; z-index: 9999; min-width: 190px; background: var(--bg-primary); border: 1px solid var(--border-default); border-radius: var(--radius-md); box-shadow: 0 8px 32px rgba(0,0,0,0.5); padding: 0.3rem; animation: ctxIn 0.1s ease; }
@keyframes ctxIn { from { opacity: 0; transform: scale(0.95); } to { opacity: 1; transform: scale(1); } }
.context-menu-item { display: flex; align-items: center; gap: 0.625rem; width: 100%; padding: 0.5rem 0.75rem; background: none; border: none; border-radius: var(--radius-sm); color: var(--text-secondary); font-size: 0.8125rem; cursor: pointer; transition: all var(--transition-fast); text-align: left; }
.context-menu-item:hover { background: var(--bg-hover); color: var(--text-primary); }
.context-menu-item--danger:hover { background: var(--error-subtle); color: #ff6166; }
.context-menu-item span { font-size: 0.9375rem; width: 18px; text-align: center; }
.context-menu-divider { height: 1px; background: var(--border-default); margin: 0.3rem 0; }
</style>
