<template>
  <div class="file-system-base">
    <!-- Toolbar -->
    <div class="path-bar" style="margin-bottom: 0.75rem">
      <button class="path-segment" @click="$emit('navigate', '/')">root</button>
      <template v-for="(part, index) in pathParts" :key="index">
        <span class="path-sep">/</span>
        <button class="path-segment" @click="navigateToPart(index)">{{ part }}</button>
      </template>
    </div>

    <!-- File List -->
    <div class="file-table" :class="{ 'is-loading': loading }" style="min-height: 200px">
      <div v-if="loading" class="loading-overlay">
        <div class="spinner"></div>
      </div>
      
      <div class="file-table-header" :style="pickerMode ? 'grid-template-columns: 1fr;' : ''">
        <span>Name</span>
        <span v-if="!pickerMode">Size</span>
        <span v-if="!pickerMode">Modified</span>
        <span v-if="showActions && !pickerMode">Actions</span>
      </div>

      <div v-if="items.length === 0 && !loading" class="empty-folder">
        <p class="text-secondary text-sm">This folder is empty</p>
      </div>
      
      <template v-else>
        <div 
          v-for="item in filteredItems" 
          :key="item.path"
          class="file-row"
          :class="{ 
            'file-row--folder': item.isDir, 
            'file-row--selected': selectedPath === item.path 
          }"
          :style="pickerMode ? 'grid-template-columns: 1fr;' : ''"
          @click="selectItem(item)"
          @dblclick="onDoubleClick(item)"
        >
          <div class="file-name-cell">
            <div class="file-icon" @click.stop="onDoubleClick(item)">
              <svg v-if="item.isDir" width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M2 4V13H14V6H8L6 4H2Z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round"/></svg>
              <svg v-else width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M4 2H10L13 5V14H4V2Z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"/><path d="M10 2V5H13" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"/></svg>
            </div>
            <span class="file-name" @click.stop="onDoubleClick(item)">{{ item.name }}</span>
          </div>
          
          <span class="file-size" v-if="!pickerMode">
            {{ item.isDir ? '—' : formatSize(item.size) }}
          </span>
          
          <span class="file-modified" v-if="!pickerMode">
            {{ formatDate(item.lastModified) }}
          </span>
          
          <div class="file-actions" v-if="showActions && !pickerMode" @click.stop>
            <button class="action-btn" title="Copy" @click="$emit('action', { type: 'copy', path: item.path })">⧉</button>
            <button class="action-btn" title="Move" @click="$emit('action', { type: 'move', path: item.path })">↷</button>
            <slot name="item-actions" :item="item"></slot>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';

export interface FileItem {
  name: string;
  path: string;
  isDir: boolean;
  size?: number;
  lastModified?: string;
  [key: string]: any;
}

const props = withDefaults(defineProps<{
  items: FileItem[];
  currentPath: string;
  loading?: boolean;
  showActions?: boolean;
  pickerMode?: boolean; // If true, hides files and only shows folders for selection
}>(), {
  loading: false,
  showActions: true,
  pickerMode: false
});

const emit = defineEmits<{
  (e: 'navigate', path: string): void;
  (e: 'select', path: string, item: FileItem): void;
  (e: 'action', payload: { type: 'copy' | 'move' | string, path: string }): void;
}>();

const selectedPath = ref<string | null>(null);

const pathParts = computed(() => {
  if (!props.currentPath || props.currentPath === '/') return [];
  return props.currentPath.split('/').filter(Boolean);
});

const filteredItems = computed(() => {
  let list = props.items;
  if (props.pickerMode) {
    list = list.filter(i => i.isDir);
  }
  return list.sort((a, b) => {
    if (a.isDir && !b.isDir) return -1;
    if (!a.isDir && b.isDir) return 1;
    return a.name.localeCompare(b.name);
  });
});

const selectItem = (item: FileItem) => {
  if (props.pickerMode && !item.isDir) return;
  
  selectedPath.value = item.path;
  emit('select', item.path, item);
};

const onDoubleClick = (item: FileItem) => {
  if (item.isDir) {
    selectedPath.value = null; // Clear selection on navigate
    emit('navigate', item.path);
  }
};

const navigateToPart = (index: number) => {
  const parts = pathParts.value.slice(0, index + 1);
  const path = '/' + parts.join('/');
  emit('navigate', path);
};

// Utils
const formatSize = (bytes?: number) => {
  if (bytes === undefined || bytes === null) return '—';
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
};

const formatDate = (dateString?: string) => {
  if (!dateString) return '—';
  const date = new Date(dateString);
  return date.toLocaleDateString('en-US', { 
    month: 'short', 
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  });
};
</script>

<style scoped>
.file-system-base {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.path-bar { display: flex; align-items: center; gap: 0.125rem; padding: 0.5rem 0.75rem; background: var(--bg-primary); border: 1px solid var(--border-default); border-radius: var(--radius-sm); overflow-x: auto; }
.path-segment { background: none; border: none; color: var(--text-secondary); font-size: 0.8125rem; cursor: pointer; padding: 0.2rem 0.375rem; border-radius: 4px; transition: all var(--transition-fast); white-space: nowrap; }
.path-segment:hover { color: var(--text-primary); background: var(--bg-hover); }
.path-segment:last-child { color: var(--text-primary); font-weight: 500; }
.path-sep { color: var(--text-tertiary); font-size: 0.75rem; }

.file-table { border: 1px solid var(--border-default); border-radius: var(--radius-lg); overflow: hidden; position: relative; }
.file-table-header { display: grid; grid-template-columns: 1fr 90px 130px 100px; padding: 0.5rem 1rem; background: var(--bg-secondary); border-bottom: 1px solid var(--border-default); font-size: 0.6875rem; font-weight: 500; color: var(--text-tertiary); text-transform: uppercase; letter-spacing: 0.04em; }
.file-row { display: grid; grid-template-columns: 1fr 90px 130px 100px; align-items: center; padding: 0.5rem 1rem; border-bottom: 1px solid var(--border-default); background: var(--bg-primary); transition: background var(--transition-fast); }
.file-row:last-child { border-bottom: none; }
.file-row:hover { background: var(--bg-secondary); }
.file-row--folder { cursor: pointer; }
.file-row--selected { background: rgba(0, 112, 243, 0.05); }
.file-name-cell { display: flex; align-items: center; gap: 0.5rem; min-width: 0; }
.file-icon { color: var(--text-tertiary); cursor: pointer; flex-shrink: 0; }
.file-name { font-size: 0.8125rem; font-weight: 450; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; cursor: pointer; }
.file-name:hover { color: var(--accent); }
.file-size, .file-modified { font-size: 0.75rem; color: var(--text-tertiary); }
.file-actions { display: flex; gap: 0.25rem; justify-content: flex-end; }
.action-btn { width: 32px; height: 32px; display: flex; align-items: center; justify-content: center; background: none; border: 1px solid transparent; border-radius: var(--radius-sm); color: var(--text-tertiary); cursor: pointer; font-size: 1rem; transition: all var(--transition-fast); }
.action-btn:hover { color: var(--text-primary); background: var(--bg-hover); border-color: var(--border-default); }
.empty-folder { display: flex; flex-direction: column; align-items: center; padding: 3rem 2rem; gap: 0.5rem; }

.loading-overlay {
  position: absolute;
  inset: 0;
  background: rgba(var(--bg-primary-rgb), 0.5);
  backdrop-filter: blur(2px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10;
}

.spinner {
  width: 24px;
  height: 24px;
  border: 2px solid var(--border-default);
  border-top-color: var(--primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
