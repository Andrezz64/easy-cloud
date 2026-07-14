<template>
  <div class="editor-overlay" @keydown="handleKeydown">
    <!-- Header -->
    <div class="editor-header">
      <div class="editor-header-left">
        <button class="editor-back-btn" @click="$emit('close')" title="Close editor">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
            <path d="M10 4L6 8L10 12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
        </button>
        <div class="editor-file-info">
          <span class="editor-filename">{{ fileName }}</span>
          <span class="editor-path text-tertiary">{{ fileKey }}</span>
        </div>
        <span v-if="isDirty" class="editor-dirty-badge">●</span>
        <span class="badge badge--neutral">{{ languageLabel }}</span>
      </div>
      <div class="editor-header-right">
        <span v-if="saving" class="text-sm text-tertiary">Saving...</span>
        <span v-else-if="lastSaved" class="text-sm text-tertiary">Saved {{ lastSavedLabel }}</span>
        <button class="btn-secondary" @click="saveFile" :disabled="saving || !isDirty">
          <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
            <path d="M3 3V13H13V6L10 3H3Z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round"/>
            <path d="M6 3V6H10V3" stroke="currentColor" stroke-width="1.3"/>
            <path d="M5 9H11" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
            <path d="M5 11H9" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/>
          </svg>
          Save
        </button>
        <button class="btn-secondary" @click="$emit('close')">Close</button>
      </div>
    </div>

    <!-- Editor Body -->
    <div class="editor-body">
      <codemirror
        v-model="content"
        :style="{ height: '100%' }"
        :autofocus="true"
        :indent-with-tab="true"
        :tab-size="2"
        :extensions="extensions"
        @change="markDirty"
      />
    </div>

    <!-- Status Bar -->
    <div class="editor-statusbar">
      <span class="text-sm text-tertiary">{{ lineCount }} lines</span>
      <span class="text-sm text-tertiary">{{ formatSize(content.length) }}</span>
      <span class="text-sm text-tertiary">{{ languageLabel }}</span>
      <span class="text-sm text-tertiary" style="margin-left:auto">Ctrl+Shift+S to save</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { Codemirror } from 'vue-codemirror';
import { oneDark } from '@codemirror/theme-one-dark';
import { yaml } from '@codemirror/lang-yaml';
import { json } from '@codemirror/lang-json';
import { html } from '@codemirror/lang-html';
import { css } from '@codemirror/lang-css';
import { javascript } from '@codemirror/lang-javascript';
import { python } from '@codemirror/lang-python';
import { markdown } from '@codemirror/lang-markdown';
import { xml } from '@codemirror/lang-xml';

const props = defineProps<{
  fileKey: string;
  initialContent: string;
  saving: boolean;
}>();

const emit = defineEmits<{
  close: [];
  save: [content: string];
}>();

const content = ref(props.initialContent);
const isDirty = ref(false);
const lastSaved = ref<Date | null>(null);

const fileName = computed(() => props.fileKey.split('/').pop() || props.fileKey);

const lineCount = computed(() => content.value.split('\n').length);

const languageLabel = computed(() => {
  const ext = getExtension();
  const map: Record<string, string> = {
    yaml: 'YAML', yml: 'YAML', json: 'JSON', jsonl: 'JSON',
    html: 'HTML', htm: 'HTML', vue: 'HTML', svelte: 'HTML',
    css: 'CSS', scss: 'CSS', less: 'CSS',
    js: 'JavaScript', mjs: 'JavaScript', cjs: 'JavaScript', jsx: 'JSX',
    ts: 'TypeScript', tsx: 'TSX',
    py: 'Python',
    md: 'Markdown', markdown: 'Markdown',
    xml: 'XML', svg: 'XML', xsl: 'XML',
    txt: 'Plain Text', log: 'Plain Text', env: 'Plain Text',
    sh: 'Shell', bash: 'Shell', zsh: 'Shell',
    sql: 'SQL', toml: 'TOML', ini: 'INI', cfg: 'Config',
    rs: 'Rust', go: 'Go', java: 'Java', rb: 'Ruby', php: 'PHP',
    c: 'C', h: 'C', cpp: 'C++', cs: 'C#', kt: 'Kotlin', swift: 'Swift',
    tf: 'Terraform', dockerfile: 'Dockerfile',
  };
  return map[ext] || 'Plain Text';
});

const extensions = computed(() => {
  const ext = getExtension();
  const lang = getLangExtension(ext);
  return lang ? [oneDark, lang] : [oneDark];
});

const lastSavedLabel = computed(() => {
  if (!lastSaved.value) return '';
  const secs = Math.floor((Date.now() - lastSaved.value.getTime()) / 1000);
  if (secs < 5) return 'just now';
  if (secs < 60) return `${secs}s ago`;
  return `${Math.floor(secs / 60)}m ago`;
});

function getExtension(): string {
  const name = fileName.value.toLowerCase();
  const parts = name.split('.');
  if (parts.length < 2) return 'txt';
  return parts.pop() || 'txt';
}

function getLangExtension(ext: string) {
  switch (ext) {
    case 'yaml': case 'yml': return yaml();
    case 'json': case 'jsonl': return json();
    case 'html': case 'htm': case 'vue': case 'svelte': return html();
    case 'css': case 'scss': case 'less': return css();
    case 'js': case 'mjs': case 'cjs': case 'jsx': return javascript();
    case 'ts': case 'tsx': return javascript({ typescript: true });
    case 'py': return python();
    case 'md': case 'markdown': return markdown();
    case 'xml': case 'svg': case 'xsl': return xml();
    default: return null;
  }
}

function markDirty() {
  isDirty.value = true;
}

function saveFile() {
  emit('save', content.value);
  isDirty.value = false;
  lastSaved.value = new Date();
}

function handleKeydown(e: KeyboardEvent) {
  // Ctrl+Shift+S or Ctrl+S to save
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') {
    e.preventDefault();
    if (isDirty.value) saveFile();
  }
}

function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0) + ' ' + units[i];
}

// Also listen globally for Ctrl+S
function globalKeyHandler(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') {
    e.preventDefault();
    if (isDirty.value) saveFile();
  }
}

onMounted(() => { document.addEventListener('keydown', globalKeyHandler); });
onUnmounted(() => { document.removeEventListener('keydown', globalKeyHandler); });
</script>

<style scoped>
.editor-overlay {
  position: fixed;
  inset: 0;
  z-index: 6000;
  background: var(--bg-root);
  display: flex;
  flex-direction: column;
  animation: editorFadeIn 0.15s ease;
}

@keyframes editorFadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.editor-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.625rem 1rem;
  border-bottom: 1px solid var(--border-default);
  background: var(--bg-primary);
  gap: 1rem;
}

.editor-header-left {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  min-width: 0;
}

.editor-back-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  cursor: pointer;
  flex-shrink: 0;
  transition: all 150ms;
}

.editor-back-btn:hover {
  color: var(--text-primary);
  border-color: var(--border-hover);
}

.editor-file-info {
  display: flex;
  align-items: baseline;
  gap: 0.5rem;
  min-width: 0;
}

.editor-filename {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
}

.editor-path {
  font-size: 0.6875rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.editor-dirty-badge {
  color: #f5a623;
  font-size: 1rem;
  line-height: 1;
}

.editor-header-right {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  flex-shrink: 0;
}

.editor-body {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

:deep(.cm-editor) {
  height: 100%;
  background: var(--bg-root) !important;
}

:deep(.cm-scroller) {
  font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace;
  font-size: 0.8125rem;
  line-height: 1.6;
}

:deep(.cm-gutters) {
  background: var(--bg-primary) !important;
  border-right: 1px solid var(--border-default) !important;
}

:deep(.cm-activeLineGutter) {
  background: var(--bg-hover) !important;
}

:deep(.cm-activeLine) {
  background: rgba(255, 255, 255, 0.02) !important;
}

.editor-statusbar {
  display: flex;
  align-items: center;
  gap: 1.5rem;
  padding: 0.375rem 1rem;
  border-top: 1px solid var(--border-default);
  background: var(--bg-primary);
}
</style>
