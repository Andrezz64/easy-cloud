<template>
  <ModalDialog 
    :visible="isOpen" 
    :title="title" 
    size="md" 
    @close="close"
  >
    <!-- Picker Body -->
    <div style="height: 400px; max-height: 60vh; display: flex; flex-direction: column; overflow: hidden; margin: -1.5rem -1.5rem 0 -1.5rem;">
      <FileSystemBase
        :items="items"
        :current-path="currentPath"
        :loading="loading"
        :picker-mode="true"
        style="border: none; border-radius: 0;"
        @navigate="onNavigate"
        @select="onSelect"
      />
    </div>
    
    <!-- Footer slots -->
    <template #footer>
      <div style="flex: 1; text-align: left; font-size: 0.8125rem; color: var(--text-secondary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; padding-right: 1rem;">
        <span v-if="selectedPath">Selected: <strong style="color: var(--text-primary)">{{ selectedPath }}</strong></span>
        <span v-else>Selected: <strong style="color: var(--text-primary)">{{ currentPath === '' ? 'root' : currentPath }}</strong></span>
      </div>
      <button class="btn-secondary" @click="close">Cancel</button>
      <button 
        class="btn-primary" 
        :disabled="loading"
        @click="confirmSelection"
      >
        {{ confirmText }}
      </button>
    </template>
  </ModalDialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import ModalDialog from './ModalDialog.vue';
import FileSystemBase, { type FileItem } from './FileSystemBase.vue';

const props = withDefaults(defineProps<{
  isOpen: boolean;
  title?: string;
  confirmText?: string;
  items: FileItem[];
  currentPath: string;
  loading?: boolean;
}>(), {
  title: 'Choose Destination',
  confirmText: 'Confirm',
  loading: false
});

const emit = defineEmits<{
  (e: 'update:isOpen', value: boolean): void;
  (e: 'navigate', path: string): void;
  (e: 'confirm', path: string): void;
}>();

const selectedPath = ref<string | null>(null);

// Reset selection when modal opens or path changes
watch([() => props.isOpen, () => props.currentPath], () => {
  selectedPath.value = null;
});

const close = () => {
  emit('update:isOpen', false);
};

const onNavigate = (path: string) => {
  selectedPath.value = null;
  emit('navigate', path);
};

const onSelect = (path: string) => {
  selectedPath.value = path;
};

const confirmSelection = () => {
  const finalPath = selectedPath.value || props.currentPath;
  emit('confirm', finalPath);
  close();
};
</script>

<style scoped>
/* FileSystemBase handles its own internal styles now.
   ModalDialog handles the modal frame.
   We just remove margins to make the file list flush with the modal body boundaries. */
</style>
