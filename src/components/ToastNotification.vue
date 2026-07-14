<template>
  <Teleport to="body">
    <TransitionGroup name="toast" tag="div" class="toast-container">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        class="toast"
        :class="`toast--${toast.type}`"
        @click="removeToast(toast.id)"
      >
        <div class="toast-icon">
          <span v-if="toast.type === 'success'">✓</span>
          <span v-else-if="toast.type === 'error'">✕</span>
          <span v-else-if="toast.type === 'warning'">!</span>
          <span v-else>i</span>
        </div>
        <div class="toast-content">
          <p class="toast-title" v-if="toast.title">{{ toast.title }}</p>
          <p class="toast-message">{{ toast.message }}</p>
        </div>
        <button class="toast-close" @click.stop="removeToast(toast.id)">×</button>
      </div>
    </TransitionGroup>
  </Teleport>
</template>

<script setup lang="ts">
import { ref } from 'vue';

export interface Toast {
  id: number;
  type: 'success' | 'error' | 'warning' | 'info';
  title?: string;
  message: string;
}

const toasts = ref<Toast[]>([]);
let nextId = 0;

function addToast(type: Toast['type'], message: string, title?: string, duration = 4000) {
  const id = nextId++;
  toasts.value.push({ id, type, message, title });
  
  if (duration > 0) {
    setTimeout(() => removeToast(id), duration);
  }
}

function removeToast(id: number) {
  toasts.value = toasts.value.filter(t => t.id !== id);
}

defineExpose({ addToast, removeToast });
</script>

<style scoped>
.toast-container {
  position: fixed;
  bottom: 1.5rem;
  right: 1.5rem;
  z-index: 9999;
  display: flex;
  flex-direction: column-reverse;
  gap: 0.5rem;
  pointer-events: none;
}

.toast {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
  padding: 0.875rem 1rem;
  background: rgba(20, 20, 20, 0.95);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
  min-width: 300px;
  max-width: 420px;
  pointer-events: all;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.toast:hover {
  border-color: var(--border-hover);
  transform: translateX(-2px);
}

.toast-icon {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.6875rem;
  font-weight: 700;
  flex-shrink: 0;
  margin-top: 1px;
}

.toast--success .toast-icon {
  background: var(--success-subtle);
  color: var(--success);
  border: 1px solid rgba(12, 206, 107, 0.3);
}

.toast--error .toast-icon {
  background: var(--error-subtle);
  color: var(--error);
  border: 1px solid rgba(238, 0, 0, 0.3);
}

.toast--warning .toast-icon {
  background: var(--warning-subtle);
  color: var(--warning);
  border: 1px solid rgba(245, 166, 35, 0.3);
}

.toast--info .toast-icon {
  background: rgba(0, 112, 243, 0.1);
  color: var(--accent);
  border: 1px solid rgba(0, 112, 243, 0.3);
}

.toast-content {
  flex: 1;
  min-width: 0;
}

.toast-title {
  font-weight: 600;
  font-size: 0.8125rem;
  color: var(--text-primary);
  margin-bottom: 0.125rem;
}

.toast-message {
  font-size: 0.8125rem;
  color: var(--text-secondary);
  line-height: 1.4;
}

.toast-close {
  background: none;
  border: none;
  color: var(--text-tertiary);
  font-size: 1.125rem;
  cursor: pointer;
  padding: 0;
  line-height: 1;
  transition: color var(--transition-fast);
}

.toast-close:hover {
  color: var(--text-primary);
}

/* Transitions */
.toast-enter-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.toast-leave-active {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(100%) scale(0.95);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(100%) scale(0.95);
}
</style>
