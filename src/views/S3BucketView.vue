<template>
  <div class="bucket-view">
    <!-- Header -->
    <div class="page-header">
      <div class="header-left">
        <button class="btn-back" @click="$router.push('/s3')">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
            <path d="M10 4L6 8L10 12" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
          </svg>
        </button>
        <div>
          <h1>{{ bucketName }}</h1>
          <p class="text-secondary text-sm" v-if="bucketDetails">
            {{ bucketDetails.region }} · {{ bucketDetails.encryption || 'No encryption' }}
          </p>
        </div>
      </div>
      <div class="page-actions">
        <button class="btn-secondary" @click="openCreateFolder" v-if="activeTab === 'objects'">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
            <path d="M2 4V13H14V6H8L6 4H2Z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round"/>
          </svg>
          New Folder
        </button>
        <button class="btn-primary" @click="triggerUpload" v-if="activeTab === 'objects'">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
            <path d="M8 12V3M8 3L4 7M8 3L12 7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M2 14H14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
          Upload
        </button>
      </div>
    </div>

    <!-- Tabs -->
    <div class="tabs-bar">
      <button class="tab" :class="{ 'tab--active': activeTab === 'objects' }" @click="activeTab = 'objects'">Objects</button>
      <button class="tab" :class="{ 'tab--active': activeTab === 'permissions' }" @click="activeTab = 'permissions'; loadPolicy()">Permissions</button>
      <button class="tab" :class="{ 'tab--active': activeTab === 'properties' }" @click="activeTab = 'properties'">Properties</button>
      <button class="tab" :class="{ 'tab--active': activeTab === 'tags' }" @click="activeTab = 'tags'; loadTags()">Tags</button>
    </div>

    <!-- OBJECTS TAB -->
    <div v-show="activeTab === 'objects'" class="tab-content" @contextmenu.prevent="openAreaContextMenu($event)">
      <!-- Path bar -->
      <div class="path-bar">
        <button class="path-segment" @click="navigateTo('')">{{ bucketName }}</button>
        <template v-for="(segment, idx) in pathSegments" :key="idx">
          <span class="path-sep">/</span>
          <button class="path-segment" @click="navigateToSegment(idx)">{{ segment }}</button>
        </template>
      </div>

      <!-- Batch actions -->
      <div v-if="selectedKeys.length > 0" class="batch-bar">
        <span class="text-sm">{{ selectedKeys.length }} selected</span>
        <button class="btn-danger" @click="batchDelete">Delete Selected</button>
        <button class="btn-secondary" @click="selectedKeys = []">Clear</button>
      </div>

      <!-- Error -->
      <div v-if="error" class="error-banner">{{ error }}</div>

      <!-- Loading -->
      <div v-else-if="loadingObjects" class="file-table">
        <div class="file-table-header"><span>Name</span><span>Size</span><span>Modified</span><span></span></div>
        <div v-for="i in 5" :key="i" class="file-row">
          <div style="display:flex;align-items:center;gap:10px"><div class="skeleton" style="width:20px;height:20px;border-radius:4px"></div><div class="skeleton" style="width:180px;height:14px"></div></div>
          <div class="skeleton" style="width:50px;height:12px"></div>
          <div class="skeleton" style="width:80px;height:12px"></div>
          <div></div>
        </div>
      </div>

      <!-- Empty -->
      <div v-else-if="objects.length === 0" class="empty-folder" @contextmenu.prevent="openAreaContextMenu($event)">
        <p class="text-secondary text-sm">This folder is empty</p>
      </div>

      <!-- File Table -->
      <div v-else class="file-table" @contextmenu.prevent="openAreaContextMenu($event)">
        <div class="file-table-header"><span>Name</span><span>Size</span><span>Modified</span><span>Actions</span></div>
        <div 
          v-for="obj in objects" 
          :key="obj.key" 
          class="file-row" 
          :class="{ 'file-row--folder': obj.is_folder, 'file-row--selected': selectedKeys.includes(obj.key) }"
          @contextmenu.prevent.stop="openFileContextMenu($event, obj)"
        >
          <div class="file-name-cell">
            <input type="checkbox" :checked="selectedKeys.includes(obj.key)" @change="toggleSelect(obj.key)" @click.stop />
            <div class="file-icon" @click="obj.is_folder ? navigateTo(obj.key) : openPreview(obj)">
              <svg v-if="obj.is_folder" width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M2 4V13H14V6H8L6 4H2Z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round"/></svg>
              <svg v-else width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M4 2H10L13 5V14H4V2Z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"/><path d="M10 2V5H13" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"/></svg>
            </div>
            <span class="file-name" @click="obj.is_folder ? navigateTo(obj.key) : openPreview(obj)">{{ displayName(obj) }}</span>
          </div>
          <span class="file-size">{{ obj.is_folder ? '—' : formatSize(obj.size) }}</span>
          <span class="file-modified">{{ obj.last_modified ? formatDate(obj.last_modified) : '—' }}</span>
          <div class="file-actions" @click.stop>
            <button v-if="!obj.is_folder" class="action-btn" title="Preview" @click="openPreview(obj)">👁</button>
            <button v-if="!obj.is_folder" class="action-btn" title="Download" @click="downloadObject(obj.key)">↓</button>
            <div class="dropdown-wrapper">
              <button class="action-btn" title="More" @click="toggleDropdown($event, obj.key)">⋮</button>
              <div v-if="openDropdownKey === obj.key" class="dropdown-menu" :style="dropdownStyle">
                <button v-if="!obj.is_folder" class="dropdown-item" @click="openShareLink(obj); closeDropdown()">
                  <span>🔗</span> Share Link
                </button>
                <button v-if="!obj.is_folder" class="dropdown-item" @click="openCopyMove(obj, 'copy'); closeDropdown()">
                  <span>⧉</span> Copy To
                </button>
                <button v-if="!obj.is_folder" class="dropdown-item" @click="openCopyMove(obj, 'move'); closeDropdown()">
                  <span>↷</span> Move To
                </button>
                <button v-if="!obj.is_folder" class="dropdown-item" @click="openRename(obj); closeDropdown()">
                  <span>✎</span> Rename
                </button>
                <div class="dropdown-divider"></div>
                <button class="dropdown-item dropdown-item--danger" @click="confirmDelete(obj); closeDropdown()">
                  <span>✕</span> Delete
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- PERMISSIONS TAB -->
    <div v-show="activeTab === 'permissions'" class="tab-content">
      <div class="section-card">
        <div class="section-card-header">
          <h3>Bucket Policy</h3>
          <button class="btn-primary" @click="savePolicy" :disabled="savingPolicy">{{ savingPolicy ? 'Saving...' : 'Save Policy' }}</button>
        </div>
        <p class="text-secondary text-sm" style="margin-bottom:1rem">JSON policy that controls access to this bucket. Leave empty to have no policy.</p>
        <textarea v-model="policyJson" class="policy-editor glass-input" placeholder='{"Version":"2012-10-17","Statement":[...]}'></textarea>
        <div v-if="policyError" class="error-banner" style="margin-top:0.75rem">{{ policyError }}</div>
      </div>
    </div>

    <!-- PROPERTIES TAB -->
    <div v-show="activeTab === 'properties'" class="tab-content">
      <div class="section-card">
        <div class="section-card-header">
          <h3>Versioning</h3>
          <button class="btn-secondary" @click="toggleVersioning" :disabled="togglingVersioning">
            {{ togglingVersioning ? 'Updating...' : (bucketDetails?.versioning === 'Enabled' ? 'Disable' : 'Enable') }}
          </button>
        </div>
        <p class="text-secondary text-sm">
          Status: <strong :style="{ color: bucketDetails?.versioning === 'Enabled' ? 'var(--success)' : 'var(--text-primary)' }">{{ bucketDetails?.versioning || 'Disabled' }}</strong>
        </p>
        <p class="text-tertiary text-sm" style="margin-top:0.5rem">When enabled, S3 keeps all versions of an object including past overwrites and deletes.</p>
      </div>
      <div class="section-card">
        <h3>Encryption</h3>
        <p class="text-secondary text-sm">Algorithm: <strong>{{ bucketDetails?.encryption || 'Default (SSE-S3)' }}</strong></p>
      </div>
      <div class="section-card">
        <h3>Region</h3>
        <p class="text-secondary text-sm">{{ bucketDetails?.region || '—' }}</p>
      </div>
    </div>

    <!-- TAGS TAB -->
    <div v-show="activeTab === 'tags'" class="tab-content">
      <div class="section-card">
        <div class="section-card-header">
          <h3>Tags</h3>
          <div style="display:flex;gap:0.5rem">
            <button class="btn-secondary" @click="addTagRow">+ Add Tag</button>
            <button class="btn-primary" @click="saveTags" :disabled="savingTags">{{ savingTags ? 'Saving...' : 'Save Tags' }}</button>
          </div>
        </div>
        <div v-if="tags.length === 0" class="text-secondary text-sm" style="padding:1rem 0">No tags configured.</div>
        <div v-else class="tags-table">
          <div class="tags-header"><span>Key</span><span>Value</span><span></span></div>
          <div v-for="(tag, idx) in tags" :key="idx" class="tags-row">
            <input v-model="tag.key" class="glass-input" placeholder="Key" />
            <input v-model="tag.value" class="glass-input" placeholder="Value" />
            <button class="action-btn action-btn--danger" @click="tags.splice(idx, 1)">✕</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Preview Panel (slides in from right) -->
    <div v-if="previewObj" class="preview-overlay" @click.self="previewObj = null">
      <div class="preview-panel">
        <div class="preview-header">
          <h3>{{ previewObj.key.split('/').pop() }}</h3>
          <div style="display:flex;gap:0.375rem;align-items:center">
            <button v-if="previewData?.content && !previewData?.is_binary && isEditableFile(previewObj.key)" class="btn-secondary" style="padding:0.3rem 0.625rem;font-size:0.75rem" @click="openEditor(previewObj!); previewObj = null">
              ✎ Edit
            </button>
            <button class="action-btn" @click="previewObj = null">✕</button>
          </div>
        </div>
        <div class="preview-meta">
          <div><span class="text-tertiary">Type:</span> {{ previewData?.content_type || '—' }}</div>
          <div><span class="text-tertiary">Size:</span> {{ previewData?.size ? formatSize(previewData.size) : '—' }}</div>
        </div>
        <div class="preview-body">
          <div v-if="loadingPreview" class="text-secondary text-sm">Loading preview...</div>
          <img v-else-if="previewData?.is_binary && previewData?.content?.startsWith('data:image')" :src="previewData.content" class="preview-image" />
          <pre v-else-if="previewData?.content && !previewData?.is_binary" class="preview-text">{{ previewData.content }}</pre>
          <div v-else class="text-secondary text-sm">No preview available for this file type.</div>
        </div>
      </div>
    </div>

    <!-- Context Menu -->
    <Teleport to="body">
      <div v-if="contextMenu.visible" class="context-menu" :style="contextMenuStyle" @click.stop>

        <!-- Area context menu (right-click on background) -->
        <template v-if="contextMenu.type === 'area'">
          <button class="context-menu-item" @click="triggerUpload(); closeContextMenu()">
            <span>⬆</span> Upload Files
          </button>
          <button class="context-menu-item" @click="openCreateFolder(); closeContextMenu()">
            <span>📁</span> New Folder
          </button>
          <button class="context-menu-item" @click="openCreateFile(); closeContextMenu()">
            <span>📄</span> New File
          </button>
          <div class="context-menu-divider"></div>
          <button class="context-menu-item" @click="fetchObjects(); closeContextMenu()">
            <span>↻</span> Refresh
          </button>
          <button class="context-menu-item" @click="activeTab = 'properties'; closeContextMenu()">
            <span>⚙</span> Properties
          </button>
        </template>

        <!-- File context menu (right-click on a file/folder) -->
        <template v-if="contextMenu.type === 'file' && contextMenu.target">
          <button v-if="!contextMenu.target.is_folder" class="context-menu-item" @click="openPreview(contextMenu.target!); closeContextMenu()">
            <span>👁</span> Preview
          </button>
          <button v-if="!contextMenu.target.is_folder && isEditableFile(contextMenu.target.key)" class="context-menu-item" @click="openEditor(contextMenu.target!); closeContextMenu()">
            <span>✎</span> Edit
          </button>
          <button v-if="!contextMenu.target.is_folder" class="context-menu-item" @click="downloadObject(contextMenu.target!.key); closeContextMenu()">
            <span>↓</span> Download
          </button>
          <button v-if="!contextMenu.target.is_folder" class="context-menu-item" @click="openShareLink(contextMenu.target!); closeContextMenu()">
            <span>🔗</span> Share Link
          </button>
          <div v-if="!contextMenu.target.is_folder" class="context-menu-divider"></div>
          <button v-if="!contextMenu.target.is_folder" class="context-menu-item" @click="openCopyMove(contextMenu.target!, 'copy'); closeContextMenu()">
            <span>⧉</span> Copy To
          </button>
          <button v-if="!contextMenu.target.is_folder" class="context-menu-item" @click="openCopyMove(contextMenu.target!, 'move'); closeContextMenu()">
            <span>↷</span> Move To
          </button>
          <button class="context-menu-item" @click="openRename(contextMenu.target!); closeContextMenu()">
            <span>✎</span> Rename
          </button>
          <div class="context-menu-divider"></div>
          <button class="context-menu-item context-menu-item--danger" @click="confirmDelete(contextMenu.target!); closeContextMenu()">
            <span>✕</span> Delete
          </button>
        </template>
      </div>
    </Teleport>

    <!-- Hidden file input -->
    <input ref="fileInput" type="file" multiple style="display:none" @change="handleFileSelect" />

    <!-- Modals -->
    <ModalDialog :visible="showFolderModal" title="Create Folder" size="sm" @close="showFolderModal = false">
      <div class="form-group"><label>Folder Name</label><input v-model="newFolderName" type="text" class="glass-input" placeholder="my-folder" @keyup.enter="createFolder" /></div>
      <template #footer>
        <button class="btn-secondary" @click="showFolderModal = false">Cancel</button>
        <button class="btn-primary" @click="createFolder" :disabled="!newFolderName.trim()">Create</button>
      </template>
    </ModalDialog>

    <ModalDialog :visible="showDeleteModal" title="Confirm Delete" size="sm" @close="showDeleteModal = false">
      <p class="text-secondary">Delete <strong style="color:var(--text-primary)">{{ deleteTarget?.key }}</strong>? This cannot be undone.</p>
      <template #footer>
        <button class="btn-secondary" @click="showDeleteModal = false">Cancel</button>
        <button class="btn-danger" @click="executeDelete" :disabled="deleting">{{ deleting ? 'Deleting...' : 'Delete' }}</button>
      </template>
    </ModalDialog>

    <ModalDialog :visible="showRenameModal" title="Rename Object" size="sm" @close="showRenameModal = false">
      <div class="form-group"><label>New Name</label><input v-model="renameNewName" type="text" class="glass-input" @keyup.enter="executeRename" /></div>
      <template #footer>
        <button class="btn-secondary" @click="showRenameModal = false">Cancel</button>
        <button class="btn-primary" @click="executeRename" :disabled="!renameNewName.trim() || renaming">{{ renaming ? 'Renaming...' : 'Rename' }}</button>
      </template>
    </ModalDialog>

    <ModalDialog :visible="showNewFileModal" title="Create File" size="sm" @close="showNewFileModal = false">
      <div class="form-group"><label>File Name</label><input v-model="newFileName" type="text" class="glass-input" placeholder="example.txt" @keyup.enter="createEmptyFile" /></div>
      <template #footer>
        <button class="btn-secondary" @click="showNewFileModal = false">Cancel</button>
        <button class="btn-primary" @click="createEmptyFile" :disabled="!newFileName.trim()">Create</button>
      </template>
    </ModalDialog>

    <!-- Copy/Move Picker Modal -->
    <FileDestinationPicker
      v-model:is-open="showCopyMoveModal"
      :title="copyMoveMode === 'copy' ? 'Copy to...' : 'Move to...'"
      :confirm-text="copyMoveMode === 'copy' ? 'Copy Here' : 'Move Here'"
      :items="pickerObjects"
      :current-path="pickerPath"
      :loading="copyMoveLoading"
      @navigate="handlePickerNavigate"
      @confirm="executeCopyMove"
    />

    <!-- Share Link Modal -->
    <ModalDialog :visible="showShareModal" title="Share Temporary Link" size="sm" @close="showShareModal = false">
      <div class="form-group" style="margin-bottom: 0.75rem">
        <label>File</label>
        <div class="text-sm text-secondary" style="font-family: monospace; word-break: break-all;">{{ shareTarget?.key }}</div>
      </div>
      <div class="form-group" style="margin-bottom: 1rem">
        <label>Expires In</label>
        <select v-model="shareExpiry" class="glass-select">
          <option :value="900">15 minutes</option>
          <option :value="3600">1 hour</option>
          <option :value="21600">6 hours</option>
          <option :value="43200">12 hours</option>
          <option :value="86400">1 day</option>
          <option :value="259200">3 days</option>
          <option :value="604800">7 days</option>
        </select>
      </div>
      <div v-if="!shareUrl" style="display:flex;justify-content:center">
        <button class="btn-primary" @click="generateShareUrl" :disabled="shareLoading">
          {{ shareLoading ? 'Generating...' : 'Generate Link' }}
        </button>
      </div>
      <div v-else class="share-url-box">
        <input :value="shareUrl" readonly class="glass-input share-url-input" @click="($event.target as HTMLInputElement).select()" />
        <button class="btn-secondary" @click="copyShareUrl" title="Copy to clipboard">📋</button>
      </div>
      <div v-if="shareError" class="error-banner" style="margin-top:0.75rem">{{ shareError }}</div>
      <template #footer>
        <button class="btn-secondary" @click="showShareModal = false">Close</button>
      </template>
    </ModalDialog>

    <!-- Upload Progress -->
    <div v-if="uploadProgress.active" class="upload-progress-bar">
      <div class="upload-progress-info">
        <span class="text-sm">Uploading {{ uploadProgress.currentFile }}...</span>
        <span class="text-sm text-tertiary">{{ uploadProgress.completed }}/{{ uploadProgress.total }}</span>
      </div>
      <div class="upload-progress-track">
        <div class="upload-progress-fill" :style="{ width: uploadProgressPercent + '%' }"></div>
      </div>
    </div>

    <!-- File Editor -->
    <S3FileEditor
      v-if="editorOpen"
      :file-key="editorFileKey"
      :initial-content="editorContent"
      :saving="editorSaving"
      @close="closeEditor"
      @save="saveEditorContent"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, inject } from 'vue';
import { useRoute } from 'vue-router';
import { useAwsAccountsStore } from '../store/awsAccounts';
import { invoke } from '@tauri-apps/api/core';
import ModalDialog from '../components/ModalDialog.vue';
import S3FileEditor from '../components/S3FileEditor.vue';
import FileDestinationPicker from '../components/FileDestinationPicker.vue';
import type { FileItem } from '../components/FileSystemBase.vue';

const route = useRoute();
const accountsStore = useAwsAccountsStore();
const toast = inject<any>('toast');

const bucketName = computed(() => route.params.bucket as string);
const activeTab = ref<'objects' | 'permissions' | 'properties' | 'tags'>('objects');
const currentPrefix = ref('');
const fileInput = ref<HTMLInputElement | null>(null);

interface S3Obj { key: string; size: number; last_modified: string | null; is_folder: boolean; }
interface BucketDetails { name: string; region: string | null; versioning: string | null; encryption: string | null; }

const objects = ref<S3Obj[]>([]);
const loadingObjects = ref(false);
const error = ref('');
const bucketDetails = ref<BucketDetails | null>(null);
const selectedKeys = ref<string[]>([]);

// Modals
const showFolderModal = ref(false);
const showDeleteModal = ref(false);
const showRenameModal = ref(false);
const showNewFileModal = ref(false);
const newFolderName = ref('');
const newFileName = ref('');
const deleteTarget = ref<S3Obj | null>(null);
const deleting = ref(false);
const renameTarget = ref<S3Obj | null>(null);
const renameNewName = ref('');
const renaming = ref(false);

// Preview
const previewObj = ref<S3Obj | null>(null);
const previewData = ref<any>(null);
const loadingPreview = ref(false);

// Copy/Move
const showCopyMoveModal = ref(false);
const copyMoveMode = ref<'copy' | 'move'>('copy');
const copyMoveSource = ref<S3Obj | null>(null);
const copyMoveLoading = ref(false);
const pickerPath = ref('');
const pickerObjects = ref<FileItem[]>([]);

// Dropdown
const openDropdownKey = ref<string | null>(null);
const dropdownStyle = ref<Record<string, string>>({});

function toggleDropdown(event: MouseEvent, key: string) {
  if (openDropdownKey.value === key) {
    openDropdownKey.value = null;
    return;
  }
  const btn = event.currentTarget as HTMLElement;
  const rect = btn.getBoundingClientRect();
  // Position below the button, aligned to the right
  const top = rect.bottom + 4;
  const right = window.innerWidth - rect.right;
  // If it would go off-screen at the bottom, show above
  const spaceBelow = window.innerHeight - rect.bottom;
  if (spaceBelow < 220) {
    dropdownStyle.value = { bottom: `${window.innerHeight - rect.top + 4}px`, right: `${right}px` };
  } else {
    dropdownStyle.value = { top: `${top}px`, right: `${right}px` };
  }
  openDropdownKey.value = key;
}
function closeDropdown() {
  openDropdownKey.value = null;
}

// Context Menu
const contextMenu = ref<{
  visible: boolean;
  type: 'area' | 'file';
  target: S3Obj | null;
  x: number;
  y: number;
}>({ visible: false, type: 'area', target: null, x: 0, y: 0 });

const contextMenuStyle = computed(() => {
  const menu = contextMenu.value;
  const style: Record<string, string> = {};
  const menuWidth = 180;
  const menuHeight = menu.type === 'area' ? 160 : 280;

  let x = menu.x;
  let y = menu.y;

  if (x + menuWidth > window.innerWidth) x = window.innerWidth - menuWidth - 8;
  if (y + menuHeight > window.innerHeight) y = window.innerHeight - menuHeight - 8;

  style.left = `${x}px`;
  style.top = `${y}px`;
  return style;
});

function openAreaContextMenu(event: MouseEvent) {
  closeDropdown();
  contextMenu.value = { visible: true, type: 'area', target: null, x: event.clientX, y: event.clientY };
}

function openFileContextMenu(event: MouseEvent, obj: S3Obj) {
  closeDropdown();
  contextMenu.value = { visible: true, type: 'file', target: obj, x: event.clientX, y: event.clientY };
}

function closeContextMenu() {
  contextMenu.value = { ...contextMenu.value, visible: false };
}

// Share Link
const showShareModal = ref(false);
const shareTarget = ref<S3Obj | null>(null);
const shareExpiry = ref(3600);
const shareUrl = ref('');
const shareLoading = ref(false);
const shareError = ref('');

// Upload Progress
const uploadProgress = ref({ active: false, total: 0, completed: 0, currentFile: '' });
const uploadProgressPercent = computed(() => {
  if (uploadProgress.value.total === 0) return 0;
  return Math.round((uploadProgress.value.completed / uploadProgress.value.total) * 100);
});

// File Editor
const editorOpen = ref(false);
const editorFileKey = ref('');
const editorContent = ref('');
const editorSaving = ref(false);

function isEditableFile(key: string): boolean {
  const ext = key.split('.').pop()?.toLowerCase() || '';
  const editable = ['txt','log','md','markdown','json','jsonl','yaml','yml','xml','html','htm',
    'css','scss','less','js','mjs','cjs','jsx','ts','tsx','py','rb','php','sh','bash',
    'sql','toml','ini','cfg','conf','env','vue','svelte','rs','go','java','c','h','cpp',
    'cs','kt','swift','tf','tfvars','graphql','gql','dockerfile','makefile','csv'];
  return editable.includes(ext) || ext === '';
}

async function openEditor(obj: S3Obj) {
  const acc = getAccount(); if (!acc) return;
  try {
    const res: any = await invoke('preview_s3_object', {
      accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey,
      region: acc.region, bucketName: bucketName.value, key: obj.key,
    });
    if (res.success && !res.is_binary) {
      editorFileKey.value = obj.key;
      editorContent.value = res.content || '';
      editorOpen.value = true;
    } else if (res.success && res.is_binary) {
      toast?.error('Cannot edit this file — it appears to be binary.', 'Editor');
    } else {
      toast?.error(res.error_message || 'Failed to load file content.', 'Editor');
    }
  } catch (e: any) { toast?.error(e.message || e.toString(), 'Error'); }
}

function closeEditor() {
  editorOpen.value = false;
}

async function saveEditorContent(content: string) {
  const acc = getAccount(); if (!acc) return;
  editorSaving.value = true;
  try {
    const res: any = await invoke('put_s3_object_content', {
      accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey,
      region: acc.region, bucketName: bucketName.value,
      key: editorFileKey.value, content,
    });
    if (res.success) {
      toast?.success('File saved to S3', 'Saved');
    } else {
      toast?.error(res.error_message || 'Save failed', 'Error');
    }
  } catch (e: any) { toast?.error(e.message || e.toString(), 'Error'); }
  finally { editorSaving.value = false; }
}

// Permissions
const policyJson = ref('');
const policyError = ref('');
const savingPolicy = ref(false);

// Properties
const togglingVersioning = ref(false);

// Tags
const tags = ref<{ key: string; value: string }[]>([]);
const savingTags = ref(false);

const pathSegments = computed(() => currentPrefix.value ? currentPrefix.value.split('/').filter(Boolean) : []);

function getAccount() {
  const id = accountsStore.activeAccountId;
  return id ? accountsStore.accounts.find(a => a.id === id) || null : null;
}
function displayName(obj: S3Obj) {
  let name = obj.key.startsWith(currentPrefix.value) ? obj.key.slice(currentPrefix.value.length) : obj.key;
  if (name.endsWith('/')) name = name.slice(0, -1);
  return name;
}
function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0) + ' ' + units[i];
}
function formatDate(d: string | null): string {
  if (!d) return '—';
  return new Date(d).toLocaleDateString('en-US', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
}
function navigateTo(prefix: string) { currentPrefix.value = prefix; selectedKeys.value = []; fetchObjects(); }
function navigateToSegment(idx: number) { currentPrefix.value = pathSegments.value.slice(0, idx + 1).join('/') + '/'; selectedKeys.value = []; fetchObjects(); }
function toggleSelect(key: string) {
  const i = selectedKeys.value.indexOf(key);
  if (i >= 0) selectedKeys.value.splice(i, 1);
  else selectedKeys.value.push(key);
}

async function fetchObjects() {
  const acc = getAccount(); if (!acc) return;
  loadingObjects.value = true; error.value = '';
  try {
    const res: any = await invoke('list_s3_objects', { accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey, region: acc.region, bucketName: bucketName.value, prefix: currentPrefix.value || null });
    if (res.success) objects.value = res.objects || [];
    else error.value = res.error_message || 'Failed to list objects.';
  } catch (e: any) { error.value = e.message || e.toString(); }
  finally { loadingObjects.value = false; }
}
async function fetchDetails() {
  const acc = getAccount(); if (!acc) return;
  try {
    const res: any = await invoke('get_s3_bucket_details', { accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey, region: acc.region, bucketName: bucketName.value });
    bucketDetails.value = res;
  } catch (_) {}
}
onMounted(() => { 
  fetchObjects(); 
  fetchDetails();
  // Close dropdown and context menu on click outside
  document.addEventListener('click', () => { 
    openDropdownKey.value = null;
    if (contextMenu.value.visible) contextMenu.value = { ...contextMenu.value, visible: false };
  });
});

// Upload
function triggerUpload() { fileInput.value?.click(); }
async function handleFileSelect(event: Event) {
  const input = event.target as HTMLInputElement;
  const files = input.files; if (!files || !files.length) return;
  const acc = getAccount(); if (!acc) return;
  
  const fileList = Array.from(files);
  const MULTIPART_THRESHOLD = 10 * 1024 * 1024; // 10MB
  
  uploadProgress.value = { active: true, total: fileList.length, completed: 0, currentFile: '' };
  let ok = 0, fail = 0;
  
  for (const file of fileList) {
    const key = currentPrefix.value + file.name;
    uploadProgress.value.currentFile = file.name;
    
    try {
      const buf = await file.arrayBuffer();
      const { tempDir } = await import('@tauri-apps/api/path');
      const tmp = await tempDir();
      const tempPath = `${tmp}${file.name}`;
      const { writeFile } = await import('@tauri-apps/plugin-fs');
      await writeFile(tempPath, new Uint8Array(buf));
      
      // Use multipart for large files
      const command = file.size > MULTIPART_THRESHOLD ? 'multipart_upload_s3_object' : 'upload_s3_object';
      const res: any = await invoke(command, {
        accessKeyId: acc.accessKeyId,
        secretAccessKey: acc.secretAccessKey,
        region: acc.region,
        bucketName: bucketName.value,
        key,
        filePath: tempPath,
      });
      
      if (res.success) ok++;
      else fail++;
    } catch (_) { fail++; }
    
    uploadProgress.value.completed++;
  }
  
  uploadProgress.value.active = false;
  if (ok) toast?.success(`${ok} file(s) uploaded`, 'Upload Complete');
  if (fail) toast?.error(`${fail} file(s) failed`, 'Upload Error');
  input.value = ''; fetchObjects();
}

// Download
async function downloadObject(key: string) {
  const acc = getAccount(); if (!acc) return;
  const fileName = key.split('/').pop() || 'download';
  const { save } = await import('@tauri-apps/plugin-dialog');
  const dest = await save({ defaultPath: fileName, title: 'Save file as' });
  if (!dest) return;
  try {
    const res: any = await invoke('download_s3_object', { accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey, region: acc.region, bucketName: bucketName.value, key, destinationPath: dest });
    if (res.success) toast?.success('Downloaded', 'Done');
    else toast?.error(res.error_message || 'Failed', 'Error');
  } catch (e: any) { toast?.error(e.message || e.toString(), 'Error'); }
}

// Delete
function confirmDelete(obj: S3Obj) { deleteTarget.value = obj; showDeleteModal.value = true; }
async function executeDelete() {
  if (!deleteTarget.value) return;
  const acc = getAccount(); if (!acc) return;
  deleting.value = true;
  try {
    const res: any = await invoke('delete_s3_object', { accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey, region: acc.region, bucketName: bucketName.value, key: deleteTarget.value.key });
    if (res.success) { toast?.success('Deleted', 'Done'); showDeleteModal.value = false; fetchObjects(); }
    else toast?.error(res.error_message || 'Failed', 'Error');
  } catch (e: any) { toast?.error(e.message || e.toString(), 'Error'); }
  finally { deleting.value = false; }
}
async function batchDelete() {
  if (!selectedKeys.value.length) return;
  const acc = getAccount(); if (!acc) return;
  try {
    const res: any = await invoke('delete_s3_objects', { accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey, region: acc.region, bucketName: bucketName.value, keys: selectedKeys.value });
    if (res.success) { toast?.success(`${selectedKeys.value.length} objects deleted`, 'Done'); selectedKeys.value = []; fetchObjects(); }
    else toast?.error(res.error_message || 'Failed', 'Error');
  } catch (e: any) { toast?.error(e.message || e.toString(), 'Error'); }
}

// Create Folder
function openCreateFolder() { newFolderName.value = ''; showFolderModal.value = true; }
async function createFolder() {
  if (!newFolderName.value.trim()) return;
  const acc = getAccount(); if (!acc) return;
  try {
    const res: any = await invoke('create_s3_folder', { accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey, region: acc.region, bucketName: bucketName.value, folderKey: currentPrefix.value + newFolderName.value.trim() });
    if (res.success) { toast?.success('Folder created', 'Done'); showFolderModal.value = false; fetchObjects(); }
    else toast?.error(res.error_message || 'Failed', 'Error');
  } catch (e: any) { toast?.error(e.message || e.toString(), 'Error'); }
}

// Create Empty File
function openCreateFile() { newFileName.value = ''; showNewFileModal.value = true; }
async function createEmptyFile() {
  if (!newFileName.value.trim()) return;
  const acc = getAccount(); if (!acc) return;
  const key = currentPrefix.value + newFileName.value.trim();
  try {
    // Upload an empty file using the upload command with a temp empty file
    const { tempDir } = await import('@tauri-apps/api/path');
    const tmp = await tempDir();
    const tempPath = `${tmp}__empty_file_${Date.now()}`;
    const { writeFile } = await import('@tauri-apps/plugin-fs');
    await writeFile(tempPath, new Uint8Array(0));
    const res: any = await invoke('upload_s3_object', { accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey, region: acc.region, bucketName: bucketName.value, key, filePath: tempPath });
    if (res.success) { toast?.success('File created', 'Done'); showNewFileModal.value = false; fetchObjects(); }
    else toast?.error(res.error_message || 'Failed', 'Error');
  } catch (e: any) { toast?.error(e.message || e.toString(), 'Error'); }
}

// Rename (copy + delete)
function openRename(obj: S3Obj) {
  renameTarget.value = obj;
  renameNewName.value = displayName(obj);
  showRenameModal.value = true;
}
async function executeRename() {
  if (!renameTarget.value || !renameNewName.value.trim()) return;
  const acc = getAccount(); if (!acc) return;
  renaming.value = true;
  const destKey = currentPrefix.value + renameNewName.value.trim();
  try {
    const copyRes: any = await invoke('copy_s3_object', { accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey, region: acc.region, sourceBucket: bucketName.value, sourceKey: renameTarget.value.key, destBucket: bucketName.value, destKey });
    if (!copyRes.success) { toast?.error(copyRes.error_message || 'Copy failed', 'Error'); return; }
    await invoke('delete_s3_object', { accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey, region: acc.region, bucketName: bucketName.value, key: renameTarget.value.key });
    toast?.success('Renamed', 'Done');
    showRenameModal.value = false;
    fetchObjects();
  } catch (e: any) { toast?.error(e.message || e.toString(), 'Error'); }
  finally { renaming.value = false; }
}

// Preview
async function openPreview(obj: S3Obj) {
  if (obj.is_folder) return;
  previewObj.value = obj;
  previewData.value = null;
  loadingPreview.value = true;
  const acc = getAccount(); if (!acc) return;
  try {
    const res: any = await invoke('preview_s3_object', { accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey, region: acc.region, bucketName: bucketName.value, key: obj.key });
    previewData.value = res;
  } catch (e: any) { previewData.value = { error_message: e.message || e.toString() }; }
  finally { loadingPreview.value = false; }
}

// Permissions
async function loadPolicy() {
  const acc = getAccount(); if (!acc) return;
  policyError.value = '';
  try {
    const res: any = await invoke('get_s3_bucket_policy', { accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey, region: acc.region, bucketName: bucketName.value });
    if (res.success) policyJson.value = res.policy ? JSON.stringify(JSON.parse(res.policy), null, 2) : '';
    else policyError.value = res.error_message || 'Failed to load policy';
  } catch (e: any) { policyError.value = e.message || e.toString(); }
}
async function savePolicy() {
  const acc = getAccount(); if (!acc) return;
  savingPolicy.value = true; policyError.value = '';
  try {
    if (!policyJson.value.trim()) { toast?.info('Policy is empty — no changes saved'); savingPolicy.value = false; return; }
    JSON.parse(policyJson.value); // validate JSON
    const res: any = await invoke('put_s3_bucket_policy', { accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey, region: acc.region, bucketName: bucketName.value, policy: policyJson.value });
    if (res.success) toast?.success('Policy saved', 'Done');
    else policyError.value = res.error_message || 'Failed to save';
  } catch (e: any) { policyError.value = 'Invalid JSON: ' + (e.message || e.toString()); }
  finally { savingPolicy.value = false; }
}

// Properties
async function toggleVersioning() {
  const acc = getAccount(); if (!acc) return;
  togglingVersioning.value = true;
  const enable = bucketDetails.value?.versioning !== 'Enabled';
  try {
    const res: any = await invoke('put_s3_bucket_versioning', { accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey, region: acc.region, bucketName: bucketName.value, enabled: enable });
    if (res.success) { toast?.success(`Versioning ${enable ? 'enabled' : 'suspended'}`, 'Done'); fetchDetails(); }
    else toast?.error(res.error_message || 'Failed', 'Error');
  } catch (e: any) { toast?.error(e.message || e.toString(), 'Error'); }
  finally { togglingVersioning.value = false; }
}

// Tags
async function loadTags() {
  const acc = getAccount(); if (!acc) return;
  try {
    const res: any = await invoke('get_s3_bucket_tags', { accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey, region: acc.region, bucketName: bucketName.value });
    if (res.success) tags.value = res.tags || [];
  } catch (_) {}
}
function addTagRow() { tags.value.push({ key: '', value: '' }); }
async function saveTags() {
  const acc = getAccount(); if (!acc) return;
  savingTags.value = true;
  const validTags = tags.value.filter(t => t.key.trim());
  try {
    const res: any = await invoke('put_s3_bucket_tags', { accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey, region: acc.region, bucketName: bucketName.value, tags: validTags });
    if (res.success) toast?.success('Tags saved', 'Done');
    else toast?.error(res.error_message || 'Failed', 'Error');
  } catch (e: any) { toast?.error(e.message || e.toString(), 'Error'); }
  finally { savingTags.value = false; }
}

// Copy/Move
function openCopyMove(obj: S3Obj, mode: 'copy' | 'move') {
  copyMoveSource.value = obj;
  copyMoveMode.value = mode;
  pickerPath.value = currentPrefix.value;
  loadPickerFolders(pickerPath.value);
  copyMoveLoading.value = false;
  showCopyMoveModal.value = true;
}

async function loadPickerFolders(prefix: string) {
  const acc = getAccount(); if (!acc) return;
  copyMoveLoading.value = true;
  try {
    const res: any = await invoke('list_s3_objects', { accessKeyId: acc.accessKeyId, secretAccessKey: acc.secretAccessKey, region: acc.region, bucketName: bucketName.value, prefix: prefix || null });
    if (res.success) {
      pickerObjects.value = (res.objects || []).map((o: any) => {
        let name = o.key.startsWith(prefix) ? o.key.slice(prefix.length) : o.key;
        if (name.endsWith('/')) name = name.slice(0, -1);
        return {
          name: name || 'root',
          path: o.key,
          isDir: o.is_folder,
          size: o.size,
          lastModified: o.last_modified
        };
      });
    }
  } catch (e: any) { }
  finally { copyMoveLoading.value = false; }
}

function handlePickerNavigate(path: string) {
  let newPrefix = path === '/' ? '' : path;
  if (newPrefix && !newPrefix.endsWith('/')) newPrefix += '/';
  pickerPath.value = newPrefix;
  loadPickerFolders(newPrefix);
}

async function executeCopyMove(destinationFolder: string) {
  if (!copyMoveSource.value) return;
  const acc = getAccount(); if (!acc) return;
  
  const destPrefix = destinationFolder === '/' ? '' : destinationFolder;
  const sourceName = copyMoveSource.value.key.split('/').pop() || '';
  const destKey = destPrefix + sourceName;
  
  // Do not allow copying/moving to the exact same path
  if (destKey === copyMoveSource.value.key) {
    toast?.info(`File is already at ${destKey}`);
    showCopyMoveModal.value = false;
    return;
  }
  
  copyMoveLoading.value = true;
  
  try {
    if (copyMoveMode.value === 'copy') {
      const res: any = await invoke('copy_s3_object', {
        accessKeyId: acc.accessKeyId,
        secretAccessKey: acc.secretAccessKey,
        region: acc.region,
        sourceBucket: bucketName.value,
        sourceKey: copyMoveSource.value.key,
        destBucket: bucketName.value,
        destKey,
      });
      if (res.success) {
        toast?.success('Object copied', 'Done');
        showCopyMoveModal.value = false;
        fetchObjects();
      } else {
        toast?.error(res.error_message || 'Copy failed', 'Error');
      }
    } else {
      const res: any = await invoke('move_s3_object', {
        accessKeyId: acc.accessKeyId,
        secretAccessKey: acc.secretAccessKey,
        region: acc.region,
        sourceBucket: bucketName.value,
        sourceKey: copyMoveSource.value.key,
        destBucket: bucketName.value,
        destKey,
      });
      if (res.success) {
        toast?.success('Object moved', 'Done');
        showCopyMoveModal.value = false;
        fetchObjects();
      } else {
        toast?.error(res.error_message || 'Move failed', 'Error');
      }
    }
  } catch (e: any) { toast?.error(e.message || e.toString(), 'Error'); }
  finally { copyMoveLoading.value = false; }
}

// Share Link (Presigned URL)
function openShareLink(obj: S3Obj) {
  shareTarget.value = obj;
  shareExpiry.value = 3600;
  shareUrl.value = '';
  shareError.value = '';
  shareLoading.value = false;
  showShareModal.value = true;
}

async function generateShareUrl() {
  if (!shareTarget.value) return;
  const acc = getAccount(); if (!acc) return;
  shareLoading.value = true;
  shareError.value = '';
  
  try {
    const res: any = await invoke('generate_presigned_url', {
      accessKeyId: acc.accessKeyId,
      secretAccessKey: acc.secretAccessKey,
      region: acc.region,
      bucketName: bucketName.value,
      key: shareTarget.value.key,
      expiresInSecs: shareExpiry.value,
    });
    
    if (res.success) {
      shareUrl.value = res.url;
    } else {
      shareError.value = res.error_message || 'Failed to generate URL';
    }
  } catch (e: any) { shareError.value = e.message || e.toString(); }
  finally { shareLoading.value = false; }
}

async function copyShareUrl() {
  if (!shareUrl.value) return;
  try {
    await navigator.clipboard.writeText(shareUrl.value);
    toast?.success('Link copied to clipboard', 'Copied');
  } catch (_) {
    // Fallback: select the input
    toast?.info('Select and copy the URL manually');
  }
}
</script>

<style scoped>
.bucket-view { height: 100%; display: flex; flex-direction: column; }
.page-header { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: 1rem; }
.header-left { display: flex; align-items: center; gap: 0.75rem; }
.header-left h1 { margin-bottom: 0.125rem; }
.btn-back { width: 32px; height: 32px; display: flex; align-items: center; justify-content: center; background: var(--bg-elevated); border: 1px solid var(--border-default); border-radius: var(--radius-sm); color: var(--text-secondary); cursor: pointer; transition: all var(--transition-fast); }
.btn-back:hover { color: var(--text-primary); border-color: var(--border-hover); }
.page-actions { display: flex; gap: 0.5rem; }

/* Tabs */
.tabs-bar { display: flex; border-bottom: 1px solid var(--border-default); margin-bottom: 1rem; gap: 0; }
.tab { padding: 0.625rem 1rem; background: none; border: none; color: var(--text-tertiary); font-size: 0.8125rem; font-weight: 450; cursor: pointer; border-bottom: 2px solid transparent; margin-bottom: -1px; transition: all var(--transition-fast); }
.tab:hover { color: var(--text-secondary); }
.tab--active { color: var(--text-primary); border-bottom-color: var(--text-primary); }
.tab-content { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 1rem; }

/* Path Bar */
.path-bar { display: flex; align-items: center; gap: 0.125rem; padding: 0.5rem 0.75rem; background: var(--bg-primary); border: 1px solid var(--border-default); border-radius: var(--radius-sm); overflow-x: auto; }
.path-segment { background: none; border: none; color: var(--text-secondary); font-size: 0.8125rem; cursor: pointer; padding: 0.2rem 0.375rem; border-radius: 4px; transition: all var(--transition-fast); white-space: nowrap; }
.path-segment:hover { color: var(--text-primary); background: var(--bg-hover); }
.path-segment:last-child { color: var(--text-primary); font-weight: 500; }
.path-sep { color: var(--text-tertiary); font-size: 0.75rem; }

/* Batch Bar */
.batch-bar { display: flex; align-items: center; gap: 0.75rem; padding: 0.5rem 0.75rem; background: var(--bg-secondary); border: 1px solid var(--border-default); border-radius: var(--radius-sm); }

/* File Table */
.file-table { border: 1px solid var(--border-default); border-radius: var(--radius-lg); overflow: hidden; }
.file-table-header { display: grid; grid-template-columns: 1fr 90px 130px 100px; padding: 0.5rem 1rem; background: var(--bg-secondary); border-bottom: 1px solid var(--border-default); font-size: 0.6875rem; font-weight: 500; color: var(--text-tertiary); text-transform: uppercase; letter-spacing: 0.04em; }
.file-row { display: grid; grid-template-columns: 1fr 90px 130px 100px; align-items: center; padding: 0.5rem 1rem; border-bottom: 1px solid var(--border-default); background: var(--bg-primary); transition: background var(--transition-fast); }
.file-row:last-child { border-bottom: none; }
.file-row:hover { background: var(--bg-secondary); }
.file-row--folder { cursor: pointer; }
.file-row--selected { background: rgba(0, 112, 243, 0.05); }
.file-name-cell { display: flex; align-items: center; gap: 0.5rem; min-width: 0; }
.file-name-cell input[type="checkbox"] { width: 14px; height: 14px; accent-color: var(--accent); cursor: pointer; }
.file-icon { color: var(--text-tertiary); cursor: pointer; flex-shrink: 0; }
.file-name { font-size: 0.8125rem; font-weight: 450; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; cursor: pointer; }
.file-name:hover { color: var(--accent); }
.file-size, .file-modified { font-size: 0.75rem; color: var(--text-tertiary); }
.file-actions { display: flex; gap: 0.25rem; justify-content: flex-end; }
.action-btn { width: 32px; height: 32px; display: flex; align-items: center; justify-content: center; background: none; border: 1px solid transparent; border-radius: var(--radius-sm); color: var(--text-tertiary); cursor: pointer; font-size: 1rem; transition: all var(--transition-fast); }
.action-btn:hover { color: var(--text-primary); background: var(--bg-hover); border-color: var(--border-default); }
.action-btn--danger:hover { color: var(--error); background: var(--error-subtle); border-color: rgba(238,0,0,0.2); }
.empty-folder { display: flex; flex-direction: column; align-items: center; padding: 3rem 2rem; gap: 0.5rem; border: 1px dashed var(--border-default); border-radius: var(--radius-lg); }
.error-banner { display: flex; align-items: center; gap: 0.5rem; padding: 0.625rem 0.875rem; background: var(--error-subtle); border: 1px solid rgba(238,0,0,0.15); border-radius: var(--radius-sm); color: #ff6166; font-size: 0.8125rem; }

/* Section Cards */
.section-card { background: var(--bg-primary); border: 1px solid var(--border-default); border-radius: var(--radius-lg); padding: 1.25rem; }
.section-card h3 { margin: 0 0 0.25rem; }
.section-card-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 0.75rem; }
.section-card-header h3 { margin: 0; }

/* Policy Editor */
.policy-editor { width: 100%; min-height: 250px; font-family: 'JetBrains Mono', monospace; font-size: 0.75rem; line-height: 1.6; resize: vertical; }

/* Tags Table */
.tags-table { display: flex; flex-direction: column; gap: 0.5rem; }
.tags-header { display: grid; grid-template-columns: 1fr 1fr 32px; gap: 0.5rem; font-size: 0.6875rem; color: var(--text-tertiary); text-transform: uppercase; letter-spacing: 0.04em; padding: 0.25rem 0; }
.tags-row { display: grid; grid-template-columns: 1fr 1fr 32px; gap: 0.5rem; align-items: center; }

/* Preview Panel */
.preview-overlay { position: fixed; inset: 0; z-index: 4000; background: rgba(0,0,0,0.5); display: flex; justify-content: flex-end; }
.preview-panel { width: 480px; max-width: 90vw; height: 100%; background: var(--bg-secondary); border-left: 1px solid var(--border-default); display: flex; flex-direction: column; animation: slideIn 0.2s ease; }
@keyframes slideIn { from { transform: translateX(100%); } to { transform: translateX(0); } }
.preview-header { display: flex; align-items: center; justify-content: space-between; padding: 1rem 1.25rem; border-bottom: 1px solid var(--border-default); }
.preview-header h3 { margin: 0; font-size: 0.875rem; word-break: break-all; }
.preview-meta { padding: 0.75rem 1.25rem; display: flex; gap: 1.5rem; font-size: 0.75rem; border-bottom: 1px solid var(--border-default); }
.preview-body { flex: 1; overflow: auto; padding: 1rem 1.25rem; }
.preview-image { max-width: 100%; border-radius: var(--radius-sm); }
.preview-text { font-family: 'JetBrains Mono', monospace; font-size: 0.75rem; line-height: 1.6; white-space: pre-wrap; word-break: break-all; color: var(--text-secondary); margin: 0; }

/* Form */
.form-group { display: flex; flex-direction: column; gap: 0.375rem; }
.form-group label { font-size: 0.75rem; font-weight: 500; color: var(--text-secondary); }

/* Share URL */
.share-url-box { display: flex; gap: 0.5rem; align-items: center; margin-top: 0.5rem; }
.share-url-input { flex: 1; font-family: 'JetBrains Mono', monospace; font-size: 0.6875rem; cursor: text; }

/* Dropdown */
.dropdown-wrapper { position: relative; }
.dropdown-menu { position: fixed; z-index: 5000; min-width: 160px; background: var(--bg-primary); border: 1px solid var(--border-default); border-radius: var(--radius-md); box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4); padding: 0.25rem; animation: dropIn 0.12s ease; }
@keyframes dropIn { from { opacity: 0; transform: translateY(-4px); } to { opacity: 1; transform: translateY(0); } }
.dropdown-item { display: flex; align-items: center; gap: 0.625rem; width: 100%; padding: 0.5rem 0.75rem; background: none; border: none; border-radius: var(--radius-sm); color: var(--text-secondary); font-size: 0.8125rem; cursor: pointer; transition: all var(--transition-fast); text-align: left; }
.dropdown-item:hover { background: var(--bg-hover); color: var(--text-primary); }
.dropdown-item--danger:hover { background: var(--error-subtle); color: #ff6166; }
.dropdown-item span { font-size: 0.9375rem; width: 18px; text-align: center; }
.dropdown-divider { height: 1px; background: var(--border-default); margin: 0.25rem 0; }

/* Context Menu */
.context-menu {
  position: fixed;
  z-index: 9999;
  min-width: 180px;
  background: var(--bg-primary);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5), 0 2px 8px rgba(0, 0, 0, 0.3);
  padding: 0.3rem;
  animation: ctxIn 0.1s ease;
}

@keyframes ctxIn {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}

.context-menu-item {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  width: 100%;
  padding: 0.5rem 0.75rem;
  background: none;
  border: none;
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  font-size: 0.8125rem;
  cursor: pointer;
  transition: all var(--transition-fast);
  text-align: left;
}

.context-menu-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.context-menu-item--danger:hover {
  background: var(--error-subtle);
  color: #ff6166;
}

.context-menu-item span {
  font-size: 0.9375rem;
  width: 18px;
  text-align: center;
}

.context-menu-divider {
  height: 1px;
  background: var(--border-default);
  margin: 0.3rem 0;
}

/* Upload Progress */
.upload-progress-bar { position: fixed; bottom: 1.5rem; left: 50%; transform: translateX(-50%); width: 360px; background: var(--bg-primary); border: 1px solid var(--border-default); border-radius: var(--radius-lg); padding: 0.875rem 1.25rem; box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3); z-index: 5000; animation: slideUp 0.2s ease; }
@keyframes slideUp { from { opacity: 0; transform: translateX(-50%) translateY(12px); } to { opacity: 1; transform: translateX(-50%) translateY(0); } }
.upload-progress-info { display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.5rem; }
.upload-progress-track { height: 4px; background: var(--bg-elevated); border-radius: 2px; overflow: hidden; }
.upload-progress-fill { height: 100%; background: var(--accent); border-radius: 2px; transition: width 0.3s ease; }
</style>
